use crate::stack::Stack;
use crate::memory::Memory;
use crate::opcode::Opcode;
use crate::gas::Gas;

#[derive(Debug, PartialEq, Eq)]
pub enum VMError {
    StackUnderflow,
    DivisionByZero,
    InvalidOpcode(u8),
    ProgramCounterOutOfBounds,
    InvalidJumpAddress,
    NegativeMemoryAddress,
    PushOperandMissing,
    OutOfGas,
}

pub struct VM {
    pub stack: Stack,
    pub memory: Memory,
    pub pc: usize,
    pub bytecode: Vec<u8>,
    pub return_value: Option<i32>,
    
    pub gas: Gas,        // if vm try execute an opcode, it will consume gas first
}


impl VM {
    pub fn new(bytecode: Vec<u8>, gas_limit: u64) -> Self {
        Self {
            stack: Stack::new(),
            memory: Memory::new(),
            pc: 0,
            bytecode,
            return_value: None,
            gas: Gas::new(gas_limit),
        }
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        loop {
            let should_continue = self.step()?;

            if !should_continue {
                break;
            }
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<bool, VMError> {

        if self.pc >= self.bytecode.len() {
            return Err(VMError::ProgramCounterOutOfBounds);
        }

        let byte = self.bytecode[self.pc];

        let opcode = Opcode::try_from(byte).map_err(|_| VMError::InvalidOpcode(byte))?;

        let gas_cost = opcode.gas_cost();

        if !self.gas.consume(gas_cost) {
            return Err(VMError::OutOfGas);
        }

        match opcode {
            Opcode::Stop => {
                Ok(false)
            }

            Opcode::Push => {
                if self.pc + 1 >= self.bytecode.len() {
                    return Err(VMError::PushOperandMissing);
                }
                let operand = self.bytecode[self.pc + 1] as i32;
                self.stack.push(operand);
                self.pc += 2; 

                Ok(true)
            }

            Opcode::Pop => {
                self.stack.pop().ok_or(VMError::StackUnderflow)?;
                self.pc += 1;

                Ok(true)
            }

            Opcode::Add => {
                let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                self.stack.push(a + b);
                self.pc += 1;

                Ok(true)
            }

            Opcode::Sub => {
                let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                self.stack.push(a - b);
                self.pc += 1;

                Ok(true)
            } 

            Opcode::Mul => {
                let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                self.stack.push(a * b);
                self.pc += 1;

                Ok(true) 
            }

            Opcode::Div => {
                let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                self.stack.push(a / b);
                self.pc += 1;

                Ok(true)
            }

            Opcode::Store => {
                let value = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let address = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                if address < 0 {
                    return Err(VMError::NegativeMemoryAddress);
                }
                self.memory.store(address as usize, value);
                self.pc += 1;

                Ok(true)
            }

            Opcode::Load => {
                let address = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                if address < 0 {
                    return Err(VMError::NegativeMemoryAddress);
                }
                let value = self.memory.load(address as usize);
                self.stack.push(value);
                self.pc += 1;

                Ok(true)
            }

            Opcode::Jump => {
                let address = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                if address < 0 {
                    return Err(VMError::InvalidJumpAddress);
                }
                let address = address as usize;
             
                if !self.is_valid_jump_destination(address) {
                    return Err(VMError::InvalidJumpAddress);
                }

                self.pc = address;

                Ok(true)
            }

            Opcode::Jumpi => {
                let address = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let condition = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                if condition != 0 {
                    if address < 0 {
                        return Err(VMError::InvalidJumpAddress);
                    }
                    let address = address as usize;
                    if !self.is_valid_jump_destination(address as usize) {
                        return Err(VMError::InvalidJumpAddress);
                    }
                    self.pc = address;
                } else {
                    self.pc += 1;
                }
                Ok(true)
            }

            Opcode::JumpDest => {
                // JumpDest is a marker and does not perform any action
                self.pc += 1;

                Ok(true)
            }

            Opcode::Return => {
                let value = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                self.return_value = Some(value);
                Ok(false)
            }
        }
    }

    pub fn is_valid_jump_destination(&self, address: usize) -> bool {
        if address >= self.bytecode.len() {
            return false;
        }
        matches!(Opcode::try_from(self.bytecode[address]), Ok(Opcode::JumpDest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_new() {
        let bytecode = vec![0x00];

        let vm = VM::new(bytecode,100);
        assert_eq!(vm.pc, 0);
        assert_eq!(vm.bytecode, vec![0x00]);
        assert_eq!(vm.stack.len(), 0);
        assert_eq!(vm.gas.remaining(), 100);
    }

    #[test]
    fn test_vm_stop() {
        let bytecode = vec![0x00];

        let mut vm = VM::new(bytecode, 100);

        let result = vm.step();

        assert_eq!(result, Ok(false));
        assert_eq!(vm.pc, 0);
        assert_eq!(vm.gas.remaining(), 100);
    }
    
    #[test]
    fn test_vm_push() {
        let bytecode = vec![0x01, 0x0A]; // PUSH 10
        let mut vm = VM::new(bytecode, 100);
        
        let result = vm.step(); // PUSH 10
        assert_eq!(result, Ok(true));
        assert_eq!(vm.stack.pop(), Some(10));
        assert_eq!(vm.pc, 2);
        assert_eq!(vm.gas.remaining(), 99);
    }

    #[test]
    fn test_vm_pop() {
        let bytecode = vec![
            0x01, 0x0A, // 0: PUSH 10
            0x01, 0x14, // 2: PUSH 20
            0x06,       // 4: POP
            0x00,       // 5: STOP
        ];

        let mut vm = VM::new(bytecode, 100);

        assert_eq!(vm.run(), Ok(()));
        assert_eq!(vm.stack.pop(), Some(10));
        assert_eq!(vm.pc, 5);
        assert_eq!(vm.gas.remaining(), 97);
    }

    #[test]
    fn test_vm_add() {
        let bytecode = vec![0x01, 0x0A, 0x01, 0x14, 0x02]; // Push 10, Push 20, Add
    
        let mut vm = VM::new(bytecode, 100);

        let result = vm.step(); // Push 10
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Push 20
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Add
        assert_eq!(result, Ok(true));

        assert_eq!(vm.stack.pop(), Some(30));
        assert_eq!(vm.pc,5);
        assert_eq!(vm.gas.remaining(), 97);
    }

    #[test]
    fn test_vm_sub() {
        let bytecode = vec![0x01, 0x14, 0x01, 0x0A, 0x03]; // Push 20, Push 10, Sub
    
        let mut vm = VM::new(bytecode, 100);

        let result = vm.step(); // Push 20
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Push 10
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Sub
        assert_eq!(result, Ok(true));

        assert_eq!(vm.stack.pop(), Some(10));
        assert_eq!(vm.pc,5);
        assert_eq!(vm.gas.remaining(), 97);
    }

    #[test]
    fn test_vm_mul() {
        let bytecode = vec![0x01, 0x05, 0x01, 0x06, 0x04]; // Push 5, Push 6, Mul

        let mut vm = VM::new(bytecode, 100);

        let result = vm.step(); // Push 5
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Push 6
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Mul
        assert_eq!(result, Ok(true));

        assert_eq!(vm.stack.pop(), Some(30));
        assert_eq!(vm.pc,5);
        assert_eq!(vm.gas.remaining(), 97);
    }

    #[test]
    fn test_vm_div() {
        let bytecode = vec![0x01, 0x14, 0x01, 0x05, 0x05]; // Push 20, Push 5, Div

        let mut vm = VM::new(bytecode, 100);

        let result = vm.step(); // Push 20
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Push 5
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Div
        assert_eq!(result, Ok(true));

        assert_eq!(vm.stack.pop(), Some(4));
        assert_eq!(vm.pc,5);
        assert_eq!(vm.gas.remaining(), 97);
    }

    #[test]
    fn test_vm_run() {
        let bytecode = vec![
            0x01, 0x0A, // Push 10 2
            0x01, 0x14, // Push 20 4
            0x02,       // Add     5
            0x00];      // Stop

        let mut vm = VM::new(bytecode, 100);

        let result = vm.run();
        assert_eq!(result, Ok(()));
        assert_eq!(vm.stack.pop(), Some(30));
        assert_eq!(vm.pc, 5);
        assert_eq!(vm.gas.remaining(), 97);
    }

    #[test]
    fn test_vm_store_load() {
        let bytecode = vec![
            0x01, 0x00, // 0: Push 0     
            0x01, 0x64, // 2: Push 100    
            0x07,       // 4: Store       
            0x01, 0x00, // 5: Push 0      
            0x08,       // 7: Load        
            0x00];      // 8: Stop 

        let mut vm = VM::new(bytecode, 100);

        let result = vm.run();
        assert_eq!(result, Ok(()));
        assert_eq!(vm.stack.pop(), Some(100));
        assert_eq!(vm.memory.load(0), 100);
        assert_eq!(vm.pc, 8);
        assert_eq!(vm.gas.remaining(), 89);
    }

    #[test]
    fn test_vm_return() {
        let bytecode = vec![
            0x01, 0x2A, // 0: Push 42   
            0x0B];      // 2: Return
        
        let mut vm = VM::new(bytecode, 100);

        let result = vm.run();
        assert_eq!(result, Ok(()));
        assert_eq!(vm.return_value, Some(42));
        assert_eq!(vm.pc, 2);
        assert_eq!(vm.gas.remaining(), 99);
    }

    #[test]
    fn test_vm_jump_is_valid_destination() {
        let bytecode = vec![
            0x01, 0x06,       // 0: Push 6     
            0x09,             // 2: Jump       
            0x01,0xFF,        // 3: Push 255   (skip)
            0x00,             // 5: Stop       (skip)
            0x0C,             // 6: JumpDest
            0x01,0x2A,        // 7: Push 42    
            0x0B];            // 9: Return

        let mut vm = VM::new(bytecode, 100);

        assert_eq!(vm.run(), Ok(()));
        assert_eq!(vm.return_value, Some(42));
        assert_eq!(vm.pc, 9);
        assert_eq!(vm.gas.remaining(),95);
    }

    #[test]
    fn test_vm_jump_invalid_destination() {
        let bytecode = vec![
            0x01, 0x05, // 0: PUSH 5
            0x09,       // 2: JUMP
            0x0C,       // 4: JumpDest
            0x00,       // 3: STOP
        ];

        let mut vm = VM::new(bytecode, 100);

        assert!(vm.run().is_err());
    }

    #[test]
    fn test_vm_jumpi_true() {
        let bytecode = vec![
            0x01, 0x01, // 0: PUSH 1
            0x01, 0x06, // 2: PUSH 6
            0x0A,       // 4: JUMPI
            0x00,       // 5: STOP
            0x0C,       // 6: JumpDest
            0x01, 0x2A, // 7: PUSH 42
            0x0B,       // 9: RETURN
        ];

        let mut vm = VM::new(bytecode, 100);

        assert_eq!(vm.run(), Ok(()));
        assert_eq!(vm.return_value, Some(42));
        assert_eq!(vm.pc, 9);
        assert_eq!(vm.gas.remaining(), 93);
    }

    #[test]
    fn test_vm_jumpi_false() {
        let bytecode = vec![
            0x01, 0x00, // 0: PUSH 0
            0x01, 0x06, // 2: PUSH 6
            0x0A,       // 4: JUMPI
            0x01, 0x2A, // 5: PUSH 42
            0x0B,       // 7: RETURN
            0x00,       // 8: STOP
        ];

        let mut vm = VM::new(bytecode, 100);

        assert_eq!(vm.run(), Ok(()));
        assert_eq!(vm.return_value, Some(42));
        assert_eq!(vm.pc, 7);
        assert_eq!(vm.gas.remaining(), 94);
    }
}