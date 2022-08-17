#[derive(thiserror::Error, Debug)]
pub enum BaseCrateError {
    #[error("Division by zero")]
    DivisionByZero,

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub(crate) type Result<T> = std::result::Result<T, BaseCrateError>;
