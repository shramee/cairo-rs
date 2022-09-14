#[derive(Debug)]
pub struct VM {
    memory: Vec<usize>,
    code: Vec<usize>,
    ip: usize,
    //hint_runner: (),
}

impl VM {
    pub fn new() -> VM {
        VM { memory: vec![0; 128], code: vec![0; 128], ip: 0 }//, hint_runner: () }
    }

    pub fn memset(&mut self, n: usize, m: usize) {
        self.memory[n] = m;
    }

    pub fn memget(&self, i: usize) -> usize {
        self.memory[i]
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
                1 => {
                    println!("opcode 1: noop")
                },
                2 => {
                    // EXECUTE HINT
                },
                _ => {
                    return Err(())
                }
            }
            self.ip += 1;
        }
        Ok(())
    }
}
 
#[cfg(test)]
mod tests {
    use crate::VM;

    #[test]
    fn memset_test() {
        let mut vm = VM::new();
        vm.memset(0, 1);

        assert_eq!(vm.memget(0), 1);
    }
}