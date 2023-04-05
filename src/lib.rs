// #![allow(unused)]

// -- Re-exports
pub use error::Error;
pub type Result<T> = core::result::Result<T, Error>;
pub use store::Store;

// -- Sub Modules
mod error;
mod store;
