#[derive(Debug)]
pub enum Error {
	FailToDeleteNoStoreForType,
	FailToUpdateNoStoreForType,
	// -- Serde/Json error wrappers.
	FailFromOrToJsonValue,
}

impl From<serde_json::Error> for Error {
	fn from(_: serde_json::Error) -> Self {
		Error::FailFromOrToJsonValue
	}
}

// region:    --- Error Boiler
impl std::fmt::Display for Error {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> core::result::Result<(), std::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}
// endregion: --- Error Boiler
