/*
	Copyright (c) 2021 Lucy <lucy@absolucy.moe>

	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::{cache::HtmlCache, config::Config};
use chacha20::{cipher::NewCipher, ChaCha8, Key, Nonce};
use color_eyre::eyre::{Result, WrapErr};
use handlebars::{Handlebars, Template};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};
use tokio::sync::Mutex;

/// Contains shared state for the sake of organization.
pub struct State {
	/// The application configuration.
	pub config: Config,
	/// The database connection pool.
	pub pool: SqlitePool,
	/// The cache for rendered pages.
	pub cache: HtmlCache,
	/// The syntax set, which stores syntax highlighting stuff.
	pub syntax_set: SyntaxSet,
	/// The theme set, which stores syntax highlighting themes.
	pub theme_set: ThemeSet,
	/// The Handlebars context for templating.
	pub handlebars: Handlebars<'static>,
	/// The ChaCha8 context for encrypting paste IDs.
	pub chacha: Mutex<ChaCha8>,
}

impl State {
	pub async fn new() -> Result<Self> {
		// Load the configuration.
		let config = Config::load().await.wrap_err("failed to load config")?;
		// Open the database.
		let pool = Self::build_db(&config)
			.await
			.wrap_err("failed to open database")?;
		// Set up our in-memory cache.
		let cache = crate::cache::create_cache(&config);
		// Set up our syntax set.
		let syntax_set =
			Self::build_syntax_definitions(&config).wrap_err("failed to build syntax set")?;
		// Set up our theme set.
		let theme_set = Self::build_syntax_themes(&config).wrap_err("failed to build theme set")?;
		// Set up our handlebars context.
		let handlebars = Self::build_handlebars().wrap_err("failed to build handlebars context")?;
		// Set up our ChaCha8 context.
		let chacha = Mutex::new(ChaCha8::new(
			Key::from_slice(&config.id_key),
			Nonce::from_slice(&[0_u8; 12]),
		));
		Ok(Self {
			config,
			pool,
			cache,
			syntax_set,
			theme_set,
			handlebars,
			chacha,
		})
	}

	/// Open/create the SQLite database.
	async fn build_db(config: &Config) -> Result<SqlitePool> {
		let pool = SqlitePool::connect_with(
			SqliteConnectOptions::new()
				.filename(&config.db_path)
				.create_if_missing(true),
		)
		.await
		.wrap_err("failed to open sqlite db")?;
		// Run SQLite migrations
		sqlx::migrate!()
			.run(&pool)
			.await
			.wrap_err("failed to run sqlite migrations")?;
		Ok(pool)
	}

	/// Create the Handlebars state.
	fn build_handlebars() -> Result<Handlebars<'static>> {
		let mut handlebars = Handlebars::new();
		// Register the template for the paste page.
		handlebars.register_template(
			"paste",
			Template::compile(include_str!("../template/paste.html"))
				.wrap_err("failed to compile 'paste' template")?,
		);
		// Register the template for the 404 page.
		handlebars.register_template(
			"404",
			Template::compile(include_str!("../template/404.html"))
				.wrap_err("failed to compile '404' template")?,
		);
		// Register the template for the upload page.
		handlebars.register_template(
			"upload",
			Template::compile(include_str!("../template/upload.html"))
				.wrap_err("failed to compile 'upload' template")?,
		);
		// Register the template for the redirect page.
		handlebars.register_template(
			"redirect",
			Template::compile(include_str!("../template/redirect.html"))
				.wrap_err("failed to compile 'redirect' template")?,
		);
		Ok(handlebars)
	}

	// Build the syntax definitions.
	fn build_syntax_definitions(config: &Config) -> Result<SyntaxSet> {
		let mut syntax_set_builder = SyntaxSet::load_defaults_newlines().into_builder();
		if let Some(syntax_path) = config.syntax_highlighting.syntax_folder.as_ref() {
			syntax_set_builder
				.add_from_folder(syntax_path, true)
				.wrap_err_with(|| format!("failed to add syntax from {}", syntax_path.display()))?;
		}
		Ok(syntax_set_builder.build())
	}

	// Build the syntax highlighting themes.
	fn build_syntax_themes(config: &Config) -> Result<ThemeSet> {
		let mut theme_set = ThemeSet::load_defaults();
		if let Some(theme_path) = config.syntax_highlighting.themes_folder.as_ref() {
			theme_set
				.add_from_folder(theme_path)
				.wrap_err_with(|| format!("failed to add themes from {}", theme_path.display()))?;
		}
		Ok(theme_set)
	}
}
