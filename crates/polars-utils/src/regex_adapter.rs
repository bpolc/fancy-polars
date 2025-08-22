use std::borrow::Cow;

use polars_error::{PolarsError, PolarsResult};
pub use regex::Regex;
use regex::{CaptureLocations as RegexCaptureLocations, NoExpand as RegexNoExpand, RegexBuilder};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};

#[derive(AsRefStr, Clone, Copy, Debug, Default, EnumString, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[strum(serialize_all = "snake_case")]
pub enum RegexEngine {
    // `regex` crate
    #[default]
    Regex,
}

pub trait RegexTrait: Sized + Clone {
    fn new(pattern: &str) -> PolarsResult<Self>;

    fn is_match(&self, text: &str) -> bool;

    fn find<'t>(&self, text: &'t str) -> Option<Match<'t>>;

    fn find_iter<'t>(&'t self, text: &'t str) -> Box<dyn Iterator<Item = Match<'t>> + 't>;

    fn replace<'t>(&self, text: &'t str, replacement: &str) -> Cow<'t, str>;

    fn replace_all<'t>(&self, text: &'t str, replacement: &str) -> Cow<'t, str>;

    fn replace_all_literal<'t>(&self, text: &'t str, replacement: &str) -> Cow<'t, str> {
        replace_all_literal_from_spans(
            text,
            replacement,
            self.find_iter(text).map(|m| (m.start(), m.end())),
        )
    }

    fn captures<'t>(&self, text: &'t str) -> Option<CaptureGroups<'t>>;

    fn captures_iter<'t>(
        &'t self,
        text: &'t str,
    ) -> Box<dyn Iterator<Item = CaptureGroups<'t>> + 't>;

    /// Get an iterator over the capture group names
    fn capture_names(&self) -> CaptureNamesIterator<'_>;

    /// Returns the number of capture groups in the pattern.
    fn captures_len(&self) -> usize;

    fn count_matches(&self, text: &str) -> usize {
        self.find_iter(text).count()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Match<'t> {
    pub text: &'t str,
    pub start: usize,
    pub end: usize,
}

impl<'t> Match<'t> {
    pub fn as_str(&self) -> &'t str {
        self.text
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}

#[derive(Debug, Clone)]
pub struct CaptureGroups<'t> {
    groups: Vec<Option<&'t str>>,
}

impl<'t> CaptureGroups<'t> {
    /// Get the number of capture groups (including group 0)
    pub fn len(&self) -> usize {
        self.groups.len()
    }

    pub fn is_empty(&self) -> bool {
        !self.groups.first().is_some_and(|m| m.is_some())
    }

    /// Get a capture group by index
    pub fn get(&self, index: usize) -> Option<&'t str> {
        self.groups.get(index).copied().flatten()
    }

    /// Get the main match (group 0)
    pub fn get_match(&self) -> Option<&'t str> {
        self.get(0)
    }

    pub fn groups(&self) -> &[Option<&'t str>] {
        &self.groups
    }
}

fn build_capture_groups<'t, F>(len: usize, get_group: F) -> CaptureGroups<'t>
where
    F: FnMut(usize) -> Option<&'t str>,
{
    let groups = (0..len).map(get_group).collect();
    CaptureGroups { groups }
}

pub enum CaptureNamesIterator<'a> {
    Regex(regex::CaptureNames<'a>),
}

impl<'a> Iterator for CaptureNamesIterator<'a> {
    type Item = Option<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CaptureNamesIterator::Regex(iter) => iter.next(),
        }
    }
}

impl RegexTrait for Regex {
    #[allow(clippy::disallowed_methods)]
    fn new(pattern: &str) -> PolarsResult<Self> {
        RegexBuilder::new(pattern)
            .unicode(true)
            .case_insensitive(false)
            .multi_line(false)
            .dot_matches_new_line(false)
            .crlf(false)
            .swap_greed(false)
            .ignore_whitespace(false)
            .octal(false)
            .build()
            .map_err(|e| PolarsError::ComputeError(e.to_string().into()))
    }

    fn is_match(&self, text: &str) -> bool {
        self.is_match(text)
    }

    fn find<'t>(&self, text: &'t str) -> Option<Match<'t>> {
        self.find(text).map(|m| Match {
            text: m.as_str(),
            start: m.start(),
            end: m.end(),
        })
    }

    fn find_iter<'t>(&'t self, text: &'t str) -> Box<dyn Iterator<Item = Match<'t>> + 't> {
        Box::new(self.find_iter(text).map(|m| Match {
            text: m.as_str(),
            start: m.start(),
            end: m.end(),
        }))
    }

    fn replace<'t>(&self, text: &'t str, replacement: &str) -> Cow<'t, str> {
        self.replace(text, replacement)
    }

    fn replace_all<'t>(&self, text: &'t str, replacement: &str) -> Cow<'t, str> {
        self.replace_all(text, replacement)
    }

    fn replace_all_literal<'t>(&self, text: &'t str, replacement: &str) -> Cow<'t, str> {
        self.replace_all(text, RegexNoExpand(replacement))
    }

    fn captures<'t>(&self, text: &'t str) -> Option<CaptureGroups<'t>> {
        self.captures(text)
            .map(|caps| build_capture_groups(caps.len(), |i| caps.get(i).map(|m| m.as_str())))
    }

    fn captures_iter<'t>(
        &'t self,
        text: &'t str,
    ) -> Box<dyn Iterator<Item = CaptureGroups<'t>> + 't> {
        Box::new(
            self.captures_iter(text)
                .map(|caps| build_capture_groups(caps.len(), |i| caps.get(i).map(|m| m.as_str()))),
        )
    }

    fn capture_names(&self) -> CaptureNamesIterator<'_> {
        CaptureNamesIterator::Regex(regex::Regex::capture_names(self))
    }

    fn captures_len(&self) -> usize {
        Regex::captures_len(self)
    }

    fn count_matches(&self, text: &str) -> usize {
        regex::Regex::find_iter(self, text).count()
    }
}

