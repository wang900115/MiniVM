pub trait Host {
    fn storage_load(&self, key: i32) -> i32;

    fn storage_store(&mut self, key: i32, value: i32);

    fn emit(&mut self, event: i32);
}

use std::collections::HashMap;

pub struct SimpleHost {
    pub storage: HashMap<i32, i32>,
    pub events: Vec<i32>,
}

impl SimpleHost {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            events: Vec::new(),
        }
    }
}

impl Host for SimpleHost {
    fn storage_load(&self, key: i32) -> i32 {
        *self.storage.get(&key).unwrap_or(&0)
    }

    fn storage_store(&mut self, key: i32, value: i32) {
        self.storage.insert(key, value);
    }

    fn emit(&mut self, event: i32) {
        self.events.push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_load_and_store() {
        let mut host = SimpleHost::new();

        assert_eq!(host.storage_load(1), 0);
        host.storage_store(1, 42);
        assert_eq!(host.storage_load(1), 42);
    }
}