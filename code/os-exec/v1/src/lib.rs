use std::io::Read;
use std::process::{Command, Stdio};

// ANCHOR: parse
/// The business logic: read XML, pull out the message, and shout it.
/// It depends only on `Read`, so it can be tested with any byte source.
pub fn get_message(mut reader: impl Read) -> std::io::Result<String> {
    let mut xml = String::new();
    reader.read_to_string(&mut xml)?;

    let document = roxmltree::Document::parse(&xml)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;

    let message = document
        .descendants()
        .find(|node| node.has_tag_name("message"))
        .and_then(|node| node.text())
        .unwrap_or_default();

    Ok(message.to_uppercase())
}
// ANCHOR_END: parse

// ANCHOR: exec
/// Fetching the data: run an external command and stream its stdout into the
/// parser. This is the only part that touches the operating system.
pub fn get_data() -> std::io::Result<String> {
    let mut child = Command::new("cat")
        .arg("msg.xml")
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("stdout was requested");
    let message = get_message(stdout)?;

    child.wait()?;
    Ok(message)
}
// ANCHOR_END: exec

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn extracts_and_uppercases_the_message() {
        let payload = b"<payload><message>Happy New Year!</message></payload>";

        let got = get_message(&payload[..]).unwrap();

        assert_eq!(got, "HAPPY NEW YEAR!");
    }

    #[test]
    fn rejects_invalid_xml() {
        let payload = b"this is not xml";

        assert!(get_message(&payload[..]).is_err());
    }
    // ANCHOR_END: test

    // ANCHOR: integration_test
    #[test]
    fn get_data_reads_the_message_from_the_command() {
        // Runs the real `cat msg.xml` command from the crate directory.
        let got = get_data().unwrap();

        assert_eq!(got, "HAPPY NEW YEAR!");
    }
    // ANCHOR_END: integration_test
}
