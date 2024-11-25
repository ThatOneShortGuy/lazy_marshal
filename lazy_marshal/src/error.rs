use std::{error::Error, fmt::Display, string::FromUtf8Error};

#[derive(Debug, Clone)]
pub enum MarshalError {
    EarlyStreamEnd,
    InvalidDecode,
    InvalidSizedDecode(usize),
    InvalidData(String),
}

impl Error for MarshalError {}

impl Display for MarshalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:#?}")
    }
}

impl From<FromUtf8Error> for MarshalError {
    fn from(value: FromUtf8Error) -> Self {
        match value.utf8_error().error_len() {
            Some(l) => Self::InvalidSizedDecode(l),
            None => Self::InvalidDecode,
        }
    }
}
