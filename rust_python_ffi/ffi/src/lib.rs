use std::collections::HashMap;

use num_bigint::BigInt;
use pyo3::{prelude::*, types::PyDict};
use vm_core::VM;
use vm_core::HintRunner;
use vm_core::Memory;

#[pyclass(unsendable)]
pub struct PyVM {
    vm: VM
}

#[pymethods]
impl PyVM {
    #[new]
    pub fn new() -> PyVM {
        PyVM {
            vm: VM::new(),
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.vm)
    }

    pub fn load(&mut self, program: &str) {
        self.vm.load(program);
    }

    pub fn run(&mut self) -> PyResult<()> {
        let hint_runner: Box<dyn HintRunner> = Box::new(PythonHintRunner::new());
        self.vm.run(&hint_runner).unwrap();
        Ok(())
    }
}

#[pyclass(unsendable)]
#[derive(Clone)]
pub struct PyVmMemory {
    memory: Memory,
    changes: HashMap<(usize, usize), BigInt>,
}

#[pymethods]
impl PyVmMemory {
    #[setter]
    pub fn __setitem__(&mut self, addr: (usize, usize), m: BigInt) -> PyResult<()> {
        self.changes.insert((addr.0, addr.1), m);
        Ok(())
    }

    #[getter]
    pub fn __getitem__(&self, addr: (usize, usize)) -> Option<BigInt> {
        self.memory.get(addr).cloned()
    }
}

struct PythonHintRunner {}

impl PythonHintRunner {
    pub fn new() -> PythonHintRunner {
        PythonHintRunner{}
    }
}

impl HintRunner for PythonHintRunner {
    fn run_hint(&self, vm: &mut VM, memory: Option<Memory>, code: &str) -> Result<(), ()> {
        Python::with_gil(|py| {
            let locals = PyDict::new(py);

            if let Some(m) = memory {
                let memory = PyVmMemory { memory: m, changes: HashMap::new() };
                let vmm = PyCell::new(py, memory).unwrap();
                locals.set_item("memory", vmm).unwrap();
                locals.set_item("x", 7u32).unwrap();

                py.run(
                    code,
                    None,
                    Some(locals),
                ).unwrap();

                let changes = locals.get_item("memory").unwrap().extract::<PyVmMemory>().unwrap().changes;

                for (key, value) in changes {
                    vm.memory.set(key, value);
                }
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
