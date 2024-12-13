pub struct Stack<T> {
    elements: Vec<T>,
}

#[allow(dead_code)]
impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { elements: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.elements.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.elements.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.elements.last()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn size(&self) -> usize {
        self.elements.len()
    }
}

#[cfg(test)]
mod tests {
    use super::Stack;

    #[test]
    fn test_push() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);
        assert_eq!(stack.size(), 3);
    }

    #[test]
    fn test_pop() {
        let mut stack = Stack::new();
        stack.push(10);
        stack.push(20);
        stack.push(30);

        assert_eq!(stack.pop(), Some(30));
        assert_eq!(stack.pop(), Some(20));
        assert_eq!(stack.pop(), Some(10));
        assert_eq!(stack.pop(), None); // Popping from an empty stack
    }

    #[test]
    fn test_peek() {
        let mut stack = Stack::new();
        stack.push(100);
        stack.push(200);
        stack.push(300);

        assert_eq!(stack.peek(), Some(&300));
        stack.pop();
        assert_eq!(stack.peek(), Some(&200));
        stack.pop();
        assert_eq!(stack.peek(), Some(&100));
        stack.pop();
        assert_eq!(stack.peek(), None); // Peeking into an empty stack
    }

    #[test]
    fn test_is_empty() {
        let mut stack = Stack::new();
        assert!(stack.is_empty());

        stack.push(42);
        assert!(!stack.is_empty());

        stack.pop();
        assert!(stack.is_empty());
    }

    #[test]
    fn test_size() {
        let mut stack = Stack::new();
        assert_eq!(stack.size(), 0);

        stack.push(1);
        assert_eq!(stack.size(), 1);

        stack.push(2);
        stack.push(3);
        assert_eq!(stack.size(), 3);

        stack.pop();
        assert_eq!(stack.size(), 2);

        stack.pop();
        stack.pop();
        assert_eq!(stack.size(), 0);
    }
}