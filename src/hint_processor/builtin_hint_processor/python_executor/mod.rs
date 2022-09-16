mod pybigint;
mod pyrelocatable;
mod python_executor_helpers;

use pybigint::PyBigInt;
use pyrelocatable::*;
use python_executor_helpers::*;

use std::{any::Any, collections::HashMap, thread};

use crate::{
    any_box,
    hint_processor::hint_processor_definition::HintReference,
    pycell,
    serde::deserialize_program::ApTracking,
    types::{exec_scope::ExecutionScopes, relocatable::MaybeRelocatable},
    vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};

use super::builtin_hint_processor_definition::HintProcessorData;
use crossbeam_channel::{unbounded, Receiver, Sender};
use pyo3::{exceptions::PyTypeError, prelude::*, types::PyDict};

const CHANNEL_ERROR_MSG: &str = "Failed to communicate between channels";

#[derive(Debug)]
pub enum Operation {
    AddSegment,
    WriteMemory(PyRelocatable, PyMaybeRelocatable),
    ReadMemory(PyRelocatable),
    ReadIds(String),
    WriteIds(String, PyMaybeRelocatable),
    WriteVecArg(PyRelocatable, Vec<PyMaybeRelocatable>),
    End,
    EnterScope(HashMap<String, PyObject>),
    ExitScope,
}

#[derive(Debug)]
pub enum OperationResult {
    ReadValue(PyMaybeRelocatable),
    Segment(PyRelocatable),
    Success,
}

#[pyclass]
pub struct PySegmentManager {
    operation_sender: Sender<Operation>,
    result_receiver: Receiver<OperationResult>,
}

#[pymethods]
impl PySegmentManager {
    pub fn add(&self) -> PyResult<PyRelocatable> {
        send_operation(&self.operation_sender, Operation::AddSegment)?;
        if let OperationResult::Segment(result) = self
            .result_receiver
            .recv()
            .map_err(|_| PyTypeError::new_err(CHANNEL_ERROR_MSG))?
        {
            return Ok(result);
        }
        Err(PyTypeError::new_err("segments.add() failure"))
    }
    pub fn write_arg(&self, ptr: PyRelocatable, arg: Vec<PyMaybeRelocatable>) -> PyResult<()> {
        send_operation(&self.operation_sender, Operation::WriteVecArg(ptr, arg))?;
        check_operation_success(&self.result_receiver, "segments.write_arg()")
    }
}

impl PySegmentManager {
    pub fn new(
        operation_sender: Sender<Operation>,
        result_receiver: Receiver<OperationResult>,
    ) -> PySegmentManager {
        PySegmentManager {
            operation_sender,
            result_receiver,
        }
    }
}

#[pyclass]
pub struct PyMemory {
    operation_sender: Sender<Operation>,
    result_receiver: Receiver<OperationResult>,
}

#[pymethods]
impl PyMemory {
    pub fn __getitem__(&self, key: &PyRelocatable, py: Python) -> PyResult<PyObject> {
        send_operation(
            &self.operation_sender,
            Operation::ReadMemory(PyRelocatable::new((key.index, key.offset))),
        )?;
        get_read_value_result(&self.result_receiver, "memory.__getitem__()", &py)
    }

    pub fn __setitem__(&self, key: &PyRelocatable, value: PyMaybeRelocatable) -> PyResult<()> {
        send_operation(
            &self.operation_sender,
            Operation::WriteMemory(PyRelocatable::new((key.index, key.offset)), value),
        )?;
        check_operation_success(&self.result_receiver, "memory.__setitem__()")
    }
}

impl PyMemory {
    pub fn new(
        operation_sender: Sender<Operation>,
        result_receiver: Receiver<OperationResult>,
    ) -> PyMemory {
        PyMemory {
            operation_sender,
            result_receiver,
        }
    }
}

#[pyclass]
pub struct PyIds {
    operation_sender: Sender<Operation>,
    result_receiver: Receiver<OperationResult>,
}

