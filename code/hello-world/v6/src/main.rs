// ANCHOR: code
const SPANISH: &str = "Spanish";
const FRENCH: &str = "French";

const ENGLISH_HELLO_PREFIX: &str = "Hello, ";
const SPANISH_HELLO_PREFIX: &str = "Hola, ";
const FRENCH_HELLO_PREFIX: &str = "Bonjour, ";

fn hello(name: &str, language: &str) -> String {
    let name = if name.is_empty() { "World" } else { name };

    if language == SPANISH {
        return format!("{SPANISH_HELLO_PREFIX}{name}");
    }

    if language == FRENCH {
        return format!("{FRENCH_HELLO_PREFIX}{name}");
    }

    format!("{ENGLISH_HELLO_PREFIX}{name}")
}

fn main() {
    println!("{}", hello("world", ""));
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saying_hello_to_people() {
        let got = hello("Chris", "");
        let want = "Hello, Chris";

        assert_eq!(got, want);
    }

    #[test]
    fn empty_string_defaults_to_world() {
        let got = hello("", "");
        let want = "Hello, World";

        assert_eq!(got, want);
    }

    #[test]
    fn in_spanish() {
        let got = hello("Elodie", "Spanish");
        let want = "Hola, Elodie";

        assert_eq!(got, want);
    }

    #[test]
    fn in_french() {
        let got = hello("Lauren", "French");
        let want = "Bonjour, Lauren";

        assert_eq!(got, want);
    }
}
