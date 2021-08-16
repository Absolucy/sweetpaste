/*
	Copyright (c) 2021 Lucy <lucy@absolucy.moe>

	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::config::Config;
use clru::{CLruCache, CLruCacheConfig, WeightScale};
use fnv::FnvBuildHasher;
use tokio::sync::Mutex;

pub type HtmlCache = Mutex<CLruCache<i64, String, FnvBuildHasher, StringScale>>;

/// A CLru weighting scale, that uses the size of a string in memory as the weight.
pub struct StringScale;

impl WeightScale<i64, String> for StringScale {
	fn weight(&self, _key: &i64, value: &String) -> usize {
		value.len() + std::mem::size_of::<String>()
	}
}

// Initializes the cache, using the given configuration.
pub fn create_cache(config: &Config) -> HtmlCache {
	let config = CLruCacheConfig::new(config.cache_limit)
		// The FNV hasher is used because it's the fastest for 64-bit keys.
		.with_hasher(FnvBuildHasher::default())
		// The StringScale is used to weight the cache by the size of the string.
		.with_scale(StringScale);
	let cache = CLruCache::with_config(config);
	Mutex::new(cache)
}
