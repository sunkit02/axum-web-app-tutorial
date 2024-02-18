use crate::{Error, Result};
use std::{env, sync::OnceLock};

pub struct Config {
	// --Db
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
			web_folder: get_env("SERVICE_WEB_FOLDER")?,
			db_url: get_env("SERVICE_DB_URL")?,
		})
	}
}

fn get_env(name: &'static str) -> Result<String> {
	env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}
