use std::fmt::{self, Debug};
use std::rc::Rc;
use std::cell::RefCell;
use num_bigint::BigInt;

pub trait HintRunner {
    fn run_hint(&self, memory: Option<Rc<RefCell<Memory>>>, code: &str) -> Result<(), ()>;
}

#[derive(Debug)]
pub struct Memory {
    data: Vec<Vec<Option<BigInt>>>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory{data: vec![vec![BigInt::parse_bytes(b"0", 10); 1024]; 1024] }
    }

    pub fn set(&mut self, addr: (usize, usize), m: BigInt) {
        self.data[addr.0][addr.1] = Some(m);
    }

    pub fn get(&self, addr: (usize, usize)) -> Option<&BigInt> {
        self.data[addr.0][addr.1].as_ref()
    }
}

pub struct VM {
    memory: Rc<RefCell<Memory>>,
    code: Vec<usize>,
    ip: usize,
    hint_runner: Option<Box<dyn HintRunner>>,
}

impl VM {
    pub fn new(hint_runner: Option<Box<dyn HintRunner>>) -> VM {
        VM {
            memory: Rc::new(RefCell::new(Memory::new())),
            code: vec![0; 32],
            ip: 0,
            hint_runner,
        }
    }

    pub fn load(&mut self, code: &str) {
        for (i, c) in code.chars().enumerate() {
            self.code[i] = c.to_digit(10).unwrap() as usize;
        }
    }

    pub fn run(&mut self) -> Result<(), ()> {
        // opcode 0: exit
        // opcode 1: noop
        // opcode 2: execute hint
        while self.code[self.ip] != 0 {
            match self.code[self.ip] {
                1 => {}
                2 => { self.run_hint(); }
                _ => return Err(()),
            }
            self.ip += 1;
        }
        Ok(())
    }

    fn run_hint(&mut self) {
        if let Some(hint_runner) = &self.hint_runner {
            let code = r#"
rv = fibonacci(x)
for i in range(1024):
    for j in range(1024):
        memory[(i, j)] = i*j
"#;
            hint_runner.run_hint(Some(Rc::clone(&self.memory)), code).unwrap();
        }
    }
}

impl fmt::Debug for VM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VM")
            .field("memory", &self.memory)
            .field("code", &self.code)
            .field("ip", &self.ip)
            .finish()
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::VM;

//     #[test]
//     fn memset_test() {
//         let mut vm = VM::new();
//         vm.memset(0, 1);
//         assert_eq!(vm.memget(0), 1);
//     }
// }
