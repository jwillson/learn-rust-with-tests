// ANCHOR: code
#[derive(Default)]
pub struct StackOfInts {
    values: Vec<i32>,
}

impl StackOfInts {
    pub fn new() -> StackOfInts {
        StackOfInts::default()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn push(&mut self, value: i32) {
        self.values.push(value);
    }

    pub fn pop(&mut self) -> Option<i32> {
        self.values.pop()
    }
}

#[derive(Default)]
pub struct StackOfStrings {
    values: Vec<String>,
}

impl StackOfStrings {
    pub fn new() -> StackOfStrings {
        StackOfStrings::default()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn push(&mut self, value: String) {
        self.values.push(value);
    }

    pub fn pop(&mut self) -> Option<String> {
        self.values.pop()
    }
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;
    use traits_generics_v2::assert_equal;

    // ANCHOR: test
    #[test]
    fn integer_stack() {
        let mut stack = StackOfInts::new();

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
    }

    #[test]
    fn string_stack() {
        let mut stack = StackOfStrings::new();

        // check stack is empty
        assert!(stack.is_empty());

        // add a thing, then check it's not empty
        stack.push("123".to_string());
        assert!(!stack.is_empty());

        // add another thing, pop it back again
        stack.push("456".to_string());
        assert_equal(stack.pop(), Some("456".to_string()));
        assert_equal(stack.pop(), Some("123".to_string()));
        assert!(stack.is_empty());
    }
    // ANCHOR_END: test
}
