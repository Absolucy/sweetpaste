/*
	Copyright (c) 2021 Lucy <lucy@absolucy.moe>

	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::{error::Error, id::Id, state::State};
use serde::Deserialize;
use std::{net::IpAddr, sync::Arc};
use warp::http::StatusCode;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Upload {
	password: Option<String>,
	name: Option<String>,
	syntax: Option<String>,
	content: String,
}

pub async fn post(
	state: Arc<State>,
	ip: IpAddr,
	authorization: Option<String>,
	upload: Upload,
) -> Result<impl warp::Reply, Error> {
	let authorized = authorization
		.or_else(|| upload.password.clone())
		.map(|auth| auth == state.config.password)
		.unwrap_or(false);
	if !state.config.public && !authorized {
		return Ok(warp::reply::with_header(
			warp::reply::with_status(
				warp::reply::html("unauthorized".to_string()),
				StatusCode::UNAUTHORIZED,
			),
			"Location",
			state.config.site_url.clone(),
		));
	}
	// Convert the IP address to bytes.
	let ip_bytes = match ip {
		IpAddr::V4(ipv4) => ipv4.octets().to_vec(),
		IpAddr::V6(ipv6) => ipv6.octets().to_vec(),
	};
	// Get the syntax name, if any.
	// We'll also check the first line of the content.
	let syntax = upload
		.syntax
		.as_ref()
		.and_then(|syntax_name| state.syntax_set.find_syntax_by_token(syntax_name))
		.or_else(|| state.syntax_set.find_syntax_by_first_line(&upload.content))
		.map(|syntax| syntax.name.to_string());
	// Submit the paste to the database, getting the new ID in return.
	let id = Id::from(
		sqlx::query!(
			r#"
		INSERT INTO pastes
			(name, ip, syntax, content)
		VALUES
			($1, $2, $3, $4)
		RETURNING
			id as "id: i64"
		"#,
			upload.name,
			ip_bytes,
			syntax,
			upload.content
		)
		.fetch_one(&state.pool)
		.await?
		.id,
	);
	let url = format!("{}/{}", state.config.site_url, id.encode(&state).await);
	let response = state
		.handlebars
		.render("redirect", &serde_json::json!({ "url": url }))?;
	// Reply with the URL to the new paste, along with a redirect header.
	Ok(warp::reply::with_header(
		warp::reply::with_status(warp::reply::html(response), warp::http::StatusCode::CREATED),
		"Location",
		url,
	))
}
