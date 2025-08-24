use std::cell::RefCell;

use polars_error::{PolarsError, PolarsResult};

use crate::cache::LruCache;
pub use crate::regex_adapter::RegexEngine;
use crate::regex_adapter::{FancyRegex, Regex, RegexAdapter};

// Regex compilation is really heavy, and the resulting regexes can be large as
// well, so we should have a good caching scheme.
//
// TODO: add larger global cache which has time-based flush.

/// A multi-engine cache for runtime-compiled regular expressions
pub struct RegexCache {
    regex_cache: LruCache<String, Regex>,
    fancy_cache: LruCache<String, FancyRegex>,
}

impl RegexCache {
    fn new() -> Self {
        Self {
            regex_cache: LruCache::with_capacity(32),
            fancy_cache: LruCache::with_capacity(32),
        }
    }

    pub fn compile_regex(&mut self, re: &str) -> Result<&Regex, regex::Error> {
        let r = self.regex_cache.try_get_or_insert_with(re, |re| {
            #[allow(clippy::disallowed_methods)]
            Regex::new(re)
        });
        Ok(&*r?)
    }

    pub fn compile_fancy(&mut self, re: &str) -> Result<&FancyRegex, Box<fancy_regex::Error>> {
        let r = self
            .fancy_cache
            .try_get_or_insert_with(re, |re| FancyRegex::new(re).map_err(Box::new));
        Ok(&*r?)
    }

    pub fn compile_adapter(
        &mut self,
        re_str: &str,
        engine: RegexEngine,
    ) -> PolarsResult<RegexAdapter> {
        match engine {
            RegexEngine::Regex => self
                .compile_regex(re_str)
                .map(|re| RegexAdapter::Regex(re.clone()))
                .map_err(|e| PolarsError::ComputeError(e.to_string().into())),
            RegexEngine::Fancy => self
                .compile_fancy(re_str)
                .map(|re| RegexAdapter::Fancy(re.clone()))
                .map_err(|e| PolarsError::ComputeError(e.to_string().into())),
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
