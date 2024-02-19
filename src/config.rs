use crate::{Error, Result};
use std::{env, str::FromStr, sync::OnceLock};

pub struct Config {
	// -- Crypt
	pub pwd_key: Vec<u8>,

	pub token_key: Vec<u8>,
	pub token_duration_sec: f64,

	// -- Db
	pub db_url: String,

	// -- Web
	pub web_folder: String,
}

pub fn config() -> &'static Config {
	static INSTANCE: OnceLock<Config> = OnceLock::new();

	INSTANCE.get_or_init(|| {
		Config::load_from_env().unwrap_or_else(|err| {
			panic!("FATAL - WHILE LOADING CONFIG - Cause: {err:?}")
		})
	})
}

impl Config {
	fn load_from_env() -> Result<Config> {
		Ok(Config {
			pwd_key: get_env_b64u_as_u8s("SERVICE_PWD_KEY")?,
			token_key: get_env_b64u_as_u8s("SERVICE_TOKEN_KEY")?,
			token_duration_sec: get_env_parse("SERVICE_TOKEN_DURATION_SEC")?,
			web_folder: get_env("SERVICE_WEB_FOLDER")?,
			db_url: get_env("SERVICE_DB_URL")?,
		})
	}
}

fn get_env(name: &'static str) -> Result<String> {
	env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}

fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
	let val = get_env(name)?;
	val.parse::<T>().map_err(|_| Error::ConfigWrongFormat(name))
}

fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>> {
	base64_url::decode(&get_env(name)?).map_err(|_| Error::ConfigWrongFormat(name))
}
