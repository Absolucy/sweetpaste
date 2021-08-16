/*
	Copyright (c) 2021 Lucy <lucy@absolucy.moe>

	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::state::State;
use std::{
	net::{IpAddr, SocketAddr},
	sync::Arc,
};
use warp::Filter;

/// A warp filter that copies an [`Arc`].
pub fn with_obj<T: Send + Sync + Clone>(
	obj: T,
) -> impl Filter<Extract = (T,), Error = std::convert::Infallible> + Clone {
	warp::any().map(move || obj.clone())
}

/// A warp filter which extracts an IP address,
/// using either the `X-Real-IP` header or the `X-Forwarded-For` header,
/// or just the plain IP if the origin IP isn't trusted.
pub fn with_ip(
	state: Arc<State>,
) -> impl Filter<Extract = (IpAddr,), Error = warp::Rejection> + Clone {
	warp::filters::addr::remote()
		// Get the originating IP address.
		.and_then(|addr: Option<SocketAddr>| async move {
			addr.map(|socket| socket.ip()).ok_or_else(warp::reject)
		})
		// Get either the `CF-Connecting-IP`, `X-Real-IP`, or the `X-Forwarded-For` header.
		// We map it to an Option so we don't reject if the header isn't present.
		.and(
			warp::header::<IpAddr>("cf-connecting-ip")
				.map(Option::<IpAddr>::Some)
				.or(warp::header::<IpAddr>("x-real-ip").map(Option::<IpAddr>::Some))
				.unify()
				.or(warp::header::<IpAddr>("x-forwarded-for").map(Option::<IpAddr>::Some))
				.unify()
				.or_else(|_| async {
					Result::<_, std::convert::Infallible>::Ok((Option::<IpAddr>::None,))
				}),
		)
		// Copy the config, we need the `trusted_ips` field.
		.and(with_obj(state))
		// Alright, this is where we do our stuff.
		.and_then(
			|origin_ip: IpAddr, header_ip: Option<IpAddr>, state: Arc<State>| async move {
				match header_ip {
					// If the origin is trusted and the IP header is set, use that.
					Some(ip) if state.config.trusted_ips.contains(&origin_ip) => {
						Result::<IpAddr, std::convert::Infallible>::Ok(ip)
					}
					// Otherwise, just use the origin IP.
					_ => Result::<IpAddr, std::convert::Infallible>::Ok(origin_ip),
				}
			},
		)
}
