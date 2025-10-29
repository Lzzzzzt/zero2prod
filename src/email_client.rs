use std::time::Duration;

use reqwest::{Client, Url};
use secrecy::{ExposeSecret, SecretString};

use crate::{config::EmailClientConfig, domain::Email};

pub struct EmailClient {
    sender: Email,
    base_url: Url,
    http_client: Client,
    token: SecretString,
    timeout: Duration,
}

impl EmailClient {
    pub fn new(base_url: Url, sender: Email, token: SecretString, timeout_ms: u32) -> Self {
        Self {
            base_url,
            sender,
            timeout: Duration::from_millis(timeout_ms as u64),
            token: SecretString::from(format!("Bearer {}", token.expose_secret())),
            http_client: Client::new(),
        }
    }

    pub async fn send_email(
        &self,
        to: Email,
        subject: impl AsRef<str>,
        raw_content: impl AsRef<str>,
        html_content: impl AsRef<str>,
    ) -> Result<(), reqwest::Error> {
        let url = self
            .base_url
            .join("/v3/mail/send")
            .expect("Failed to join url");

        let body = request::Body::new(
            &self.sender,
            to,
            subject.as_ref(),
            raw_content.as_ref(),
            html_content.as_ref(),
        );

        let _ = self
            .http_client
            .post(url)
            .json(&body)
            .header("Authorization", self.token.expose_secret())
            .timeout(self.timeout)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

impl From<EmailClientConfig> for EmailClient {
    fn from(value: EmailClientConfig) -> Self {
        Self::new(value.base_url, value.sender, value.token, value.timeout_ms)
    }
}

mod request {
    use serde::Serialize;

    use crate::domain::Email;

    #[derive(Serialize)]
    pub struct Body<'a> {
        personalizations: Vec<Personalization>,
        from: &'a Email,
        subject: &'a str,
        content: Vec<Content<'a>>,
    }

    impl<'a> Body<'a> {
        pub fn new(
            from: &'a Email,
            to: Email,
            subject: &'a str,
            raw_content: &'a str,
            html_content: &'a str,
        ) -> Self {
            Body {
                personalizations: vec![Personalization::new().add_one(to)],
                from,
                subject,
                content: vec![Content::text(raw_content), Content::html(html_content)],
            }
        }
    }

    #[derive(Serialize)]
    struct Personalization {
        to: Vec<Email>,
    }

    impl Personalization {
        fn new() -> Self {
            Self { to: vec![] }
        }

        fn add_one(mut self, email: Email) -> Self {
            self.to.push(email);
            self
        }
    }

    #[derive(Serialize)]
    #[serde(tag = "type")]
    enum Content<'a> {
        #[serde(rename = "text/plain")]
        Text { value: &'a str },
        #[serde(rename = "text/html")]
        Html { value: &'a str },
    }

    impl<'a> Content<'a> {
        #[inline]
        fn text(value: &'a str) -> Self {
            Self::Text { value }
        }

        #[inline]
        fn html(value: &'a str) -> Self {
            Self::Html { value }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use claims::{assert_err, assert_ok};
    use fake::{
        Fake, Faker,
        faker::{internet::en::SafeEmail, lorem::en::Sentence},
    };
    use secrecy::SecretString;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{any, header, header_exists, method},
    };

    use crate::{domain::Email, email_client::EmailClient};

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let res = serde_json::from_slice::<serde_json::Value>(&request.body);

            match res {
                Ok(body) => {
                    body.get("personalizations").is_some_and(|ps| {
                        ps.is_array()
                            && ps.get(0).is_some_and(|p| {
                                p.get("to").is_some_and(|es| {
                                    es.is_array()
                                        && es.get(0).is_some_and(|e| e.get("email").is_some())
                                })
                            })
                    }) && body.get("from").is_some_and(|e| e.get("email").is_some())
                        && body.get("subject").is_some()
                        && body.get("content").is_some_and(|c| {
                            c.is_array()
                                && c.get(0).is_some_and(|c1| {
                                    c1.get("value").is_some()
                                        && c1
                                            .get("type")
                                            .is_some_and(|t| t.as_str() == Some("text/plain"))
                                })
                                && c.get(1).is_some_and(|c2| {
                                    c2.get("value").is_some()
                                        && c2
                                            .get("type")
                                            .is_some_and(|t| t.as_str() == Some("text/html"))
                                })
                        })
                }
                Err(_) => false,
            }
        }
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Sentence(1..10).fake()
    }

    fn email() -> Email {
        Email::try_from(SafeEmail().fake::<String>()).unwrap()
    }

    fn email_client(uri: String) -> EmailClient {
        EmailClient::new(
            uri.parse().unwrap(),
            email(),
            SecretString::from(Faker.fake::<String>()),
            200,
        )
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;

        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_ok!(result);
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        let mock_server = MockServer::start().await;

        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_ok!(result);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;

        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(result);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;

        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(60)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(result);
    }
}
