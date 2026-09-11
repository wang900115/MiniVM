#[derive(Debug, Default)]

pub struct Register {
    pub pc: usize,
    pub sp: usize,
    pub fp: usize,
}

impl Register {
    pub fn new() -> Self {
        Self {
            pc: 0,
            sp: 0,
            fp: 0,
        }
    }
}
