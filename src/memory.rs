#[derive(Debug, Clone, PartialEq, Eq)]

pub struct Memory {
    data: Vec<i32>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            data: Vec::new(),
        }
    }

    pub fn store(&mut self, address: usize, value: i32) {
        if address >= self.data.len() {
            self.data.resize(address + 1,0);
        }
        self.data[address] = value;
    }

    pub fn load(&self, address: usize) -> i32 {
        if address >= self.data.len() {
            return 0;
        }
        self.data[address]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_load() {
        let mut memory = Memory::new();

        memory.store(0, 100);
        memory.store(1, 200);

        assert_eq!(memory.load(0), 100);
        assert_eq!(memory.load(1), 200);
    }

    #[test]
    fn test_load_empty_memory() {
        let memory = Memory::new();

        assert_eq!(memory.load(10), 0);
    }

    #[test]
    fn test_store_large_address() {
        let mut memory = Memory::new();

        memory.store(5,999);

        assert_eq!(memory.load(5), 999);
        assert_eq!(memory.load(4), 0);
    }
}
