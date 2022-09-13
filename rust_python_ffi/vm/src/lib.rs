use pyo3::{prelude::*, types::{PyDict, PyCFunction, self}};

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

fn execute_hint() -> PyResult<()> {
    let mem_insert = |args: &types::PyTuple, _kwargs: Option<&types::PyDict>|
    -> PyResult<()> {
    let mut mem: PyRefMut<PyMemory> = 
        args
        .get_item(0)
        .unwrap()
        .extract()
        .unwrap();

    let n: u32 = 
        args
        .get_item(1)
        .unwrap()
        .extract()
        .unwrap();

    mem.insert(n);

    Ok(())
    };

    Python::with_gil(|py| {
        let mem = Memory {
            data: vec![0, 0, 1],
        };

        let pymem = to_pymem(mem);
        let memory: &PyCell<PyMemory> = PyCell::new(py, pymem).unwrap();

        let locals = PyDict::new(py);

        let mem_insert_func = PyCFunction::new_closure(mem_insert, py).unwrap();

        locals.set_item("memory", memory).unwrap();
        locals.set_item("mem_insert", mem_insert_func).unwrap();
        py.run(
            r#"
print("Memory before method insert: ", memory.data)
memory.insert(2)
print("Memory after method insert: ", memory.data)
mem_insert(memory, 2)
print("Memory after function insert: ", memory.data)
print("Memory object in Python: ", memory)
        "#,
            None,
            Some(locals),
        )
        .unwrap();

        // let mem = locals.get_item("memory").unwrap().extract::<PyRef<PyMemory>>().unwrap().as_ptr();
        let mem: PyRefMut<PyMemory> = locals.get_item("memory").unwrap().extract().unwrap();

        println!("Memory from Rust back again: {:?}", mem);

        // println!("Original memory: {:?}", pymem);
        Ok(())
    })
}

#[pyfunction]
fn run() -> PyResult<()> {
    execute_hint()
}

#[pymodule]
pub fn vm(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run, m)?)?;
    Ok(())
}
