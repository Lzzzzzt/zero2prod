use crate::routes::FormData;

use super::*;

pub struct Subscriber {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
}

impl TryFrom<FormData> for Subscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name.try_into()?,
            email: value.email.try_into()?,
        })
    }
}
