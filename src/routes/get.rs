/*
	Copyright (c) 2021 Lucy <lucy@absolucy.moe>

	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::{error::Error, id::Id, state::State};
use std::{collections::BTreeMap, sync::Arc};
use syntect::parsing::SyntaxReference;

fn render(
	state: &State,
	syntax: &SyntaxReference,
	name: Option<String>,
	_posted: i64,
	content: &str,
) -> Result<String, Error> {
	let highlighted = syntect::html::highlighted_html_for_string(
		content,
		&state.syntax_set,
		syntax,
		&state.theme_set.themes[state.config.syntax_highlighting.theme.as_str()],
	);
	let mut data = BTreeMap::<&'static str, String>::new();
	data.insert("content", highlighted);
	if let Some(name) = name {
		data.insert("name", name);
	}
	data.insert("language", syntax.name.clone());
	state.handlebars.render("paste", &data).map_err(Error::from)
}

pub async fn get(id: String, state: Arc<State>) -> Result<impl warp::Reply, Error> {
	let id = i64::from(Id::decode(&state, &id).await?);
	let mut cache = state.cache.lock().await;
	// Check the cache for the rendered HTML for this paste, and if so, just return that.
	if let Some(response) = cache.get(&id) {
		return Ok(warp::reply::with_status(
			warp::reply::html(response.clone()),
			warp::http::StatusCode::OK,
		));
	}
	// Try to find the paste with the given ID.
	let paste = match sqlx::query!(
		r#"
		SELECT
			name, syntax, content, posted as "posted: i64"
		FROM
			pastes
		WHERE
			id = $1
		"#,
		id
	)
	.fetch_optional(&state.pool)
	.await?
	{
		// We found it!
		Some(x) => x,
		// We didn't find it? Time to render the 404 page.
		None => {
			let rendered = state.handlebars.render("404", &()).map_err(Error::from)?;
			return Ok(warp::reply::with_status(
				warp::reply::html(rendered),
				warp::http::StatusCode::NOT_FOUND,
			));
		}
	};
	// Find the syntax highlighter for this paste,
	// otherwise use plain text as a fallback.
	let syntax_highlighting = paste
		.syntax
		.as_ref()
		.and_then(|syntax_name| state.syntax_set.find_syntax_by_name(syntax_name))
		.unwrap_or_else(|| state.syntax_set.find_syntax_plain_text());

	// Render the paste.
	let rendered = render(
		&state,
		syntax_highlighting,
		paste.name,
		paste.posted,
		&paste.content,
	)?;
	// Cache the rendered HTML for this paste, and return it.
	let _ = cache.put_with_weight(id, rendered.clone());
	Ok(warp::reply::with_status(
		warp::reply::html(rendered),
		warp::http::StatusCode::OK,
	))
}
