pub const URL: &str = "https://pastebin.com";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url() {
        assert_eq!(URL, "https://pastebin.com");
    }
}
