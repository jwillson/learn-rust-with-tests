// ANCHOR: code
const ENGLISH_HELLO_PREFIX: &str = "Hello, ";

fn hello(name: &str) -> String {
    format!("{ENGLISH_HELLO_PREFIX}{name}")
}

fn main() {
    println!("{}", hello("world"));
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saying_hello_to_people() {
        let got = hello("Chris");
        let want = "Hello, Chris";

        assert_eq!(got, want);
    }
}
