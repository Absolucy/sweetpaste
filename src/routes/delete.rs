/*
	Copyright (c) 2021 Lucy <lucy@absolucy.moe>

	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::{error::Error, id::Id, state::State};
use std::{net::IpAddr, sync::Arc};
use warp::http::StatusCode;

pub async fn delete(
	id: String,
	state: Arc<State>,
	ip: IpAddr,
	authorization: Option<String>,
) -> Result<impl warp::Reply, Error> {
	let id = i64::from(Id::decode(&state, &id).await?);
	// Convert the IP address to bytes.
	let ip_bytes = match ip {
		IpAddr::V4(ipv4) => ipv4.octets().to_vec(),
		IpAddr::V6(ipv6) => ipv6.octets().to_vec(),
	};
	// Check to see if the API key matches.
	let authorized = authorization
		.map(|key| key == state.config.password)
		.unwrap_or(false);
	// Delete the paste from the database,
	// as long as the sender's IP address matches that of the uploader's,
	// or if `authorized` is true.
	if sqlx::query!(
		r#"
		DELETE FROM
			pastes
		WHERE
			id = $1 AND
			(ip = $2 OR $3 = 1)
	"#,
		id,
		ip_bytes,
		authorized
	)
	.execute(&state.pool)
	.await?
	.rows_affected()
		> 0
	{
		state.cache.lock().await.pop(&id);
		Ok(warp::reply::with_status("removed", StatusCode::OK))
	} else {
		Ok(warp::reply::with_status(
			"not removed",
			StatusCode::BAD_REQUEST,
		))
	}
}