#[pymethods]
impl PyIds {
    pub fn __getattr__(&self, name: &str, py: Python) -> PyResult<PyObject> {
        send_operation(&self.operation_sender, Operation::ReadIds(name.to_string()))?;
        get_read_value_result(&self.result_receiver, "ids.__getattr__()", &py)
    }

    pub fn __setattr__(&self, name: &str, value: PyMaybeRelocatable) -> PyResult<()> {
        send_operation(
            &self.operation_sender,
            Operation::WriteIds(name.to_string(), value),
        )?;
        check_operation_success(&self.result_receiver, "ids.__setattr__()")
    }
}

impl PyIds {
    pub fn new(
        operation_sender: Sender<Operation>,
        result_receiver: Receiver<OperationResult>,
    ) -> PyIds {
        PyIds {
            operation_sender,
            result_receiver,
        }
    }
}

#[pyfunction]
#[pyo3(pass_module)]
fn vm_exit_scope(module: &PyModule) -> PyResult<()> {
    (module.getattr("scope")? as &dyn Any)
        .downcast_ref::<PyScope>()
        .unwrap()
        .exit_scope()
}

//TODO: Add vm_enter_scope

type PyScopeDict = HashMap<String, PyObject>;

#[pyclass]
pub struct PyScope {
    operation_sender: Sender<Operation>,
    result_receiver: Receiver<OperationResult>,
}

#[pymethods]
impl PyScope {
    pub fn exit_scope(&self) -> PyResult<()> {
        send_operation(&self.operation_sender, Operation::ExitScope)?;
        check_operation_success(&self.result_receiver, "vm_exit_scope")
    }

    pub fn enter_scope(&self, new_scope: HashMap<String, PyObject>) -> PyResult<()> {
        send_operation(&self.operation_sender, Operation::EnterScope(new_scope))?;
        check_operation_success(&self.result_receiver, "vm_enter_scope")
    }
}

impl PyScope {
    pub fn new(
        operation_sender: Sender<Operation>,
        result_receiver: Receiver<OperationResult>,
    ) -> PyScope {
        PyScope {
            operation_sender,
            result_receiver,
        }
    }
}

fn handle_messages(
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
    operation_receiver: Receiver<Operation>,
    result_sender: Sender<OperationResult>,
    vm: &mut VirtualMachine,
) -> Result<(Option<PyScopeDict>, bool), VirtualMachineError> {
    let mut new_scope: Option<PyScopeDict> = None;
    let mut exit_scope = false;
    loop {
        match operation_receiver
            .recv()
            .map_err(|_| VirtualMachineError::PythonExecutorChannel)?
        {
            Operation::End => break,
            Operation::ReadMemory(address) => {
                if let Some(value) = vm.memory.get(&address.to_relocatable())? {
                    send_result(&result_sender, OperationResult::ReadValue(value.into()))?;
                };
            }
            Operation::WriteMemory(key, value) => {
                vm.memory.insert(
                    &key.to_relocatable(),
                    &(Into::<MaybeRelocatable>::into(value)),
                )?;
                send_result(&result_sender, OperationResult::Success)?;
            }
            Operation::AddSegment => {
                let result = vm.segments.add(&mut vm.memory);
                send_result(&result_sender, OperationResult::Segment(result.into()))?;
            }
            Operation::ReadIds(name) => {
                let hint_ref = ids_data
                    .get(&name)
                    .ok_or(VirtualMachineError::FailedToGetIds)?;
                let value = get_value_from_reference(vm, hint_ref, ap_tracking)?;
                send_result(&result_sender, OperationResult::ReadValue(value.into()))?;
            }
            Operation::WriteIds(name, value) => {
                let hint_ref = ids_data
                    .get(&name)
                    .ok_or(VirtualMachineError::FailedToGetIds)?;
                let addr = compute_addr_from_reference(
                    hint_ref,
                    &vm.run_context,
                    &vm.memory,
                    ap_tracking,
                )?;
                vm.memory
                    .insert(&addr, &(Into::<MaybeRelocatable>::into(value)))?;
                send_result(&result_sender, OperationResult::Success)?;
            }
            Operation::WriteVecArg(ptr, arg) => {
                write_py_vec_args(&mut vm.memory, &ptr, &arg, &vm.prime)?;
                send_result(&result_sender, OperationResult::Success)?;
            }
            //TODO: Handle this case
            //Proposed solution: Make this function return an enum ScopeOperation::Exit
            //(Other variants should be Enter and NoOp)
            Operation::ExitScope => {
                exit_scope = true;
                println!("VM_EXIT_SCOPE");
            }
            //TODO: Handle exter_scope -> return ScopeOperation::Enter(vars)(possible conflicts with PyAny and threads)
            Operation::EnterScope(scope) => {
                new_scope = Some(scope);
                send_result(&result_sender, OperationResult::Success)?;
            }
        }
    }
    Ok((new_scope, exit_scope))
}

