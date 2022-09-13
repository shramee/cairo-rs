use pyo3::PyRefMut;
use pyo3::{
    prelude::*,
    types::{self, PyCFunction, PyDict},
};

use python_executor::vm_poc::{to_pymem, Memory, PyMemory};
// use pyo3::AsPyPointer;

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let mem = Memory {
            data: vec![0, 0, 1],
        };

        let pymem = to_pymem(mem);
        let memory: &PyCell<PyMemory> = PyCell::new(py, pymem).unwrap();

        let locals = PyDict::new(py);

        let mem_insert = |args: &types::PyTuple, _kwargs: Option<&types::PyDict>| -> PyResult<()> {
            let mut mem: PyRefMut<PyMemory> = args.get_item(0).unwrap().extract().unwrap();
            let n: u32 = args.get_item(1).unwrap().extract().unwrap();
            mem.insert(n);
            Ok(())
        };

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

        // pymem = *mem;

        println!("Memory from Rust back again: {:?}", mem);

        // println!("Original memory: {:?}", pymem);
        Ok(())
    })
}

// #[pymodule]
// pub fn vm_poc(_py: Python, m: &PyModule) -> PyResult<()> {
//     m.add_function(wrap_pyfunction!(mem_insert, m)?)?;
//     m.add_function(wrap_pyfunction!(get_mem, m)?)?;
//     Ok(())
// }
