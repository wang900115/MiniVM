#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiCall {
    pub function: String,
    pub args: Vec<i32>,
}

impl AbiCall {
    pub fn validate(&self, function: &AbiFunction) -> bool {
        self.function == function.name && self.args.len() == function.arg_count
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractAbi {
    pub functions: Vec<AbiFunction>,
}

impl ContractAbi {
    pub fn find_function(&self, name: &str) -> Option<&AbiFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    pub fn encode_call(&self, call: &AbiCall) -> Option<Vec<i32>> {
        let function = self.find_function(&call.function)?;
        
        if !call.validate(function) {
            return None;
        }
        Some(call.encode(function))
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

        assert!(call.validate(&function));

        let encoded = call.encode(&function);

        assert_eq!(encoded, vec![10, 20, 10, 2]);
    }

    #[test]
    fn test_contract_abi_encode_call() {
        let abi = ContractAbi {
            functions: vec![
                AbiFunction {
                    name: "add".to_string(),
                    address: 10,
                    arg_count: 2,
                },
                AbiFunction {
                    name: "sub".to_string(),
                    address: 20,
                    arg_count: 2,
                },
            ],
        };

        let call = AbiCall {
            function: "add".to_string(),
            args: vec![10, 20],
        };

        let encoded = abi.encode_call(&call).unwrap();

        assert_eq!(encoded, vec![10, 20, 10, 2]);
    }
}