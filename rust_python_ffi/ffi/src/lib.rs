
use pyo3::{prelude::*, types::PyDict};
use vm_core::{VM, HintRunner, Memory, MemoryProxy};

// #[pyclass]
// struct PyCellVM {
//     vm: PyCell<PyVM>,
// }

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

    pub fn memset(&mut self, n: usize, m: usize) {
        self.vm.memset(n, m)
    }

    pub fn memget(&self, i: usize) -> usize {
        self.vm.memget(i)
    }

    pub fn load(&mut self, program: &str) {
        self.vm.load(program);
    }

    pub fn run(&mut self) -> PyResult<()> {
        self.vm.run();
        Ok(())
    }

}

struct PythonHintRunner {

}

impl PythonHintRunner {
    pub fn new() -> PythonHintRunner {
        PythonHintRunner {}
    }
}

#[pyclass]
struct PythonMemoryProxy {
    mp: MemoryProxy>,
}

impl HintRunner for PythonHintRunner {
    fn run_hint(&self, memory: &mut Memory, code: &str) -> Result<(), ()> {
        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            let mp = MemoryProxy::new(memory);
            let vm_cell = PyCell::new(py, mp).unwrap();
            //locals.set_item("vm", vm_cell).unwrap();

            py.run(
                code,
                None,
                Some(locals),
            ).unwrap();

            let rv: u32 = locals.get_item("rv").unwrap().extract().unwrap();
            println!("{:?}", rv);
        });
        Ok(())
    }    
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn ffi(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyVM>()?;
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}