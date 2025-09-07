use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use polars_error::PolarsResult;

use crate::cache::LruCache;
#[cfg(feature = "pcre2")]
use crate::regex_adapter::Pcre2Regex;
pub use crate::regex_adapter::RegexEngine;
use crate::regex_adapter::{FancyRegex, Regex, RegexAdapter, RegexTrait};

#[derive(Hash, Eq, PartialEq, Clone)]
struct RegexKey {
    engine: RegexEngine,
    pattern: String,
}

impl RegexKey {
    #[inline]
    fn new(pattern: &str, engine: RegexEngine) -> Self {
        Self {
            engine,
            pattern: pattern.to_owned(),
        }
    }
}

struct GlobalRegexEntry {
    compiled: Arc<RegexAdapter>,
    created_at: Instant,
}

/// Thread-local cache storing weak references to globally cached regexes.
/// This avoids duplicating the heavy compiled structures in each thread.
pub struct RegexCache {
    cache: LruCache<RegexKey, Weak<RegexAdapter>>,
    capacity: usize,
}

impl RegexCache {
    fn new() -> Self {
        let capacity = current_local_capacity();
        Self {
            cache: LruCache::with_capacity(capacity),
            capacity,
        }
    }

    fn ensure_local_capacity_uptodate(&mut self) {
        let new_capacity = current_local_capacity();
        if new_capacity == self.capacity {
            return;
        }
        self.cache.resize(new_capacity);
        self.capacity = new_capacity;
    }

    #[inline]
    pub fn compile(&mut self, re_str: &str, engine: RegexEngine) -> PolarsResult<RegexAdapter> {
        let key = RegexKey::new(re_str, engine);

        // Local weak-ref cache
        if let Some(shared) = self.cache.get(&key).and_then(|weak| weak.upgrade()) {
            return Ok(shared.as_ref().clone());
        }

        // Global LRU with TTL
        if let Some(shared) = try_get_from_global(&key) {
            self.ensure_local_capacity_uptodate();
            self.cache.insert(key.clone(), Arc::downgrade(&shared));
            return Ok(shared.as_ref().clone());
        }

        // Compile fresh (outside global lock), then publish to global and local
        let compiled = match engine {
            RegexEngine::Regex => {
                let re = <Regex as RegexTrait>::new(re_str)?;
                RegexAdapter::Regex(re)
            },
            RegexEngine::Fancy => {
                let re = <FancyRegex as RegexTrait>::new(re_str)?;
                RegexAdapter::Fancy(re)
            },
            #[cfg(feature = "pcre2")]
            RegexEngine::Pcre2 => {
                let re = <Pcre2Regex as RegexTrait>::new(re_str)?;
                RegexAdapter::Pcre2(re)
            },
        };

        // Another thread may have compiled concurrently; try global again to avoid duplicates
        if let Some(shared) = try_get_from_global(&key) {
            self.ensure_local_capacity_uptodate();
            self.cache.insert(key.clone(), Arc::downgrade(&shared));
            return Ok(shared.as_ref().clone());
        }

        let shared = Arc::new(compiled);
        insert_into_global(key.clone(), shared.clone());
        self.ensure_local_capacity_uptodate();
        self.cache.insert(key, Arc::downgrade(&shared));
        Ok(shared.as_ref().clone())
    }
}

thread_local! {
    static LOCAL_REGEX_CACHE: RefCell<RegexCache> = RefCell::new(RegexCache::new());
}

const DEFAULT_TTL_MS: usize = 10 * 60 * 1000;
const DEFAULT_GLOBAL_CAPACITY: usize = 256;
const DEFAULT_LOCAL_CAPACITY: usize = 64;

static TTL_MS: AtomicUsize = AtomicUsize::new(DEFAULT_TTL_MS);
static GLOBAL_CAPACITY: AtomicUsize = AtomicUsize::new(DEFAULT_GLOBAL_CAPACITY);
static LOCAL_CAPACITY: AtomicUsize = AtomicUsize::new(DEFAULT_LOCAL_CAPACITY);

