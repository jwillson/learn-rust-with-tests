use std::io::Write;

// ANCHOR: code
pub trait Store {
    fn fetch(&self) -> String;
}

pub fn respond(writer: &mut impl Write, store: &impl Store) -> std::io::Result<()> {
    write!(writer, "{}", store.fetch())
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    struct StubStore {
        response: String,
    }

    impl Store for StubStore {
        fn fetch(&self) -> String {
            self.response.clone()
        }
    }

    #[test]
    fn returns_data_from_the_store() {
        let store = StubStore {
            response: "hello, world".to_string(),
        };
        let mut response = Vec::new();

        respond(&mut response, &store).unwrap();

        assert_eq!(String::from_utf8(response).unwrap(), "hello, world");
    }
    // ANCHOR_END: test
}
