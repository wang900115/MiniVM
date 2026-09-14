#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbiError {
    FunctionNotFound,
    InvalidArgumentCount,
}




#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiCall {
    pub function: String,
    pub args: Vec<i32>,
}

impl AbiCall {
    pub fn new(function: &str, args: Vec<i32>) -> Self {
        Self {
            function: function.to_string(),
            args,
        }
    }

    pub fn validate(&self, function: &AbiFunction) -> Result<(), AbiError> {
        if self.args.len() != function.arg_count {
            return Err(AbiError::InvalidArgumentCount);
        }
        Ok(())
    }

    pub fn encode(&self, function: &AbiFunction) -> Vec<i32> {
        let mut values = self.args.clone();
        values.push(function.address as i32);
        values.push(function.arg_count as i32);
        values
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiFunction {
    pub name: String,
    pub address: usize,
    pub arg_count: usize,
}

impl AbiFunction {
    pub fn new(name: &str, address: usize, arg_count: usize) -> Self {
        Self {
            name: name.to_string(),
            address,
            arg_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractAbi {
    pub functions: Vec<AbiFunction>,
}

impl ContractAbi {
    pub fn new() -> Self {
        Self { functions: Vec::new() }
    }

    pub fn add_function(&mut self, function: AbiFunction) {
        self.functions.push(function);
    }

    pub fn find_function(&self, name: &str) -> Option<&AbiFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    pub fn encode_call(&self, call: &AbiCall) -> Result<Vec<i32>, AbiError> {
        let function = self
            .find_function(&call.function)
            .ok_or(AbiError::FunctionNotFound)?;

        call.validate(function)?;

        Ok(call.encode(function))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_call_encode() {
        let call = AbiCall {
            function: "add".to_string(),
            args: vec![10, 20],
        };
        let function = AbiFunction {
            name: "add".to_string(),
            address: 10,
            arg_count: 2,
        };
        assert!(call.validate(&function).is_ok());
        let encoded = call.encode(&function);
        assert_eq!(encoded, vec![10, 20, 10, 2]);
    }

    #[test]
    fn test_contract_abi_encode_call() {
        let mut abi = ContractAbi::new();
        abi.add_function(AbiFunction::new("add", 10, 2));
        abi.add_function(AbiFunction::new("sub", 20, 2));
        let call = AbiCall::new("add", vec![10, 20]);
        let encoded = abi.encode_call(&call).unwrap();

        assert_eq!(encoded, vec![10, 20, 10, 2]);
    }
}