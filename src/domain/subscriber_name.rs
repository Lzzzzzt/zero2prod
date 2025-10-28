use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl TryFrom<String> for SubscriberName {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let is_empty_or_whitespace = value.trim().is_empty();

        let is_too_long = value.graphemes(true).count() > 255;

        const FORBIDDEN_CHARACTERS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters =
            value.chars().any(|c| FORBIDDEN_CHARACTERS.contains(&c));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("Subscriber name: {value} is not valid."))
        } else {
            Ok(SubscriberName(value))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<SubscriberName> for String {
    fn from(value: SubscriberName) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_255_grapheme_long_name_is_valid() {
        let name = "ё".repeat(255);
        assert_ok!(SubscriberName::try_from(name));
    }

    #[test]
    fn a_name_longer_than_255_graphemes_is_rejected() {
        let name = "a".repeat(256);
        assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::try_from(name));
        }
    }

    #[test]
    fn a_valid_name_is_try_fromd_successfully() {
        let name = "Lzzzt".to_string();
        assert_ok!(SubscriberName::try_from(name));
    }
}
