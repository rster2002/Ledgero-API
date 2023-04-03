#[derive(Debug)]
pub enum Error {
    Server(ledgero_api::error::Error),
}

impl From<ledgero_api::error::Error> for Error {
    fn from(value: ledgero_api::error::Error) -> Self {
        Error::Server(value)
    }
}
