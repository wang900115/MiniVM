#[derive(Debug, Clone, PartialEq, Eq)]

pub struct Stack {
    // dynamic array to store stack elements
    data: Vec<i32>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    pub fn push(&mut self, value: i32) {
        self.data.push(value);
    }

    pub fn pop(&mut self) -> Option<i32> {
        self.data.pop()
    }

    pub fn get(&self, index: usize) -> Option<i32> {
        self.data.get(index).copied()
    }

    pub fn set(&mut self, index: usize, value: i32) -> Result<(), ()> {
        if index  >= self.data.len() {
            return Err(());
        }
        self.data[index] = value;
        Ok(())
    } 

    pub fn peek(&self) -> Option<&i32> {
        self.data.last()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push() {
        let mut stack = Stack::new();

        stack.push(10);
        stack.push(20);

        assert_eq!(stack.len(), 2);
        assert_eq!(stack.peek(), Some(&20));
    }

    #[test]
    fn test_pop() {
        let mut stack = Stack::new();
        stack.push(10);
        stack.push(20);

        assert_eq!(stack.pop(), Some(20));
        assert_eq!(stack.pop(), Some(10));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_peek() {
        let mut stack = Stack::new();
        assert_eq!(stack.peek(), None);

        stack.push(10);
        assert_eq!(stack.peek(), Some(&10));

        stack.push(20);
        assert_eq!(stack.peek(), Some(&20));
    }

    #[test]
    fn test_empty_stack() {
        let mut stack = Stack::new();
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.peek(), None);
        assert_eq!(stack.pop(), None);
    }
}