
#[derive(Debug, Clone)]
pub struct ApplicationError {
    pub message: String
}

impl ApplicationError {
    pub fn new(message: &str) -> ApplicationError {
        ApplicationError { message: message.to_string() }
    }
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApplicationError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        Option::None
    }
    fn description(&self) -> &str {
        &self.message
    }
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Option::None
    }
}