static GLOBAL_REGEX_CACHE: std::sync::LazyLock<Mutex<LruCache<RegexKey, GlobalRegexEntry>>> =
    std::sync::LazyLock::new(|| Mutex::new(LruCache::with_capacity(current_global_capacity())));

#[inline]
fn current_ttl() -> Duration {
    Duration::from_millis(TTL_MS.load(Ordering::Relaxed) as u64)
}

#[inline]
fn current_global_capacity() -> usize {
    GLOBAL_CAPACITY.load(Ordering::Relaxed).max(1)
}

#[inline]
fn current_local_capacity() -> usize {
    LOCAL_CAPACITY.load(Ordering::Relaxed).max(1)
}

#[inline]
fn now_instant() -> Instant {
    Instant::now()
}

#[inline]
fn is_expired(created_at: Instant) -> bool {
    created_at.elapsed() >= current_ttl()
}

fn try_get_from_global(key: &RegexKey) -> Option<Arc<RegexAdapter>> {
    let mut guard = GLOBAL_REGEX_CACHE.lock().unwrap();
    if let Some(entry) = guard.get(key) {
        if is_expired(entry.created_at) {
            // Evict expired entries proactively to keep the cache tidy.
            guard.remove(key);
            return None;
        }
        return Some(Arc::clone(&entry.compiled));
    }
    None
}

fn insert_into_global(key: RegexKey, value: Arc<RegexAdapter>) {
    let mut guard = GLOBAL_REGEX_CACHE.lock().unwrap();
    let entry = GlobalRegexEntry {
        compiled: value,
        created_at: now_instant(),
    };
    guard.insert(key, entry);
}

pub fn get_regex_cache_ttl_ms() -> usize {
    TTL_MS.load(Ordering::Relaxed)
}

pub fn set_regex_cache_ttl_ms(ttl_ms: Option<usize>) {
    let new_ttl_ms = ttl_ms.unwrap_or(DEFAULT_TTL_MS).max(1);
    let current = TTL_MS.load(Ordering::Relaxed);
    if current == new_ttl_ms {
        return;
    }
    TTL_MS.store(new_ttl_ms, Ordering::Relaxed);
}

pub fn get_global_regex_cache_capacity() -> usize {
    GLOBAL_CAPACITY.load(Ordering::Relaxed)
}

pub fn set_global_regex_cache_capacity(capacity: Option<usize>) {
    let new_capacity = capacity.unwrap_or(DEFAULT_GLOBAL_CAPACITY).max(1);
    let current = GLOBAL_CAPACITY.load(Ordering::Relaxed);
    if current == new_capacity {
        return;
    }
    GLOBAL_CAPACITY.store(new_capacity, Ordering::Relaxed);

    // Resize the global cache with the new capacity.
    let mut guard = GLOBAL_REGEX_CACHE.lock().unwrap();
    guard.resize(new_capacity);
}

pub fn get_local_regex_cache_capacity() -> usize {
    LOCAL_CAPACITY.load(Ordering::Relaxed)
}

pub fn set_local_regex_cache_capacity(capacity: Option<usize>) {
    let new_capacity = capacity.unwrap_or(DEFAULT_LOCAL_CAPACITY).max(1);
    let current = LOCAL_CAPACITY.load(Ordering::Relaxed);
    if current == new_capacity {
        return;
    }
    LOCAL_CAPACITY.store(new_capacity, Ordering::Relaxed);
    // Reconfigure the current thread's local cache immediately; others will update lazily.
    LOCAL_REGEX_CACHE.with(|cache| cache.borrow_mut().ensure_local_capacity_uptodate());
}

pub fn compile_regex(re: &str, engine: RegexEngine) -> PolarsResult<RegexAdapter> {
    LOCAL_REGEX_CACHE.with(|cache| cache.borrow_mut().compile(re, engine))
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
