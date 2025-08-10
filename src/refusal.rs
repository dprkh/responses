use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub struct Refusal(String);

impl Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(string) = self;

        string.fmt(f)
    }
}

impl From<Refusal> for String {
    fn from(value: Refusal) -> Self {
        let Refusal(value) = value;

        value
    }
}

impl From<String> for Refusal {
    fn from(value: String) -> Self {
        Self(value)
    }
}
