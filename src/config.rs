/*
	Copyright (c) 2021 Lucy <lucy@absolucy.moe>

	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use color_eyre::eyre::{Result, WrapErr};
use rand::RngCore;
use serde::Deserialize;
use std::{
	net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
	num::NonZeroUsize,
	path::PathBuf,
};

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct Config {
	/// The address/port to bind the web server to.
	/// Default: localhost:8080
	pub address: SocketAddr,
	/// The base URL for the site.
	/// Should NOT include a trailing slash!
	pub site_url: String,
	/// Whether the API key is required to paste or not.
	pub public: bool,
	/// An optional directory where static files will be served from.
	/// Static files take priority over pastes!
	pub static_dir: Option<PathBuf>,
	/// Maximum size of a single paste in bytes.
	/// Default: 8 MB
	pub paste_limit: NonZeroUsize,
	/// Maximium size of the cache in bytes.
	/// Default: 64 MB
	pub cache_limit: NonZeroUsize,
	/// Path where the database will be created.
	/// Default: sweetpaste.db
	pub db_path: PathBuf,
	/// A password, used for uploading on non-public instances, and deleting *any* paste.
	pub password: String,
	/// The encryption key used to encrypt paste IDs for the public API.
	#[serde(with = "hex::serde")]
	pub id_key: [u8; 32],
	/// A list of "trusted" IPs, which will be trusted to provide
	/// valid X-Forwarded-For/X-Real-IP headers.
	/// Default: 127.0.0.1, ::1
	pub trusted_ips: Vec<IpAddr>,
	/// Syntax highlighting configuration.
	pub syntax_highlighting: SyntaxHighlightConfig,
}

impl Config {
	/// Attempts to load the configuration from the `config.toml` file.
	pub async fn load() -> Result<Self> {
		// Open the `config.toml` file as a string.
		let file = tokio::fs::read_to_string("config.toml")
			.await
			.wrap_err("failed to read config.toml")?;
		// Parse the file string as a TOML file.
		let config = toml::from_str::<Self>(&file).wrap_err("failed to parse config.toml")?;
		if config.id_key.iter().all(|&x| x == 0) {
			let mut new_key = [0_u8; 32];
			rand::thread_rng().fill_bytes(&mut new_key);
			Err(color_eyre::eyre::eyre!(
				"You need to set the ID key!\nIf you need a key, try this:\nid-key = \"{}\"",
				hex::encode(&new_key)
			))
		} else {
			Ok(config)
		}
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			// Defaults to localhost:8080
			address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
			// Defaults to localhost.
			site_url: "http://127.0.0.1:8080".to_string(),
			/// Private site by default.
			public: false,
			// Defaults to no static file serving.
			static_dir: None,
			// Default paste limit is 8 MB.
			paste_limit: NonZeroUsize::new(8388608).unwrap_or_else(|| unreachable!()),
			// Default cache limit is 64 MB.
			cache_limit: NonZeroUsize::new(67108864).unwrap_or_else(|| unreachable!()),
			// Default database path is `sweetpaste.db`.
			db_path: PathBuf::from("sweetpaste.db"),
			// This is not a secure password. You should change this.
			password: "hunter2".to_string(),
			/// This key will be rejected by default!
			id_key: [0; 32],
			// Defaults to localhost.
			trusted_ips: vec![
				IpAddr::V4(Ipv4Addr::LOCALHOST),
				IpAddr::V6(Ipv6Addr::LOCALHOST),
			],
			// Default configuration.
			syntax_highlighting: SyntaxHighlightConfig::default(),
		}
	}
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct SyntaxHighlightConfig {
	pub theme: String,
	/// A path to a directory containing .tmTheme files.
	pub themes_folder: Option<PathBuf>,
	/// A path to a directory containing .sublime-syntax files.
	pub syntax_folder: Option<PathBuf>,
}

impl Default for SyntaxHighlightConfig {
	fn default() -> Self {
		Self {
			// Use the "base16-eighties.dark" theme by default
			theme: "base16-eighties.dark".to_string(),
			// Don't load any folders by default.
			themes_folder: None,
			syntax_folder: None,
		}
	}
}
