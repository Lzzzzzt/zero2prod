use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl TryFrom<String> for SubscriberEmail {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.validate_email() {
            Ok(Self(value))
        } else {
            Err(format!("Email: {value} is not valid."))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::assert_err;
    use fake::Fake;
    use rand::{SeedableRng, rngs::StdRng};

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = fake::faker::internet::en::SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::try_from(valid_email.0).is_ok()
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::try_from(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "mainlzzzt.cc".to_string();
        assert_err!(SubscriberEmail::try_from(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@lzzzt.cc".to_string();
        assert_err!(SubscriberEmail::try_from(email));
    }
}
