use crate::stack::Stack;
use crate::memory::Memory;
use crate::opcode::Opcode;

#[derive(Debug)]

pub struct VM {
    pub stack: Stack,
    pub memory: Memory,
    pub pc: usize,
    pub bytecode: Vec<u8>,
    pub return_value: Option<i32>,
}


impl VM {
    pub fn new(bytecode: Vec<u8>) -> Self {
        Self {
            stack: Stack::new(),
            memory: Memory::new(),
            pc: 0,
            bytecode,
            return_value: None,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        loop {
            let should_continue = self.step()?;

            if !should_continue {
                break;
            }
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<bool, String> {
        if self.pc >= self.bytecode.len() {
            return Err("Program counter out of bounds".to_string());
        }

        let byte = self.bytecode[self.pc];

        let opcode = Opcode::try_from(byte)?;

        match opcode {
            Opcode::Stop => {
                Ok(false)
            }

            Opcode::Push => {
                if self.pc + 1 >= self.bytecode.len() {
                    return Err("Not enough bytes for Push operand".to_string());
                }
                let operand = self.bytecode[self.pc + 1] as i32;
                self.stack.push(operand);
                self.pc += 2; 

                Ok(true)
            }

            Opcode::Add => {
                let b = self.stack.pop().ok_or("Stack underflow".to_string())?;
                let a = self.stack.pop().ok_or("Stack underflow".to_string())?;
                self.stack.push(a + b);
                self.pc += 1;

                Ok(true)
            }

            Opcode::Sub => {
                let b = self.stack.pop().ok_or("Stack underflow".to_string())?;
                let a = self.stack.pop().ok_or("Stack underflow".to_string())?;
                self.stack.push(a - b);
                self.pc += 1;

                Ok(true)
            } 

            Opcode::Mul => {
                let b = self.stack.pop().ok_or("Stack underflow".to_string())?;
                let a = self.stack.pop().ok_or("Stack underflow".to_string())?;
                self.stack.push(a * b);
                self.pc += 1;

                Ok(true) 
            }

            Opcode::Div => {
                let b = self.stack.pop().ok_or("Stack underflow".to_string())?;
                let a = self.stack.pop().ok_or("Stack underflow".to_string())?;
                if b == 0 {
                    return Err("Division by zero".to_string());
                }
                self.stack.push(a / b);
                self.pc += 1;

                Ok(true)
            }

            Opcode::Store => {
                let value = self.stack.pop().ok_or("Stack underflow".to_string())?;
                let address = self.stack.pop().ok_or("Stack underflow".to_string())?;
                if address < 0 {
                    return Err("Negative memory address".to_string());
                }
                self.memory.store(address as usize, value);
                self.pc += 1;

                Ok(true)
            }

            Opcode::Load => {
                let address = self.stack.pop().ok_or("Stack underflow".to_string())?;
                if address < 0 {
                    return Err("Negative memory address".to_string());
                }
                let value = self.memory.load(address as usize);
                self.stack.push(value);
                self.pc += 1;

                Ok(true)
            }

            Opcode::Jump => {
                let address = self.stack.pop().ok_or("Stack underflow".to_string())?;
                if address < 0 {
                    return Err("Invalid jump address".to_string());
                }
                let address = address as usize;
                if address >= self.bytecode.len() {
                    return Err("Jump address out of bounds".to_string());
                }
                self.pc = address;

                Ok(true)
            }

            Opcode::Jumpi => {
                let address = self.stack.pop().ok_or("Stack underflow".to_string())?;
                let condition = self.stack.pop().ok_or("Stack underflow".to_string())?;
                if condition != 0 {
                    if address < 0 {
                        return Err("Invalid jump address".to_string());
                    }
                    let address = address as usize;
                    if address >= self.bytecode.len() {
                        return Err("Jump address out of bounds".to_string());
                    }
                    self.pc = address;
                } else {
                    self.pc += 1;
                }
                Ok(true)
            }

            Opcode::Return => {
                let value = self.stack.pop().ok_or("Stack underflow".to_string())?;
                self.return_value = Some(value);
                Ok(false)
            }

            _ => {
                Err(format!("Unhandled opcode: {:?}", opcode))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_new() {
        let bytecode = vec![0x00];

        let vm = VM::new(bytecode);
        assert_eq!(vm.pc, 0);
        assert_eq!(vm.bytecode, vec![0x00]);
        assert_eq!(vm.stack.len(), 0);
    }

    #[test]
    fn test_vm_stop() {
        let bytecode = vec![0x00];

        let mut vm = VM::new(bytecode);

        let result = vm.step();

        assert_eq!(result, Ok(false));
        assert_eq!(vm.pc, 0);
    }

    #[test]
    fn test_vm_add() {
        let bytecode = vec![0x01, 0x0A, 0x01, 0x14, 0x02]; // Push 10, Push 20, Add
    
        let mut vm = VM::new(bytecode);

        let result = vm.step(); // Push 10
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Push 20
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Add
        assert_eq!(result, Ok(true));

        assert_eq!(vm.stack.pop(), Some(30));
        assert_eq!(vm.pc,5);
    }

    #[test]
    fn test_vm_sub() {
        let bytecode = vec![0x01, 0x14, 0x01, 0x0A, 0x03]; // Push 20, Push 10, Sub
    
        let mut vm = VM::new(bytecode);

        let result = vm.step(); // Push 20
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Push 10
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Sub
        assert_eq!(result, Ok(true));

        assert_eq!(vm.stack.pop(), Some(10));
        assert_eq!(vm.pc,5);
    
    }

    #[test]
    fn test_vm_mul() {
        let bytecode = vec![0x01, 0x05, 0x01, 0x06, 0x04]; // Push 5, Push 6, Mul

        let mut vm = VM::new(bytecode);

        let result = vm.step(); // Push 5
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Push 6
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Mul
        assert_eq!(result, Ok(true));

        assert_eq!(vm.stack.pop(), Some(30));
        assert_eq!(vm.pc,5);
    }

    #[test]
    fn test_vm_div() {
        let bytecode = vec![0x01, 0x14, 0x01, 0x05, 0x05]; // Push 20, Push 5, Div

        let mut vm = VM::new(bytecode);

        let result = vm.step(); // Push 20
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Push 5
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Div
        assert_eq!(result, Ok(true));

        assert_eq!(vm.stack.pop(), Some(4));
        assert_eq!(vm.pc,5);
    }

    #[test]
    fn test_vm_run() {
        let bytecode = vec![0x01, 0x0A, 0x01, 0x14, 0x02, 0x00]; // Push 10, Push 20, Add, Stop

        let mut vm = VM::new(bytecode);

        let result = vm.run();
        assert_eq!(result, Ok(()));
        assert_eq!(vm.stack.pop(), Some(30));
        assert_eq!(vm.pc, 5);
    }

    #[test]
    fn test_vm_store_load() {
        let bytecode = vec![0x01, 0x00, 0x01, 0x64, 0x07,0x01,0x00,0x08,0x00]; // Push 0, Push 100, Store, Push 0, Load, Stop 

        let mut vm = VM::new(bytecode);

        let result = vm.run();
        assert_eq!(result, Ok(()));
        assert_eq!(vm.stack.pop(), Some(100));
        assert_eq!(vm.memory.load(0), 100);
        assert_eq!(vm.pc, 9);
    }

    #[test]
    fn test_vm_return() {
        let bytecode = vec![0x01, 0x2A, 0x0B]; // Push 42, Return
        
        let mut vm = VM::new(bytecode);

        let result = vm.run();
        assert_eq!(result, Ok(()));
        assert_eq!(vm.return_value, Some(42));
        assert_eq!(vm.pc, 1);
    }

    #[test]
    fn test_vm_jump() {
        let bytecode = vec![0x01, 0x06,0x09,0x01,0xFF,0x00,0x01,0x2A, 0x0B];

        let mut vm = VM::new(bytecode);

        assert_eq!(vm.run(), Ok(()));
        assert_eq!(vm.return_value, Some(42));
    }
}