use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
	// Key
	KeyFailHmac,

	// -- Pwd
	PwdNotMatching,

	// -- Token,
	TokenInvalidFormat,
	TokenCannotDecodeIdent,
	TokenCannotDecodeExp,
	TokenSignatureNotMatching,
	TokenExpNotIso,
	TokenExpired,
}

impl core::fmt::Display for Error {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}
