use num_bigint::BigInt;
use pyo3::{prelude::*, types::PyDict};
use std::cell::RefCell;
use std::rc::Rc;
use vm_core::HintRunner;
use vm_core::Memory;
use vm_core::VM;

#[pyclass(unsendable)]
pub struct PyVM {
    vm: VM,
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
    memory: Rc<RefCell<Memory>>,
}

#[pymethods]
impl PyVmMemory {
    pub fn __setitem__(&mut self, addr: (usize, usize), m: BigInt) -> PyResult<()> {
        self.memory.borrow_mut().set(addr, m);
        Ok(())
    }

    pub fn __getitem__(&self, addr: (usize, usize)) -> Option<BigInt> {
        self.memory.borrow_mut().get(addr).cloned()
    }
}

struct PythonHintRunner {}

impl PythonHintRunner {
    pub fn new() -> PythonHintRunner {
        PythonHintRunner {}
    }
}

impl HintRunner for PythonHintRunner {
    fn run_hint(&self, memory: Option<Rc<RefCell<Memory>>>, code: &str) -> Result<(), ()> {
        Python::with_gil(|py| {
            let locals = PyDict::new(py);

            if let Some(m) = memory {
                let memory = PyVmMemory { memory: m };
                let vmm = PyCell::new(py, memory).unwrap();
                locals.set_item("memory", vmm).unwrap();

                locals.set_item("x", 7u32).unwrap();
                py.run(code, None, Some(locals)).unwrap();
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
