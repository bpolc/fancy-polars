use std::cell::RefCell;

use polars_error::PolarsResult;

use crate::cache::LruCache;
#[cfg(feature = "pcre2")]
use crate::regex_adapter::Pcre2Regex;
pub use crate::regex_adapter::RegexEngine;
use crate::regex_adapter::{FancyRegex, Regex, RegexAdapter, RegexTrait};

// Regex compilation is really heavy, and the resulting regexes can be large as
// well, so we should have a good caching scheme.
//
// TODO: add larger global cache which has time-based flush.

/// A multi-engine cache for runtime-compiled regular expressions
pub struct RegexCache {
    regex_cache: LruCache<String, Regex>,
    fancy_cache: LruCache<String, FancyRegex>,
    #[cfg(feature = "pcre2")]
    pcre2_cache: LruCache<String, Pcre2Regex>,
}

impl RegexCache {
    fn new() -> Self {
        Self {
            regex_cache: LruCache::with_capacity(32),
            fancy_cache: LruCache::with_capacity(32),
            #[cfg(feature = "pcre2")]
            pcre2_cache: LruCache::with_capacity(32),
        }
    }

    pub fn compile_adapter(
        &mut self,
        re_str: &str,
        engine: RegexEngine,
    ) -> PolarsResult<RegexAdapter> {
        match engine {
            RegexEngine::Regex => {
                let re = self
                    .regex_cache
                    .try_get_or_insert_with(re_str, <Regex as RegexTrait>::new)?;
                Ok(RegexAdapter::Regex(re.clone()))
            },
            RegexEngine::Fancy => {
                let re = self
                    .fancy_cache
                    .try_get_or_insert_with(re_str, <FancyRegex as RegexTrait>::new)?;
                Ok(RegexAdapter::Fancy(re.clone()))
            },
            #[cfg(feature = "pcre2")]
            RegexEngine::Pcre2 => {
                let re = self
                    .pcre2_cache
                    .try_get_or_insert_with(re_str, <Pcre2Regex as RegexTrait>::new)?;
                Ok(RegexAdapter::Pcre2(re.clone()))
            },
        }
    }
}

thread_local! {
    static LOCAL_REGEX_CACHE: RefCell<RegexCache> = RefCell::new(RegexCache::new());
}

pub fn compile_regex(re: &str, engine: RegexEngine) -> PolarsResult<RegexAdapter> {
    LOCAL_REGEX_CACHE.with_borrow_mut(|cache| cache.compile_adapter(re, engine))
}

pub fn with_regex_cache<R, F: FnOnce(&mut RegexCache) -> R>(f: F) -> R {
    LOCAL_REGEX_CACHE.with_borrow_mut(f)
}

#[macro_export]
macro_rules! cached_regex {
    () => {};

    ($vis:vis static $name:ident = $regex:expr; $($rest:tt)*) => {
        #[allow(clippy::disallowed_methods)]
        $vis static $name: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| regex::Regex::new($regex).unwrap());
        $crate::regex_cache::cached_regex!($($rest)*);
    };
}
pub use cached_regex;
