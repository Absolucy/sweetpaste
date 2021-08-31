/*
	Copyright (c) 2021 Lucy <lucy@absolucy.moe>

	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

mod cache;
mod config;
mod error;
mod filter;
mod id;
mod routes;
mod state;

use crate::filter::{with_ip, with_obj};
use color_eyre::eyre::{Result, WrapErr};
use futures::TryFutureExt;
use state::State;
use std::sync::Arc;
use warp::{http::StatusCode, Filter};

async fn recover(
	state: Arc<State>,
	rejection: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
	let status: StatusCode;
	let response: String;
	if rejection.is_not_found() {
		status = StatusCode::NOT_FOUND;
		response = state
			.handlebars
			.render("404", &())
			.expect("failed to render 404 page");
	} else if let Some(err) = rejection.find::<error::Error>() {
		status = StatusCode::INTERNAL_SERVER_ERROR;
		response = err.to_string();
	} else if let Some(err) = rejection.find::<warp::body::BodyDeserializeError>() {
		status = StatusCode::BAD_REQUEST;
		response = err.to_string();
	} else if let Some(err) = rejection.find::<warp::reject::MethodNotAllowed>() {
		status = StatusCode::BAD_REQUEST;
		response = err.to_string();
	} else if let Some(err) = rejection.find::<warp::reject::PayloadTooLarge>() {
		status = StatusCode::BAD_REQUEST;
		response = err.to_string();
	} else {
		status = StatusCode::INTERNAL_SERVER_ERROR;
		response = "internal server error".to_string();
	}
	Ok(warp::reply::with_status(
		warp::reply::html(response),
		status,
	))
}

#[tokio::main]
async fn main() -> Result<()> {
	// Install our fancy error handler for color-eyre.
	color_eyre::install().wrap_err("failed to install color_eyre error handler")?;

	// Initialize our state.
	let state = Arc::new(
		State::new()
			.await
			.wrap_err("failed to initialize sweetpaste")?,
	);

	let post = warp::path::end()
		.and(warp::post())
		.and(with_obj(state.clone()))
		.and(with_ip(state.clone()))
		.and(warp::filters::body::content_length_limit(
			state.config.paste_limit.get() as u64,
		))
		.and(warp::filters::header::optional::<String>("authorization"))
		.and(warp::filters::body::form::<routes::post::Upload>())
		.and_then(|state, ip, authorization, upload| {
			routes::post::post(state, ip, authorization, upload).map_err(warp::reject::custom)
		});

	let get = warp::get()
		.and(warp::path!(String))
		.and(with_obj(state.clone()))
		.and_then(|id, state| routes::get::get(id, state).map_err(warp::reject::custom));

	let delete = warp::delete()
		.and(warp::path!(String))
		.and(with_obj(state.clone()))
		.and(with_ip(state.clone()))
		.and(warp::filters::header::optional::<String>("authorization"))
		.and_then(|id, state, ip, authorization| {
			routes::delete::delete(id, state, ip, authorization).map_err(warp::reject::custom)
		});

	let upload = warp::path::end()
		.and(warp::get())
		.and({
			// Get the name of every syntax we have loaded.
			// We do this here for efficiency - we won't need to allocate
			// this list every time we serve a paste.
			let mut languages = state
				.syntax_set
				.syntaxes()
				.iter()
				.map(|syntax| syntax.name.clone())
				.collect::<Vec<String>>();
			// Sort the syntax list.
			languages.sort();
			// Pre-render the page, it'll never change anyways
			let rendered = state
				.handlebars
				.render(
					"upload",
					&serde_json::json!({ "languages": &*languages, "public": state.config.public }),
				)
				.wrap_err("failed to pre-render upload page")?;
			with_obj(Arc::new(rendered))
		})
		.and_then(|rendered: Arc<String>| async move {
			Result::<_, std::convert::Infallible>::Ok(warp::reply::html(rendered.to_string()))
		});

	match state.config.static_dir.as_ref() {
		Some(static_dir) => {
			warp::serve(
				warp::filters::fs::dir(static_dir.clone())
					.or(post)
					.or(get)
					.or(upload)
					.or(delete)
					.recover({
						let state = state.clone();
						move |rejection| recover(state.clone(), rejection)
					}),
			)
			.run(state.config.address)
			.await;
		}
		None => {
			warp::serve(post.or(get).or(upload).or(delete).recover({
				let state = state.clone();
				move |rejection| recover(state.clone(), rejection)
			}))
			.run(state.config.address)
			.await;
		}
	}

	Ok(())
}
