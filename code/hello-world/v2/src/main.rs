// ANCHOR: code
fn hello() -> String {
    "Hello, world".to_string()
}

fn main() {
    println!("{}", hello());
}
// ANCHOR_END: code

// ANCHOR: test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saying_hello_to_the_world() {
        let got = hello();
        let want = "Hello, world";

        assert_eq!(got, want);
    }
}
// ANCHOR_END: test
