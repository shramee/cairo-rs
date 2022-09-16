use pyo3::{prelude::*, types::PyDict};
use vm_core::VM;
use vm_core::HintRunner;
use vm_core::Memory;
use std::rc::Rc;
use std::cell::RefCell;

#[pyclass(unsendable)]
pub struct PyVM {
    vm: VM
}

#[pymethods]
impl PyVM {
    #[new]
    pub fn new() -> PyVM {
        let hint_runner: Option<Box<dyn HintRunner>> = Some(Box::new(PythonHintRunner::new()));
        PyVM {
            vm: VM::new(hint_runner),
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.vm)
    }

    pub fn load(&mut self, program: &str) {
        self.vm.load(program);
    }

    pub fn run(&mut self) -> PyResult<()> {
        self.vm.run().unwrap();
        Ok(())
    }
}

#[pyclass(unsendable)]
pub struct PyVmMemory {
    memory: Rc<RefCell<Memory>>
}

#[pymethods]
impl PyVmMemory {
    pub fn set(&mut self, n: usize, m: usize) {
        self.memory.borrow_mut().set(n, m)
    }

    pub fn get(&self, i: usize) -> usize {
        self.memory.borrow_mut().get(i)
    }
}

struct PythonHintRunner {}

impl PythonHintRunner {
    pub fn new() -> PythonHintRunner {
        PythonHintRunner{}
    }
}

impl HintRunner for PythonHintRunner {
    fn run_hint(&self, memory: Option<Rc<RefCell<Memory>>>, code: &str) -> Result<(), ()> {
        Python::with_gil(|py| {
            let locals = PyDict::new(py);

            if let Some(m) = memory {
                let memory = PyVmMemory{memory: m};
                let vmm = PyCell::new(py, memory).unwrap();
                locals.set_item("vm_memory", vmm).unwrap();

                locals.set_item("x", 7).unwrap();
                py.run(
                    code,
                    None,
                    Some(locals),
                ).unwrap();

                let rv: u32 = locals.get_item("rv").unwrap().extract().unwrap();
                println!("rv = {:?}", rv);

                let rv = vmm.borrow_mut().get(16);
                println!("vmm[16] = {:?}", rv);
            }

        });
        Ok(())
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn ffi(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyVM>()?;
    m.add_class::<PyVmMemory>()?;
    Ok(())
}