pub struct PythonExecutor {}

impl PythonExecutor {
    pub fn execute_hint(
        vm: &mut VirtualMachine,
        hint_data: &Box<dyn Any>,
        exec_scopes: &mut ExecutionScopes,
    ) -> Result<(), VirtualMachineError> {
        let hint_data = hint_data
            .downcast_ref::<HintProcessorData>()
            .ok_or(VirtualMachineError::WrongHintData)?;
        let code = hint_data.code.clone();
        let (operation_sender, operation_receiver) = unbounded();
        let (result_sender, result_receiver) = unbounded();
        let ap = vm.run_context.ap;
        let fp = vm.run_context.fp;
        pyo3::prepare_freethreaded_python();
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_scope = get_py_scope(exec_scopes)?;
        let (new_vars, new_scope, exit_scope) = py
            .allow_threads(
                move || -> Result<(PyScopeDict, Option<PyScopeDict>, bool), VirtualMachineError> {
                    let new_vars =
                        thread::spawn(move || -> Result<PyScopeDict, VirtualMachineError> {
                            println!(" -- Starting python hint execution -- ");
                            let gil = Python::acquire_gil();
                            let py = gil.python();
                            let memory = pycell!(
                                py,
                                PyMemory::new(operation_sender.clone(), result_receiver.clone())
                            );
                            let segments = pycell!(
                                py,
                                PySegmentManager::new(
                                    operation_sender.clone(),
                                    result_receiver.clone()
                                )
                            );
                            let scope = pycell!(
                                py,
                                PyScope::new(operation_sender.clone(), result_receiver.clone())
                            );
                            let ids =
                                pycell!(py, PyIds::new(operation_sender.clone(), result_receiver));
                            let ap = pycell!(py, PyRelocatable::new((1, ap)));
                            let fp = pycell!(py, PyRelocatable::new((1, fp)));
                            let scope_module = PyModule::new(py, "cairo_scopes")?;
                            let vm_exit_scope = wrap_pyfunction!(vm_exit_scope, scope_module)?;
                            let globals = PyDict::new(py);
                            let locals = PyDict::new(py);
                            for (name, value) in py_scope.iter() {
                                locals
                                    .set_item(name, value)
                                    .map_err(Into::<VirtualMachineError>::into)?;
                            }
                            globals.set_item("memory", memory).unwrap();
                            globals.set_item("ids", ids).unwrap();
                            globals.set_item("segments", segments).unwrap();
                            globals.set_item("scope", scope).unwrap();
                            globals.set_item("ap", ap).unwrap();
                            globals.set_item("fp", fp).unwrap();
                            globals.set_item("vm_exit_scope", vm_exit_scope).unwrap();
                            py.run(&code, Some(globals), Some(locals)).unwrap();
                            println!(" -- Ending python hint -- ");
                            operation_sender
                                .send(Operation::End)
                                .map_err(|_| VirtualMachineError::PythonExecutorChannel)?;
                            Ok(get_scope_variables(locals, py))
                        });
                    let (new_scope, exit_scope) = handle_messages(
                        &hint_data.ids_data,
                        &hint_data.ap_tracking,
                        operation_receiver,
                        result_sender,
                        vm,
                    )?;
                    let new_vars = new_vars.join().unwrap().unwrap();
                    Ok((new_vars, new_scope, exit_scope))
                },
            )
            .unwrap();
        update_scope(exec_scopes, &new_vars);
        // This is a pretty horrible wip. Something similar could be done for exit_scope
        // (like making the thread return a boolean/the amount of times exit_scope should be called)
        if let Some(scope) = new_scope {
            enter_scope(exec_scopes, &scope);
        }
        if exit_scope {
            exec_scopes.exit_scope()?;
        }
        Ok(())
    }
}

