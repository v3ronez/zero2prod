use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(value: String) -> Result<SubscriberName, String> {
        let is_empty_whitespace = value.trim().is_empty();
        let is_too_long = value.graphemes(true).count() > 256;
        let forbiden_chars = [
            '/', '\\', '(', ')', '<', '>', ':', '"', ';', '[', ']', '{', '}', '?', '*', '|', '&',
            '#', '%', '^', '~', '`', '$', '=', '+', ',',
        ];
        let container_forbiden_chars = value.chars().any(|c| forbiden_chars.contains(&c));
        if is_empty_whitespace || is_too_long || container_forbiden_chars {
            return Err(format!("{} is not a valid subscriber name.", value));
        }
        Ok(Self(value.to_string()))
    }

    //Consume self and return the string
    pub fn inner(self) -> String {
        self.0
    }

    //share referance value of string wrapped
    pub fn inner_ref(&self) -> &str {
        &self.0
    }

    //share mutate reference from self wrapped string
    pub fn inner_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0.as_str()
    }
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};

    use crate::domain::SubscriberName;

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a̐".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }
    #[test]
    fn a_name_longer_than_256_characters_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn whitespace_only_is_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn a_name_with_invalid_chacarters_is_rejected() {
        let name = "https://github.com/v3ronez".to_string();
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn a_valid_name_passed() {
        let name = "Isabella Ceciliano".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
