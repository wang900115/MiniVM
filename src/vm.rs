use crate::stack::Stack;
use crate::memory::Memory;
use crate::opcode::Opcode;
use crate::gas::Gas;
use crate::register::Register;
use crate::call_frame::CallFrame;
use crate::host::Host;

#[derive(Debug, PartialEq, Eq)]
pub enum VMError {
    StackUnderflow,
    DivisionByZero,
    InvalidOpcode(u8),
    ProgramCounterOutOfBounds,
    InvalidJumpAddress,
    NegativeMemoryAddress,
    InvalidMemoryAddress,
    PushOperandMissing,
    OutOfGas,
}

pub struct VM<H: Host> {
    pub stack: Stack,
    pub memory: Memory,
    
    pub bytecode: Vec<u8>,
    pub return_value: Option<i32>,
    pub register: Register,
    pub gas: Gas,        // if vm try execute an opcode, it will consume gas first
    pub call_stack: Vec<CallFrame>,

    pub host: H,
}

impl<H: Host> VM<H> {
    pub fn new(bytecode: Vec<u8>, gas_limit: u64, host: H) -> Self {
        Self {
            stack: Stack::new(),
            memory: Memory::new(),
            register: Register::new(),
            bytecode,
            return_value: None,
            gas: Gas::new(gas_limit),
            call_stack: Vec::new(),
            host,
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

        if self.register.pc >= self.bytecode.len() {
            return Err(VMError::ProgramCounterOutOfBounds);
        }

        let byte = self.bytecode[self.register.pc];

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
                if self.register.pc + 1 >= self.bytecode.len() {
                    return Err(VMError::PushOperandMissing);
                }
                let operand = self.bytecode[self.register.pc + 1] as i32;
                self.push_stack(operand);
                self.register.pc += 2; 

                Ok(true)
            }

            Opcode::Pop => {
                self.pop_stack()?;
                self.register.pc += 1;

                Ok(true)
            }

            Opcode::Add => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.push_stack(a + b);
                self.register.pc += 1;

                Ok(true)
            }

            Opcode::Sub => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.push_stack(a - b);
                self.register.pc += 1;

