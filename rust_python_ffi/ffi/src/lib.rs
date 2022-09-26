use num_bigint::BigInt;
use pyo3::{prelude::*, types::PyDict};
use std::cell::RefCell;
use std::rc::Rc;
use vm_core::HintRunner;
use vm_core::VMErr;
use vm_core::VM;

#[pyclass(unsendable)]
pub struct PyVM {
    vm: Rc<RefCell<VM>>,
}

#[pymethods]
impl PyVM {
    #[new]
    pub fn new() -> PyVM {
        PyVM {
            vm: Rc::new(RefCell::new(VM::new())),
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.vm)
    }

    pub fn load(&self, program: &str) {
        self.vm.borrow_mut().load(program);
    }
}

impl PyVM {
    pub fn run_hint(&self, hint_code: &String) -> PyResult<()> {
        Python::with_gil(|py| {
            let locals = PyDict::new(py);

            let memory = PyMemory {
                vm: Rc::clone(&self.vm),
            };
            let vmm = PyCell::new(py, memory).unwrap();
            locals.set_item("memory", vmm).unwrap();

            locals.set_item("x", 7u32).unwrap();
            py.run(&hint_code, None, Some(locals)).unwrap();
        });
        Ok(())
    }

    fn step_hint(&self, ip: usize, hint_runner: &Box<dyn HintRunner>) {
        if self.vm.borrow().code[ip] == 2 {
            let (should_run_hint, hint_code) = {
                let hint_code = self.vm.borrow().hint_codes.get(&ip).unwrap().clone();
                match hint_runner.run_hint(&mut self.vm.borrow_mut(), &hint_code) {
                    Err(VMErr::UnknownHint) => (true, Some(hint_code)),
                    res => {
                        res.unwrap();
                        (false, None)
                    }
                }
            };

            if let (true, Some(hint_code)) = (should_run_hint, hint_code) {
                self.run_hint(&hint_code).unwrap();
            }
        }
    }
    
}

struct BuiltinHintRunner {}

impl BuiltinHintRunner {
    pub fn new() -> BuiltinHintRunner {
        BuiltinHintRunner {}
    }
}

impl HintRunner for BuiltinHintRunner {
    fn run_hint(&self, _vm: &mut VM, _code: &String) -> Result<(), VMErr> {
        Err(VMErr::UnknownHint)
    }
}

#[pyfunction]
pub fn cairo_run(code: &str) {
    let pyvm = PyVM::new();
    let hint_runner: Box<dyn HintRunner> = Box::new(BuiltinHintRunner::new());
    pyvm.vm.borrow_mut().initialize(code);

    while pyvm.vm.borrow().code[pyvm.vm.borrow().ip] != 0 {
        let ip = pyvm.vm.borrow().ip;
        pyvm.step_hint(ip, &hint_runner);
        pyvm.vm.borrow_mut().step_instruction().unwrap();
    }
    pyvm.vm.borrow().print_output();
}

#[pyclass(unsendable)]
pub struct PyMemory {
    vm: Rc<RefCell<VM>>,
}

#[pymethods]
impl PyMemory {
    pub fn __setitem__(&mut self, addr: (usize, usize), m: BigInt) -> PyResult<()> {
        self.vm.borrow_mut().memory.set(addr, m);
        Ok(())
    }

    pub fn __getitem__(&self, addr: (usize, usize)) -> Option<BigInt> {
        self.vm.borrow().memory.get(addr).cloned()
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn ffi(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyVM>()?;
    // m.add_class::<PyMemory>()?;
    m.add_wrapped(wrap_pyfunction!(cairo_run)).unwrap();
    Ok(())
}
