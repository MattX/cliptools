use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub struct CliptoolsError {
    message: String,
    exit_code: Option<u8>,
}

impl std::fmt::Display for CliptoolsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CliptoolsError {}