                Ok(true)
            } 

            Opcode::Mul => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.push_stack(a * b);
                self.register.pc += 1;

                Ok(true) 
            }

            Opcode::Div => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                self.push_stack(a / b);
                self.register.pc += 1;

                Ok(true)
            }

            Opcode::Store => {
                let value = self.pop_stack()?;
                let address = self.pop_stack()?;
                if address < 0 {
                    return Err(VMError::NegativeMemoryAddress);
                }
                self.memory.store(address as usize, value);
                self.register.pc += 1;

                Ok(true)
            }

            Opcode::Load => {
                let address = self.pop_stack()?;
                if address < 0 {
                    return Err(VMError::NegativeMemoryAddress);
                }
                let value = self.memory.load(address as usize);
                self.push_stack(value);
                self.register.pc += 1;

                Ok(true)
            }

            Opcode::Jump => {
                let address = self.pop_stack()?;
                if address < 0 {
                    return Err(VMError::InvalidJumpAddress);
                }
                let address = address as usize;
                if !self.is_valid_jump_destination(address) {
                    return Err(VMError::InvalidJumpAddress);
                }
                self.register.pc = address;

                Ok(true)
            }

            Opcode::Jumpi => {
                let address = self.pop_stack()?;
                let condition = self.pop_stack()?;
                if condition != 0 {
                    if address < 0 {
                        return Err(VMError::InvalidJumpAddress);
                    }
                    let address = address as usize;
                    if !self.is_valid_jump_destination(address as usize) {
                        return Err(VMError::InvalidJumpAddress);
                    }
                    self.register.pc = address;
                } else {
                    self.register.pc += 1;
                }

                Ok(true)
            }

            Opcode::JumpDest => {
                // JumpDest is a marker and does not perform any action
                self.register.pc += 1;

                Ok(true)
            }

            Opcode::Call => {
                let address = self.pop_stack()?;
                if address < 0 {
                    return Err(VMError::InvalidJumpAddress);
                }
                let address = address as usize;
                if !self.is_valid_jump_destination(address) {
                    return Err(VMError::InvalidJumpAddress);
                }
                self.call_stack.push(CallFrame {
                    return_pc: self.register.pc + 1,
                    stack_base: self.stack.len(),
                    previous_fp: self.register.fp,
                });

                self.register.fp = self.register.sp;
                self.register.pc = address;

                Ok(true)
            }

            Opcode::Return => {
                let value = self.pop_stack()?;

                if let Some(frame) = self.call_stack.pop() {

                    while self.stack.len() > frame.stack_base {
                        self.pop_stack()?;
                    }

                    self.push_stack(value);

                    self.register.pc = frame.return_pc;
                    self.register.fp = frame.previous_fp;

                    Ok(true)
                } else {
                    self.return_value = Some(value);
                    
                    Ok(false)
                }
            }

            Opcode::LoadLocal => {
                if self.register.pc + 1 >= self.bytecode.len() {
                    return Err(VMError::PushOperandMissing);
                }

                let offset = self.bytecode[self.register.pc + 1] as usize;

                let index = self.register.fp + offset;
                
                let value = self.stack.get(index).ok_or(VMError::InvalidMemoryAddress)?;
                self.push_stack(value);
                self.register.pc += 2;

                Ok(true)
            }

            Opcode::StoreLocal => {
                if self.register.pc + 1 >= self.bytecode.len() {
                    return Err(VMError::PushOperandMissing);
                }

                let offset = self.bytecode[self.register.pc + 1] as usize;

                let value = self.pop_stack()?;

                let index = self.register.fp + offset;

                self.stack.set(index,value).map_err(|_| VMError::InvalidMemoryAddress)?;
                self.register.pc += 2;

                Ok(true)
            }

            Opcode::StorageLoad => {
                let key = self.pop_stack()?;
                let value = self.host.storage_load(key);
                self.push_stack(value);
                self.register.pc += 1;

                Ok(true)
            }

            Opcode::StorageStore => {
                let value = self.pop_stack()?;
                let key = self.pop_stack()?;
                self.host.storage_store(key, value);
                self.register.pc += 1;

                Ok(true)
            }

            Opcode::Emit => {
                let value = self.pop_stack()?;
                self.host.emit(value);
                self.register.pc += 1;

                Ok(true)
            }
        }
    }

    pub fn is_valid_jump_destination(&self, address: usize) -> bool {
        if address >= self.bytecode.len() {
            return false;
        }
        matches!(Opcode::try_from(self.bytecode[address]), Ok(Opcode::JumpDest))
    }

    fn push_stack(&mut self, value: i32) {
        self.stack.push(value);
        self.register.sp = self.stack.len();
    }

    fn pop_stack(&mut self) -> Result<i32, VMError> {
        let value = self.stack.pop().ok_or(VMError::StackUnderflow)?;
        self.register.sp = self.stack.len();

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::SimpleHost;


    fn create_vm(bytecode: Vec<u8>) -> VM<SimpleHost> {
        let host = SimpleHost::new();
        VM::new(bytecode, 100, host)
    }

    #[test]
    fn test_vm_new() {
        let bytecode = vec![0x00];
        let vm = create_vm(bytecode);
        assert_eq!(vm.register.pc, 0);
        assert_eq!(vm.bytecode, vec![0x00]);
        assert_eq!(vm.stack.len(), 0);
        assert_eq!(vm.gas.remaining(), 100);
    }

    #[test]
    fn test_vm_stop() {
        let bytecode = vec![0x00];
        let mut vm = create_vm(bytecode);
        let result = vm.step();
        assert_eq!(result, Ok(false));
        assert_eq!(vm.register.pc, 0);
        assert_eq!(vm.gas.remaining(), 100);
    }
    
    #[test]
    fn test_vm_push() {
        let bytecode = vec![0x01, 0x0A]; // PUSH 10
        let mut vm = create_vm(bytecode);
        let result = vm.step(); // PUSH 10
        assert_eq!(result, Ok(true));
        assert_eq!(vm.stack.pop(), Some(10));
        assert_eq!(vm.register.pc, 2);
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

        let mut vm = create_vm(bytecode);
        
        assert_eq!(vm.run(), Ok(()));
        assert_eq!(vm.stack.pop(), Some(10));
        assert_eq!(vm.register.pc, 5);
        assert_eq!(vm.gas.remaining(), 97);
    }

    #[test]
    fn test_vm_add() {
        let bytecode = vec![0x01, 0x0A, 0x01, 0x14, 0x02]; // Push 10, Push 20, Add
    
        let mut vm = create_vm(bytecode);

        let result = vm.step(); // Push 10
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Push 20
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Add
        assert_eq!(result, Ok(true));

        assert_eq!(vm.stack.pop(), Some(30));
        assert_eq!(vm.register.pc,5);
        assert_eq!(vm.gas.remaining(), 97);
    }

    #[test]
    fn test_vm_sub() {
        let bytecode = vec![0x01, 0x14, 0x01, 0x0A, 0x03]; // Push 20, Push 10, Sub
    
        let mut vm = create_vm(bytecode);

        let result = vm.step(); // Push 20
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Push 10
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Sub
        assert_eq!(result, Ok(true));

        assert_eq!(vm.stack.pop(), Some(10));
        assert_eq!(vm.register.pc,5);
        assert_eq!(vm.gas.remaining(), 97);
    }

    #[test]
    fn test_vm_mul() {
        let bytecode = vec![0x01, 0x05, 0x01, 0x06, 0x04]; // Push 5, Push 6, Mul

        let mut vm = create_vm(bytecode);

        let result = vm.step(); // Push 5
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Push 6
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Mul
        assert_eq!(result, Ok(true));

        assert_eq!(vm.stack.pop(), Some(30));
        assert_eq!(vm.register.pc,5);
        assert_eq!(vm.gas.remaining(), 97);
    }

    #[test]
    fn test_vm_div() {
        let bytecode = vec![0x01, 0x14, 0x01, 0x05, 0x05]; // Push 20, Push 5, Div

        let mut vm = create_vm(bytecode);

        let result = vm.step(); // Push 20
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Push 5
        assert_eq!(result, Ok(true));
        let result = vm.step(); // Div
        assert_eq!(result, Ok(true));

        assert_eq!(vm.stack.pop(), Some(4));
        assert_eq!(vm.register.pc,5);
        assert_eq!(vm.gas.remaining(), 97);
    }

    #[test]
    fn test_vm_run() {
        let bytecode = vec![
            0x01, 0x0A, // Push 10 2
            0x01, 0x14, // Push 20 4
            0x02,       // Add     5
            0x00];      // Stop
        
        let mut vm = create_vm(bytecode);

        let result = vm.run();
        assert_eq!(result, Ok(()));
        assert_eq!(vm.stack.pop(), Some(30));
        assert_eq!(vm.register.pc, 5);
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

        let mut vm = create_vm(bytecode);

        let result = vm.run();
        assert_eq!(result, Ok(()));
        assert_eq!(vm.stack.pop(), Some(100));
        assert_eq!(vm.memory.load(0), 100);
        assert_eq!(vm.register.pc, 8);
        assert_eq!(vm.gas.remaining(), 89);
    }

    #[test]
    fn test_vm_return() {
        let bytecode = vec![
            0x01, 0x2A, // 0: Push 42   
            0x0B];      // 2: Return
        
        let mut vm = create_vm(bytecode);

        let result = vm.run();
        assert_eq!(result, Ok(()));
        assert_eq!(vm.return_value, Some(42));
        assert_eq!(vm.register.pc, 2);
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

        let mut vm = create_vm(bytecode);

        assert_eq!(vm.run(), Ok(()));
        assert_eq!(vm.return_value, Some(42));
        assert_eq!(vm.register.pc, 9);
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

        let mut vm = create_vm(bytecode);

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

        let mut vm = create_vm(bytecode);

        assert_eq!(vm.run(), Ok(()));
        assert_eq!(vm.return_value, Some(42));
        assert_eq!(vm.register.pc, 9);
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

        let mut vm = create_vm(bytecode);

        assert_eq!(vm.run(), Ok(()));
        assert_eq!(vm.return_value, Some(42));
        assert_eq!(vm.register.pc, 7);
        assert_eq!(vm.gas.remaining(), 94);
    }

    #[test]
    fn test_vm_load_local() {
        let bytecode = vec![
            0x01, 0x2A, // 0: PUSH 42                      [42]
            0x0E, 0x00, // 2: LOAD_LOCAL 0                 [42, 42] 
            0x00,       // 4: STOP                            
        ];

        let mut vm = create_vm(bytecode);


        vm.step().unwrap();


        let result = vm.step();

        assert_eq!(result, Ok(true));
        assert_eq!(vm.stack.pop(), Some(42));
        assert_eq!(vm.stack.pop(), Some(42));
        assert_eq!(vm.register.pc, 4);
    }

    #[test]
    fn test_vm_store_local() {
        let bytecode = vec![
        0x01, 0x00, // 0: PUSH 0      -> local slot         [0]
        0x01, 0x2A, // 2: PUSH 42                           [0, 42]
        0x0F, 0x00, // 4: STORE_LOCAL 0                     [42]
        0x0E, 0x00, // 6: LOAD_LOCAL 0                      [42, 42]
        0x00,       // 8: STOP
        ];

        let mut vm = create_vm(bytecode);

        let result = vm.run();

        assert_eq!(result, Ok(()));
        assert_eq!(vm.stack.pop(), Some(42));
        assert_eq!(vm.stack.pop(), Some(42));
        assert_eq!(vm.register.pc, 8);
    }

    #[test]
    fn test_vm_multiple_locals() {
        let bytecode = vec![
            0x01, 0x00, // 0: PUSH 0       -> local 0        [0]
            0x01, 0x00, // 2: PUSH 0       -> local 1        [0, 0]

            0x01, 0x0A, // 4: PUSH 10                        [0, 0, 10]
            0x0F, 0x00, // 6: STORE_LOCAL 0                  [10, 0]

            0x01, 0x14, // 8: PUSH 20                        [10, 0, 20]
            0x0F, 0x01, // 10: STORE_LOCAL 1                 [10, 20]

            0x0E, 0x00, // 12: LOAD_LOCAL 0                  [10, 20, 10]
            0x0E, 0x01, // 14: LOAD_LOCAL 1                  [10, 20, 10, 20]

            0x00,       // 16: STOP
        ];

        let mut vm = create_vm(bytecode);

        let result = vm.run();

        assert_eq!(result, Ok(()));

        assert_eq!(vm.stack.pop(), Some(20));
        assert_eq!(vm.stack.pop(), Some(10));
        assert_eq!(vm.stack.pop(), Some(20));
        assert_eq!(vm.stack.pop(), Some(10));
        assert_eq!(vm.register.pc, 16);
    }

    #[test]
    fn test_vm_call_frame() {
        let bytecode = vec![
            0x01, 0x04, // 0: PUSH 4                         1. [4]
            0x0D,       // 2: CALL                           2. []                   callframe: [3, 0, 0]
            0x0B,       // 3: RETURN                         7. [42]

            // function
            0x0C,       // 4: JUMPDEST
            0x01, 0x00, // 5: PUSH 0       -> local slot     3. [0]                 
            0x01, 0x2A, // 7: PUSH 42                        4. [0, 42]
            0x0F, 0x00, // 9: STORE_LOCAL 0                  5. [42]
            0x0E, 0x00, // 11: LOAD_LOCAL 0                  6. [42, 42]
            0x0B,       // 13: RETURN                         
        ];

        let mut vm = create_vm(bytecode);

        let result = vm.run();

        assert_eq!(result, Ok(()));
        assert_eq!(vm.return_value, Some(42));
        assert_eq!(vm.register.pc, 3);
        assert_eq!(vm.register.fp, 0);
        assert_eq!(vm.register.sp, 0);
        assert!(vm.call_stack.is_empty());
    }

    #[test]
    fn test_vm_nested_call() {
        let bytecode = vec![
            // main
            0x01, 0x04, // 0: PUSH 4       1. [4]
            0x0D,       // 2: CALL         2. []       callframe: [3, 0, 0]
            0x0B,       // 3: RETURN

            // function A
            0x0C,       // 4: JUMPDEST     3. []
            0x01, 0x09, // 5: PUSH 9       4. [9]
            0x0D,       // 7: CALL         5. []       callframe: [3, 0, 0] [8, 0, 0]
            0x0B,       // 8: RETURN

            // function B
            0x0C,       // 9: JUMPDEST     6. []
            0x01, 0x14, // 10: PUSH 20     7. [20]
            0x0B,       // 12: RETURN      8. [20]      
        ];

        let mut vm = create_vm(bytecode);

        let result = vm.run();

        assert_eq!(result, Ok(()));
        assert_eq!(vm.return_value, Some(20));
        assert_eq!(vm.register.pc, 3);
        assert_eq!(vm.register.fp, 0);
        assert_eq!(vm.register.sp, 0);
        assert!(vm.call_stack.is_empty());
    }

    #[test]
    fn test_vm_nested_call_local() {
        let bytecode = vec![

            0x01, 0x04, // 0: PUSH 4
            0x0D,       // 2: CALL
            0x0B,       // 3: RETURN


            // function A
            0x0C,       // 4: JUMPDEST

            0x01, 0x00, // 5: PUSH 0       
            0x01, 0x0A, // 7: PUSH 10
            0x0F, 0x00, // 9: STORE_LOCAL 0 

            0x01, 0x12, // 11: PUSH 18    
            0x0D,       // 13: CALL

            0x06,       // 14: POP         

            0x0E, 0x00, // 15: LOAD_LOCAL 0 
            0x0B,       // 17: RETURN


            // function B
            0x0C,       // 18: JUMPDEST

            0x01, 0x00, // 19: PUSH 0      
            0x01, 0x14, // 21: PUSH 20
            0x0F, 0x00, // 23: STORE_LOCAL 0 

            0x0E, 0x00, // 25: LOAD_LOCAL 0
            0x0B,       // 27: RETURN
        ];

        let mut vm = create_vm(bytecode);

        let result = vm.run();

        assert_eq!(result, Ok(()));
        assert_eq!(vm.return_value, Some(10));
        assert_eq!(vm.register.pc, 3);
        assert_eq!(vm.register.fp, 0);
        assert_eq!(vm.register.sp, 0);
        assert!(vm.call_stack.is_empty());
    }

    #[test]
    fn test_vm_storage() {
        let bytecode = vec![
            0x01, 0x01, // PUSH 1
            0x01, 0x2A, // PUSH 42
            0x11,       // STORAGE_STORE

            0x01, 0x01, // PUSH 1
            0x10,       // STORAGE_LOAD

            0x00,       // STOP
        ];

        let mut vm = create_vm(bytecode);

        assert_eq!(vm.run(), Ok(()));

        assert_eq!(vm.stack.pop(), Some(42));
    }

    #[test]
    fn test_vm_emit() {
        let bytecode = vec![
            0x01, 0x2A, // PUSH 42
            0x12,       // EMIT
            0x00,       // STOP
        ];

        let mut vm = create_vm(bytecode);

        assert_eq!(vm.run(), Ok(()));
        assert_eq!(vm.host.events, vec![42]);
    }
}