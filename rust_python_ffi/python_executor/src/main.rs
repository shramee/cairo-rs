use vm_poc::{to_pymem, Memory, PyMemory};
use pyo3::{prelude::*, types::PyDict};

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let mem = Memory {
            data: vec![0, 0, 1],
        };

        let pymem = to_pymem(mem);
        println!("Initial memory: {:?}", pymem.data);
        let memory: &PyCell<PyMemory> = PyCell::new(py, pymem).unwrap();
        //println!("Pointer PyCell: {:?}", memory.get_type_ptr());

        let locals = PyDict::new(py);
        locals.set_item("memory", memory).unwrap();
        py.run(
            r#"
print("Memory before insert: ", memory.data)
memory.insert(2)
#print("Memory pointer: ", memory)
print("Memory after insert: ", memory.data)

import vm_poc
vm_poc.mem_insert(memory, 2)

print("Memory after FFI insert: ", memory.data)
        "#,
            None,
            Some(locals),
        )
        .unwrap();

        //let mem = locals.get_item("memory").unwrap().get_type_ptr();
        let mem: PyMemory = locals.get_item("memory").unwrap().extract().unwrap();

        println!("Memory from Rust back again: {:?}", mem);

        Ok(())
    })
}