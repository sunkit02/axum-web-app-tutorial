use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize)]
pub enum Error {
	// Key
	KeyFailHmac,

	// -- Pwd
	PwdNotMatching,
}

impl core::fmt::Display for Error {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}
