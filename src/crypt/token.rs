use std::str::FromStr;

use crate::config;
use crate::crypt::{encrypt_into_b64u, EncryptContent, Error, Result};
use crate::utils::{
	b64u_decode, b64u_encode, now_utc, now_utc_plus_sec_str, parse_utc,
};

// region:      --- Token Type

/// String format: `ident_b64u.exp_b64u.sign_b64u`
#[derive(Debug, PartialEq)]
pub struct Token {
	/// Identifier (username for example)
	pub ident: String,
	/// Expiration date in Rfc3339
	pub exp: String,
	/// Signature, base64url encoded
	pub sign_b64u: String,
}

impl FromStr for Token {
	type Err = Error;

	fn from_str(token_str: &str) -> std::prelude::v1::Result<Self, Self::Err> {
		let splits: Vec<&str> = token_str.split('.').collect();

		let [ident_b64u, exp_b64u, sign_b64u] = splits[..] else {
			// splits.len() != 3
			return Err(Error::TokenInvalidFormat);
		};

		Ok(Self {
			ident: b64u_decode(ident_b64u)
				.map_err(|_| Error::TokenCannotDecodeIdent)?,
			exp: b64u_decode(exp_b64u).map_err(|_| Error::TokenCannotDecodeExp)?,
			sign_b64u: sign_b64u.to_string(),
		})
	}
}

impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}.{}.{}",
			b64u_encode(&self.ident),
			b64u_encode(&self.exp),
			self.sign_b64u
		)
	}
}

// endregion:   --- Token Type

// region:      --- Web Token Gen and Validation

pub fn generate_web_token(user: &str, salt: &str) -> Result<Token> {
	let config = &config();
	_generate_token(user, config.token_duration_sec, salt, &config.token_key)
}

pub fn validate_web_token(origin_token: &Token, salt: &str) -> Result<()> {
	let config = &config();
	_validate_token_sign_and_exp(origin_token, salt, &config.token_key)?;

	Ok(())
}

// endregion:   --- Web Token Gen and Validation

// region:      --- (private) Token Gen and Validation

fn _generate_token(
	ident: &str,
	duration_sec: f64,
	salt: &str,
	key: &[u8],
) -> Result<Token> {
	// -- Compute the first two components
	let ident = ident.to_string();
	let exp = now_utc_plus_sec_str(duration_sec);

	// -- Sign the two first two components
	let sign_b64u = _token_sign_into_b64u(&ident, &exp, salt, key)?;

	Ok(Token {
		ident,
		exp,
		sign_b64u,
	})
}

fn _validate_token_sign_and_exp(
	origin_token: &Token,
	salt: &str,
	key: &[u8],
) -> Result<()> {
	// -- Validate signature
	let new_sign_b64u =
		_token_sign_into_b64u(&origin_token.ident, &origin_token.exp, salt, key)?;

	if new_sign_b64u != origin_token.sign_b64u {
		return Err(Error::TokenSignatureNotMatching);
	}

	// -- Validate expiration
	let origin_exp =
		parse_utc(&origin_token.exp).map_err(|_| Error::TokenExpNotIso)?;
	let now = now_utc();

	if origin_exp < now {
		return Err(Error::TokenExpired);
	}

	Ok(())
}

/// Create token signature from token parts and salt.
fn _token_sign_into_b64u(
	ident: &str,
	exp: &str,
	salt: &str,
	key: &[u8],
) -> Result<String> {
	let content = format!("{}.{}", b64u_encode(ident), b64u_encode(exp));
	let signature = encrypt_into_b64u(
		key,
		&EncryptContent {
			content,
			salt: salt.to_string(),
		},
	);

	signature
}

// endregion:   --- (private) Token Gen and Validation

#[cfg(test)]
mod tests {
	use std::{thread, time::Duration};

	use super::*;

	#[test]
	fn test_token_display_ok() -> Result<()> {
		// -- Fixtures
		let fx_token_str =
			"ZngtaWRlbnQtMDE.MjAyMy0wNS0xN1QxNTo4MDowMFo.some-sign-b64u-encoded";
		let fx_token = Token {
			ident: "fx-ident-01".to_string(),
			exp: "2023-05-17T15:80:00Z".to_string(),
			sign_b64u: "some-sign-b64u-encoded".to_string(),
		};

		// -- Exec & Check
		assert_eq!(fx_token.to_string(), fx_token_str);

		Ok(())
	}

	#[test]
	fn test_token_from_str_ok() -> Result<()> {
		// -- Fixtures
		let fx_token_str =
			"ZngtaWRlbnQtMDE.MjAyMy0wNS0xN1QxNTo4MDowMFo.some-sign-b64u-encoded";
		let fx_token = Token {
			ident: "fx-ident-01".to_string(),
			exp: "2023-05-17T15:80:00Z".to_string(),
			sign_b64u: "some-sign-b64u-encoded".to_string(),
		};

		// -- Exec
		let token: Token = fx_token_str.parse()?;

		// -- Check
		assert_eq!(token, fx_token);

		Ok(())
	}

	#[test]
	fn test_validate_web_token_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_user = "user_one";
		let fx_salt = "pepper";
		let fx_duration_sec = 0.02; // 20ms
		let token_key = &config().token_key;
		let fx_token =
			_generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

		// -- Exec
		thread::sleep(Duration::from_millis(10));
		let res = validate_web_token(&fx_token, fx_salt);

		// -- Check
		res?;

		Ok(())
	}

	#[test]
	fn test_validate_web_token_err_expire() -> Result<()> {
		// -- Setup & Fixtures
		let fx_user = "user_one";
		let fx_salt = "pepper";
		let fx_duration_sec = 0.01; // 10ms
		let token_key = &config().token_key;
		let fx_token =
			_generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

		// -- Exec
		thread::sleep(Duration::from_millis(20));
		let res = validate_web_token(&fx_token, fx_salt);

		// -- Check
		assert!(
			matches!(res, Err(Error::TokenExpired)),
			"Should have matched `Err(Error::TokenExpired)` but was `{res:?}`"
		);

		Ok(())
	}
}
