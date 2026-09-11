pub struct Gas {
    remaining: u64, 
}

impl Gas {
    pub fn new(amount: u64) -> Self {
        Self {
            remaining: amount,
        }
    }

    pub fn consume(&mut self, amount: u64) -> bool {
        if self.remaining < amount {
            return false;
        }
        self.remaining -= amount;
        true
    }

    pub fn remaining(&self) -> u64 {
        self.remaining
    }
}