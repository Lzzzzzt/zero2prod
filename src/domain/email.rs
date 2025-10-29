use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
};
use validator::ValidateEmail;

#[derive(Debug, Serialize, Clone)]
pub struct Email {
    #[serde(rename = "email")]
    inner: String,
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EmailVisitor;

        impl<'de> Visitor<'de> for EmailVisitor {
            type Value = String;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an valid email string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.to_string())
            }
        }

        let value = deserializer.deserialize_string(EmailVisitor)?;

        Email::try_from(value).map_err(de::Error::custom)
    }
}

impl TryFrom<String> for Email {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.validate_email() {
            Ok(Self { inner: value })
        } else {
            Err(format!("Email: {value} is not valid."))
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::Email;
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
        Email::try_from(valid_email.0).is_ok()
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(Email::try_from(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "mainlzzzt.cc".to_string();
        assert_err!(Email::try_from(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@lzzzt.cc".to_string();
        assert_err!(Email::try_from(email));
    }
}
