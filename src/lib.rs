pub mod site;
pub use crate::site::Site;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
