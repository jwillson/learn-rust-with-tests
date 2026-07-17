use std::collections::HashMap;
use std::sync::mpsc;

// ANCHOR: code
pub type WebsiteChecker = fn(&str) -> bool;

pub fn check_websites(checker: WebsiteChecker, urls: &[&str]) -> HashMap<String, bool> {
    let mut results = HashMap::new();
    let (sender, receiver) = mpsc::channel();

    std::thread::scope(|scope| {
        for &url in urls {
            let sender = sender.clone();
            scope.spawn(move || {
                sender.send((url, checker(url))).unwrap();
            });
        }

        for _ in 0..urls.len() {
            let (url, good) = receiver.recv().unwrap();
            results.insert(url.to_string(), good);
        }
    });

    results
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_website_checker(url: &str) -> bool {
        url != "waat://furhurterwe.geds"
    }

    #[test]
    fn checks_all_the_websites() {
        let websites = [
            "http://google.com",
            "http://blog.gypsydave5.com",
            "waat://furhurterwe.geds",
        ];

        let want = HashMap::from([
            ("http://google.com".to_string(), true),
            ("http://blog.gypsydave5.com".to_string(), true),
            ("waat://furhurterwe.geds".to_string(), false),
        ]);

        let got = check_websites(mock_website_checker, &websites);

        assert_eq!(got, want);
    }
}