/// Helper to build a literal replacement result from an iterator of match spans.
/// When a zero-width match occurs at the end of the string, advance beyond the end to signal termination.
fn replace_all_literal_from_spans<'t, I>(text: &'t str, replacement: &str, spans: I) -> Cow<'t, str>
where
    I: IntoIterator<Item = (usize, usize)>,
{
    let mut result = String::with_capacity(text.len() + replacement.len() * 2);
    let mut last_end = 0;
    let mut has_matches = false;

    for (start, end) in spans {
        has_matches = true;
        result.push_str(&text[last_end..start]);
        result.push_str(replacement);
        last_end = end;
    }

    if has_matches {
        result.push_str(&text[last_end..]);
        Cow::Owned(result)
    } else {
        Cow::Borrowed(text)
    }
}

macro_rules! dispatch {
    ($self:expr, $method:ident $(, $args:expr)*) => {
        match $self {
            RegexAdapter::Regex(re) => <Regex as RegexTrait>::$method(re, $($args),*),
        }
    }
}

#[derive(Clone)]
pub enum RegexAdapter {
    Regex(Regex),
}

impl RegexAdapter {
    pub fn is_match(&self, text: &str) -> bool {
        dispatch!(self, is_match, text)
    }

    pub fn find<'t>(&self, text: &'t str) -> Option<Match<'t>> {
        dispatch!(self, find, text)
    }

    pub fn find_iter<'t>(&'t self, text: &'t str) -> Box<dyn Iterator<Item = Match<'t>> + 't> {
        dispatch!(self, find_iter, text)
    }

    pub fn replace<'t>(&self, text: &'t str, replacement: &str) -> std::borrow::Cow<'t, str> {
        dispatch!(self, replace, text, replacement)
    }

    pub fn replace_all<'t>(&self, text: &'t str, replacement: &str) -> std::borrow::Cow<'t, str> {
        dispatch!(self, replace_all, text, replacement)
    }

    pub fn replace_all_literal<'t>(
        &self,
        text: &'t str,
        replacement: &str,
    ) -> std::borrow::Cow<'t, str> {
        dispatch!(self, replace_all_literal, text, replacement)
    }

    pub fn captures<'t>(&self, text: &'t str) -> Option<CaptureGroups<'t>> {
        dispatch!(self, captures, text)
    }

    pub fn captures_iter<'t>(
        &'t self,
        text: &'t str,
    ) -> Box<dyn Iterator<Item = CaptureGroups<'t>> + 't> {
        dispatch!(self, captures_iter, text)
    }

    pub fn capture_names(&self) -> CaptureNamesIterator<'_> {
        dispatch!(self, capture_names)
    }

    pub fn captures_len(&self) -> usize {
        dispatch!(self, captures_len)
    }

    pub fn count_matches(&self, text: &str) -> usize {
        dispatch!(self, count_matches, text)
    }
}

#[derive(Default)]
pub struct CaptureLocationsBuffer {
    locations: Option<LocationsInner>,
}

enum LocationsInner {
    Regex(RegexCaptureLocations),
}

impl CaptureLocationsBuffer {
    fn regex_locations_mut(&mut self) -> Option<&mut RegexCaptureLocations> {
        match self.locations {
            Some(LocationsInner::Regex(ref mut locs)) => Some(locs),
            _ => None,
        }
    }
}

impl RegexAdapter {
    pub fn create_locations_buffer(&self) -> CaptureLocationsBuffer {
        match self {
            RegexAdapter::Regex(re) => CaptureLocationsBuffer {
                locations: Some(LocationsInner::Regex(re.capture_locations())),
            },
        }
    }

    pub fn captures_with_buffer<'t, 'b>(
        &'t self,
        text: &'t str,
        buffer: &'b mut CaptureLocationsBuffer,
    ) -> Option<FastCaptureResult<'t, 'b>> {
        match self {
            RegexAdapter::Regex(re) => {
                if let Some(locs) = buffer.regex_locations_mut() {
                    if re.captures_read(locs, text).is_some() {
                        return Some(FastCaptureResult::RegexLocations(text, locs));
                    }
                }
                None
            },
        }
    }

    pub fn get_group_with_buffer<'t>(
        &'t self,
        text: &'t str,
        group_index: usize,
        buffer: &mut CaptureLocationsBuffer,
    ) -> Option<&'t str> {
        match self {
            RegexAdapter::Regex(re) => {
                if let Some(locs) = buffer.regex_locations_mut() {
                    if re.captures_read(locs, text).is_some() {
                        return locs.get(group_index).map(|(start, end)| &text[start..end]);
                    }
                }
                None
            },
        }
    }
}

pub enum FastCaptureResult<'t, 'b> {
    RegexLocations(&'t str, &'b regex::CaptureLocations),
}

impl<'t, 'b> FastCaptureResult<'t, 'b> {
    /// Get a capture group by index
    pub fn get(&self, index: usize) -> Option<&'t str> {
        match self {
            FastCaptureResult::RegexLocations(text, locs) => {
                locs.get(index).map(|(start, end)| &text[start..end])
            },
        }
    }
}
