#![allow(dead_code)]

pub struct Stack<T> {
    elements: Vec<T>,
}

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
