
use pyo3::{prelude::*, types::PyDict};
use pyo3::PyRef;
use vm_core::VM;

// #[pyclass]
// struct PyCellVM {
//     vm: PyCell<PyVM>,
// }

#[pyclass]
pub struct PyVM {
    vm: VM,
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

    pub fn run_hint(&mut self) {
        let code = "rv = fibonacci(7)";
        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            //let vm_cell = PyCell::new(py, vm).unwrap();
            //locals.set_item("vm", vm_cell).unwrap();

            py.run(
                code,
                None,
                Some(locals),
            ).unwrap();

            let rv: u32 = locals.get_item("rv").unwrap().extract().unwrap();
            println!("{:?}", rv);
        })

    }
}

// fn run_hint(code: &str) -> PyResult<()> {
// }

/*
n = fibonacci(42)
vm.memset(0, n)
*/

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