fn get_py_scope(
    exec_scopes: &mut ExecutionScopes,
) -> Result<HashMap<String, PyObject>, VirtualMachineError> {
    let mut py_scope = HashMap::<String, PyObject>::new();
    for (name, value) in exec_scopes
        .get_local_variables()
        .ok_or(VirtualMachineError::ScopeError)?
        .iter()
    {
        py_scope.insert(
            name.to_string(),
            value
                .downcast_ref::<PyObject>()
                .ok_or(VirtualMachineError::ScopeError)?
                .clone(),
        );
    }
    Ok(py_scope)
}

fn update_scope(
    exec_scopes: &mut ExecutionScopes,
    new_scope_variables: &HashMap<String, PyObject>,
) {
    for (name, pyobj) in new_scope_variables {
        exec_scopes.assign_or_update_variable(name, any_box!(pyobj.clone()))
    }
}

fn get_scope_variables(locals: &PyDict, py: Python) -> HashMap<String, PyObject> {
    let mut new_locals = HashMap::<String, PyObject>::new();
    for (name, pyvalue) in locals.iter() {
        new_locals.insert(name.to_string(), pyvalue.to_object(py));
    }
    new_locals
}

fn enter_scope(exec_scopes: &mut ExecutionScopes, new_scope: &HashMap<String, PyObject>) {
    let mut transformed_scope = HashMap::new();
    for (key, value) in new_scope.iter() {
        transformed_scope.insert(key.to_string(), any_box!(value.clone()));
    }
    exec_scopes.enter_scope(transformed_scope);
}

fn send_result(
    sender: &Sender<OperationResult>,
    result: OperationResult,
) -> Result<(), VirtualMachineError> {
    sender
        .send(result)
        .map_err(|_| VirtualMachineError::PythonExecutorChannel)
}

fn send_operation(sender: &Sender<Operation>, operation: Operation) -> Result<(), PyErr> {
    sender
        .send(operation)
        .map_err(|_| PyTypeError::new_err(CHANNEL_ERROR_MSG))
}

fn check_operation_success(
    receiver: &Receiver<OperationResult>,
    method_name: &str,
) -> Result<(), PyErr> {
    if let OperationResult::Success = receiver
        .recv()
        .map_err(|_| PyTypeError::new_err(CHANNEL_ERROR_MSG))?
    {
        return Ok(());
    }
    let string = format!("{} failure", method_name);
    Err(PyTypeError::new_err(string))
}

fn get_read_value_result(
    receiver: &Receiver<OperationResult>,
    method_name: &str,
    py: &Python,
) -> PyResult<PyObject> {
    if let OperationResult::ReadValue(result) = receiver
        .recv()
        .map_err(|_| PyTypeError::new_err(CHANNEL_ERROR_MSG))?
    {
        return Ok(result.to_object(*py));
    }
    let string = format!("{} failure", method_name);
    Err(PyTypeError::new_err(string))
}
