pub trait ContractInterface {
    fn bytecode(&self) -> &[u8];
}

pub struct Contract {
    pub bytecode: Vec<u8>,
}

impl Contract {
    pub fn new(bytecode: Vec<u8>) -> Self {
        Self { bytecode }
    }
}

impl ContractInterface for Contract {
    fn bytecode(&self) -> &[u8] {
        &self.bytecode
    }
}