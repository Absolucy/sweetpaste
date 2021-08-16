/*
	Copyright (c) 2021 Lucy <lucy@absolucy.moe>

	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("warp error: {0}")]
	Warp(#[from] warp::Error),
	#[error("invalid utf8: {0}")]
	InvalidUtf8(#[from] std::string::FromUtf8Error),
	#[error("database error: {0}")]
	Db(#[from] sqlx::Error),
	#[error("id error: {0}")]
	Mnemonic(#[from] mnemonic::Error),
	#[error("invalid id")]
	InvalidId,
	#[error("form is missing '{0}' entry")]
	IncompleteForm(&'static str),
	#[error("didn't upload any paste")]
	EmptyForm,
	#[error("failed to render: {0}")]
	Render(#[from] handlebars::RenderError),
}

impl warp::reject::Reject for Error {}
