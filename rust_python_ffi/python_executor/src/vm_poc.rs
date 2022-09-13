use pyo3::prelude::*;

#[derive(Debug)]
pub struct Memory {
    pub data: Vec<u32>,
}

impl Memory {
    pub fn push(&mut self, n: u32) {
        self.data.push(n);
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyMemory {
    #[pyo3(get, set)]
    pub data: Vec<u32>,
}

#[pymethods]
impl PyMemory {
    #[new]
    pub fn py_new() -> PyResult<Self> {
        Ok(PyMemory { data: Vec::new() })
    }

    pub fn insert(&mut self, n: u32) {
        self.data.push(n)
    }
}

pub fn to_pymem(mem: Memory) -> PyMemory {
    PyMemory {
        data: mem.data.clone(),
    }
}

// For making a Python module that runs Rust code.

// Push a new element to the top of the memory.
#[pyfunction]
pub fn mem_insert(mem: &mut PyMemory, n: u32) {
    mem.insert(n)
}

// #[pyfunction]
// fn get_mem() -> PyResult<PyMemory> {
//     let mem = Memory {
//         data: vec![0, 1, 1],
//     };
//     Ok(to_pymem(mem))
// }

// #[pymodule]
// pub fn vm_poc(_py: Python, m: &PyModule) -> PyResult<()> {
//     m.add_function(wrap_pyfunction!(mem_insert, m)?)?;
//     m.add_function(wrap_pyfunction!(get_mem, m)?)?;
//     Ok(())
// }
