#[derive(Debug, Clone)]
pub struct CallFrame {
    pub return_pc: usize,
    pub stack_base: usize,
    pub previous_fp: usize,
}