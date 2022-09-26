use num_bigint::BigInt;
use std::fmt::{self, Debug};

pub trait HintRunner {
    fn run_hint(&self, vm: &mut VM, memory: Option<Memory>, code: &str) -> Result<(), ()>;
}

#[derive(Debug, Clone)]
pub struct Memory {
    data: Vec<Vec<Option<BigInt>>>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            data: vec![vec![Some(BigInt::parse_bytes(b"0", 10).unwrap()); 1024]; 1024],
        }
    }

    pub fn set(&mut self, addr: (usize, usize), m: BigInt) {
        self.data[addr.0][addr.1] = Some(m);
    }

    pub fn get(&self, addr: (usize, usize)) -> Option<&BigInt> {
        self.data[addr.0][addr.1].as_ref()
    }
}

pub struct VM {
    pub memory: Memory,
    code: Vec<usize>,
    ip: usize,
}

impl VM {
    pub fn new() -> VM {
        VM {
            memory: Memory::new(),
            code: vec![0; 32],
            ip: 0,
        }
    }

    pub fn load(&mut self, code: &str) {
        for (i, c) in code.chars().enumerate() {
            self.code[i] = c.to_digit(10).unwrap() as usize;
        }
    }

    pub fn run(&mut self, hint_runner: &Box<dyn HintRunner>) -> Result<(), ()> {
        // opcode 0: exit
        // opcode 1: noop
        // opcode 2: execute hint
        while self.code[self.ip] != 0 {
            match self.code[self.ip] {
                1 => {}
                2 => {
                    self.run_hint(hint_runner);
                }
                _ => return Err(()),
            }
            self.ip += 1;
        }
        Ok(())
    }

    fn run_hint(&mut self, hint_runner: &Box<dyn HintRunner>) {
        let code = r#"
rv = fibonacci(x)
for i in range(1024):
    for j in range(1024):
        memory[(i, j)] = i*j
"#;
        let memory = self.memory.clone();
        hint_runner.run_hint(self, Some(memory), code).unwrap();
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
