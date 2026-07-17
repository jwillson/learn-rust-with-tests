// ANCHOR: code
pub struct Stack<T> {
    values: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack { values: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn push(&mut self, value: T) {
        self.values.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.values.pop()
    }
}
// ANCHOR_END: code

impl<T> Default for Stack<T> {
    fn default() -> Stack<T> {
        Stack::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use traits_generics_v2::assert_equal;

    // ANCHOR: test
    #[test]
    fn integer_stack() {
        let mut stack = Stack::new();

        // check stack is empty
        assert!(stack.is_empty());

        // add a thing, then check it's not empty
        stack.push(123);
        assert!(!stack.is_empty());

        // add another thing, pop it back again
        stack.push(456);
        assert_equal(stack.pop(), Some(456));
        assert_equal(stack.pop(), Some(123));
        assert!(stack.is_empty());

        // can use the numbers we put in as numbers, no downcasting required
        stack.push(1);
        stack.push(2);
        let first_num = stack.pop().unwrap();
        let second_num = stack.pop().unwrap();
        assert_equal(first_num + second_num, 3);
    }
    // ANCHOR_END: test

    // ANCHOR: turbofish
    #[test]
    fn an_empty_stack_needs_its_type_spelled_out() {
        let stack = Stack::<String>::new();

        assert!(stack.is_empty());
    }
    // ANCHOR_END: turbofish
}
