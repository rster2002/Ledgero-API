#[derive(Debug)]
pub enum Error {
    Server(ledgero_api::error::Error),
    DotEnv(dotenv::Error),
}

impl From<ledgero_api::error::Error> for Error {
    fn from(value: ledgero_api::error::Error) -> Self {
        Error::Server(value)
    }
}

impl From<dotenv::Error> for Error {
    fn from(value: dotenv::Error) -> Self {
        Error::DotEnv(value)
    }
}
