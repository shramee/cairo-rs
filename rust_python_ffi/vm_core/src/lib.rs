use std::collections::HashMap;
use std::fmt::{self, Debug};
use num_bigint::BigInt;

#[derive(Debug)]
pub enum VMErr {
    UnknownHint,
}

pub trait HintRunner {
    fn run_hint(&self, vm: &mut VM, hint_code: &String) -> Result<(), VMErr>;
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
    pub memory: Memory,
    pub code: Vec<usize>,
    pub ip: usize,
    // hint_runner: Option<Box<dyn HintRunner>>,
    pub hint_codes: HashMap<usize, String>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            memory: Memory::new(),
            code: vec![0; 32],
            ip: 0,
            hint_codes: HashMap::new(),
        }
    }

    pub fn load(&mut self, code: &str) {
        for (i, c) in code.chars().enumerate() {
            self.code[i] = c.to_digit(10).unwrap() as usize;
        }
    }

    pub fn initialize(&mut self, code: &str) {
        let hint_code = r#"
rv = fibonacci(x)
for i in range(1024):
    for j in range(1024):
       memory[(i, j)] = i*j
print("Hint run succesfully from Python!")
"#;
        self.hint_codes.insert(4, hint_code.to_string());
        self.load(code);
    }

    pub fn step_instruction(&mut self) -> Result<(), ()> {
        // opcode 0: exit
        // opcode 1: noop
        // opcode 2: execute hint
        println!("Running awesome instruction");
        self.ip += 1;
        Ok(())
    }

    pub fn step_hint(&mut self, hint_runner: &Box<dyn HintRunner>) -> Result<(), ()> {
        if self.code[self.ip] == 2 {
            self.run_hint(hint_runner);
        }

        Ok(())
    }

    pub fn step(&mut self, hint_runner: &Box<dyn HintRunner>) -> Result<(), ()> {
        self.step_hint(hint_runner)?;
        self.step_instruction()?;

        Ok(())
    }

    pub fn run_to_end(&mut self, hint_runner: &Box<dyn HintRunner>) -> Result<(), ()> {
        while self.code[self.ip] != 0 {
            self.step(hint_runner)?;
        }

        Ok(())
    }

    pub fn print_output(&self) {
        println!("Successful run");
    }

    pub fn run_hint(&mut self, hint_runner: &Box<dyn HintRunner>) {
        if let Some(hint_code) = self.hint_codes.get(&self.ip) {
            let hint_code = hint_code.clone();
            hint_runner.run_hint(self, &hint_code).unwrap();
        }
    }
}

pub fn cairo_run(code: &str, hint_runner: &Box<dyn HintRunner>) -> Result<(),()> {
    let mut vm = VM::new();
    vm.initialize(code);
    vm.run_to_end(hint_runner)?;
    vm.print_output();

    Ok(())
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
