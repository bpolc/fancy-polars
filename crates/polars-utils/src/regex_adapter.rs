use std::borrow::Cow;

pub use fancy_regex::Regex as FancyRegex;
use fancy_regex::RegexBuilder as FancyRegexBuilder;
#[cfg(feature = "pcre2")]
use pcre2::bytes::CaptureLocations as Pcre2CaptureLocations;
#[cfg(feature = "pcre2")]
pub use pcre2::bytes::{Regex as Pcre2Regex, RegexBuilder as Pcre2RegexBuilder};
use polars_error::{PolarsError, PolarsResult};
pub use regex::Regex;
use regex::{CaptureLocations as RegexCaptureLocations, RegexBuilder};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};

#[cfg(feature = "pcre2")]
use crate::aliases::PlHashMap;

#[derive(AsRefStr, Clone, Copy, Debug, Default, EnumString, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[strum(serialize_all = "snake_case")]
pub enum RegexEngine {
    // `regex` crate
    #[default]
    Regex,
    // `fancy-regex` crate
    Fancy,
    // `pcre2` crate
    #[cfg(feature = "pcre2")]
    Pcre2,
}

pub trait RegexTrait: Sized + Clone {
    fn new(pattern: &str) -> PolarsResult<Self>;

    fn is_match(&self, text: &str) -> bool;

    fn find<'t>(&self, text: &'t str) -> Option<Match<'t>>;

    fn find_iter<'t>(&'t self, text: &'t str) -> Box<dyn Iterator<Item = Match<'t>> + 't>;

    fn replace<'t>(&self, text: &'t str, replacement: &str) -> Cow<'t, str>;

    fn replace_all<'t>(&self, text: &'t str, replacement: &str) -> Cow<'t, str>;

    fn captures<'t>(&self, text: &'t str) -> Option<CaptureGroups<'t>>;

    /// Get an iterator over the capture group names
    fn capture_names(&self) -> CaptureNamesIterator<'_>;

    /// Returns the number of capture groups in the pattern.
    fn captures_len(&self) -> usize;

    fn count_matches(&self, text: &str) -> usize;
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
    Fancy(fancy_regex::CaptureNames<'a>),
    #[cfg(feature = "pcre2")]
    Pcre2(std::slice::Iter<'a, Option<String>>),
}

impl<'a> Iterator for CaptureNamesIterator<'a> {
    type Item = Option<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CaptureNamesIterator::Regex(iter) => iter.next(),
            CaptureNamesIterator::Fancy(iter) => iter.next(),
            #[cfg(feature = "pcre2")]
            CaptureNamesIterator::Pcre2(iter) => iter.next().map(|opt| opt.as_deref()),
        }
    }
}

fn advance_position(text: &str, match_start: usize, match_end: usize) -> usize {
    if match_end == match_start {
        // Zero-width match, advance by one character
        if match_start < text.len() {
            text[match_start..]
                .chars()
                .next()
                .map(|ch| match_start + ch.len_utf8())
                .unwrap_or(text.len() + 1)
        } else {
            text.len() + 1
        }
    } else {
        match_end
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

    fn captures<'t>(&self, text: &'t str) -> Option<CaptureGroups<'t>> {
        self.captures(text)
            .map(|caps| build_capture_groups(caps.len(), |i| caps.get(i).map(|m| m.as_str())))
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

struct FancyFindIterator<'r, 't> {
    re: &'r FancyRegex,
    text: &'t str,
    pos: usize,
    text_len: usize,
}

impl<'r, 't> Iterator for FancyFindIterator<'r, 't> {
    type Item = Match<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos > self.text_len {
            return None;
        }
        match self.re.find_from_pos(self.text, self.pos) {
            Ok(Some(m)) => {
                let start = m.start();
                let end = m.end();

                let result = Match {
                    text: m.as_str(),
                    start,
                    end,
                };

                self.pos = advance_position(self.text, start, end);
                Some(result)
            },
            _ => {
                self.pos = self.text_len + 1;
                None
            },
        }
    }
}

struct FancyCapturesIterator<'r, 't> {
    re: &'r FancyRegex,
    text: &'t str,
    pos: usize,
    text_len: usize,
}

impl<'r, 't> FancyCapturesIterator<'r, 't> {
    fn new(re: &'r FancyRegex, text: &'t str) -> Self {
        Self {
            re,
            text,
            pos: 0,
            text_len: text.len(),
        }
    }
}

impl<'r, 't> Iterator for FancyCapturesIterator<'r, 't> {
    type Item = fancy_regex::Captures<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos > self.text_len {
            return None;
        }

        match self.re.captures_from_pos(self.text, self.pos) {
            Ok(Some(caps)) => {
                let full_match = caps.get(0).unwrap();
                let start = full_match.start();
                let end = full_match.end();

                self.pos = advance_position(self.text, start, end);
                Some(caps)
            },
            _ => {
                self.pos = self.text_len + 1;
                None
            },
        }
    }
}

impl RegexTrait for FancyRegex {
    fn new(pattern: &str) -> PolarsResult<Self> {
        FancyRegexBuilder::new(pattern)
            .case_insensitive(false)
            .multi_line(false)
            .ignore_whitespace(false)
            .dot_matches_new_line(false)
            .verbose_mode(false)
            .unicode_mode(true)
            .build()
            .map_err(|e| PolarsError::ComputeError(e.to_string().into()))
    }

    fn is_match(&self, text: &str) -> bool {
        matches!(self.is_match(text), Ok(true))
    }

    fn find<'t>(&self, text: &'t str) -> Option<Match<'t>> {
        match self.find(text) {
            Ok(Some(m)) => Some(Match {
                start: m.start(),
                end: m.end(),
                text: m.as_str(),
            }),
            _ => None,
        }
    }

    fn find_iter<'t>(&'t self, text: &'t str) -> Box<dyn Iterator<Item = Match<'t>> + 't> {
        Box::new(FancyFindIterator {
            re: self,
            text,
            pos: 0,
            text_len: text.len(),
        })
    }

    fn replace<'t>(&self, text: &'t str, replacement: &str) -> Cow<'t, str> {
        match self.captures(text) {
            Ok(Some(caps)) => {
                let m = caps.get(0).unwrap();
                let mut dest = String::with_capacity(text.len());
                dest.push_str(&text[..m.start()]);
                caps.expand(replacement, &mut dest);
                dest.push_str(&text[m.end()..]);
                Cow::Owned(dest)
            },
            _ => Cow::Borrowed(text),
        }
    }

    fn replace_all<'t>(&self, text: &'t str, replacement: &str) -> Cow<'t, str> {
        let mut dest = String::with_capacity(text.len() + replacement.len() * 4);
        let mut last_match = 0;
        let mut found_match = false;

        for cap_result in FancyCapturesIterator::new(self, text) {
            found_match = true;
            let m = cap_result.get(0).unwrap();
            let start = m.start();
            let end = m.end();
            dest.push_str(&text[last_match..start]);
            cap_result.expand(replacement, &mut dest);
            last_match = end;
        }

        if !found_match {
            return Cow::Borrowed(text);
        }

        dest.push_str(&text[last_match..]);
        Cow::Owned(dest)
    }

    fn captures<'t>(&self, text: &'t str) -> Option<CaptureGroups<'t>> {
        match self.captures(text) {
            Ok(Some(caps)) => Some(build_capture_groups(caps.len(), |i| {
                caps.get(i).map(|m| m.as_str())
            })),
            _ => None,
        }
    }

    fn capture_names(&self) -> CaptureNamesIterator<'_> {
        CaptureNamesIterator::Fancy(fancy_regex::Regex::capture_names(self))
    }

    fn captures_len(&self) -> usize {
        fancy_regex::Regex::captures_len(self)
    }

    fn count_matches(&self, text: &str) -> usize {
        // Avoid building full capture structs when counting.
        let mut pos = 0;
        let mut count = 0;
        let text_len = text.len();
        while pos <= text_len {
            match self.find_from_pos(text, pos) {
                Ok(Some(m)) => {
                    count += 1;
                    pos = advance_position(text, m.start(), m.end());
                },
                _ => break,
            }
        }
        count
    }
}

#[cfg(feature = "pcre2")]
impl RegexTrait for Pcre2Regex {
    fn new(pattern: &str) -> PolarsResult<Self> {
        let mut builder = Pcre2RegexBuilder::new();
        builder
            .caseless(false)
            .dotall(false)
            .extended(false)
            .multi_line(false)
            .crlf(false)
            .ucp(true)
            .utf(true)
            .jit_if_available(true)
            .build(pattern)
            .map_err(|e| PolarsError::ComputeError(e.to_string().into()))
    }

    fn is_match(&self, text: &str) -> bool {
        self.is_match(text.as_bytes()).unwrap_or(false)
    }

    fn find<'t>(&self, text: &'t str) -> Option<Match<'t>> {
        self.find(text.as_bytes()).ok().flatten().map(|m| Match {
            start: m.start(),
            end: m.end(),
            text: &text[m.start()..m.end()],
        })
    }

    fn find_iter<'t>(&'t self, text: &'t str) -> Box<dyn Iterator<Item = Match<'t>> + 't> {
        Box::new(
            self.find_iter(text.as_bytes())
                .filter_map(move |m| m.ok())
                .map(move |m| Match {
                    start: m.start(),
                    end: m.end(),
                    text: &text[m.start()..m.end()],
                }),
        )
    }

    fn replace<'t>(&self, text: &'t str, replacement: &str) -> Cow<'t, str> {
        let replacer = build_pcre2_replacer(self, replacement);
        pcre2_replace_impl(self, text, replacement, &replacer, false)
    }

    fn replace_all<'t>(&self, text: &'t str, replacement: &str) -> Cow<'t, str> {
        let replacer = build_pcre2_replacer(self, replacement);
        pcre2_replace_impl(self, text, replacement, &replacer, true)
    }

    fn captures<'t>(&self, text: &'t str) -> Option<CaptureGroups<'t>> {
        self.captures(text.as_bytes()).ok().flatten().map(|caps| {
            build_capture_groups(caps.len(), |i| {
                caps.get(i).map(|m| &text[m.start()..m.end()])
            })
        })
    }

    fn capture_names(&self) -> CaptureNamesIterator<'_> {
        CaptureNamesIterator::Pcre2(pcre2::bytes::Regex::capture_names(self).iter())
    }

    fn captures_len(&self) -> usize {
        pcre2::bytes::Regex::captures_len(self)
    }

    fn count_matches(&self, text: &str) -> usize {
        // Use the byte iterator directly and count successful matches, avoiding wrapper mapping.
        self.find_iter(text.as_bytes())
            .fold(0usize, |acc, res| acc + usize::from(res.is_ok()))
    }
}

#[cfg(feature = "pcre2")]
fn build_pcre2_replacer<'a>(re: &Pcre2Regex, replacement: &'a str) -> Pcre2Replacer<'a> {
    let names_vec = pcre2::bytes::Regex::capture_names(re);
    let caps_len = pcre2::bytes::Regex::captures_len(re);
    Pcre2Replacer::new(replacement, caps_len, Some(names_vec))
}

#[cfg(feature = "pcre2")]
fn pcre2_replace_impl<'t>(
    re: &Pcre2Regex,
    text: &'t str,
    replacement: &str,
    replacer: &Pcre2Replacer<'_>,
    replace_all: bool,
) -> Cow<'t, str> {
    let mut result =
        String::with_capacity(text.len() + replacement.len() * if replace_all { 4 } else { 1 });
    let mut last_end = 0;
    let mut has_matches = false;

    let mut locs = re.capture_locations();

    while last_end <= text.len() {
        match re.captures_read_at(&mut locs, text.as_bytes(), last_end) {
            Ok(Some(m)) => {
                has_matches = true;
                result.push_str(&text[last_end..m.start()]);
                replacer.expand_into(&mut result, text, &locs);

                let match_start = m.start();
                let match_end = m.end();
                if !replace_all {
                    // For single replace, append the remainder starting from the end of the match.
                    result.push_str(&text[match_end..]);
                    return Cow::Owned(result);
                }
                last_end = advance_position(text, match_start, match_end);
            },
            _ => break,
        }
    }

    if has_matches {
        result.push_str(&text[last_end..]);
        Cow::Owned(result)
    } else {
        Cow::Borrowed(text)
    }
}

#[cfg(feature = "pcre2")]
#[derive(Debug, Clone)]
pub enum Pcre2ReplacementPart<'a> {
    Literal(&'a str),
    Capture(usize),
}

#[cfg(feature = "pcre2")]
#[derive(Debug, Clone)]
pub struct Pcre2Replacer<'a> {
    parts: Vec<Pcre2ReplacementPart<'a>>,
}

#[cfg(feature = "pcre2")]
impl<'a> Pcre2Replacer<'a> {
    fn make_name_index(names_opt: Option<&[Option<String>]>) -> Option<PlHashMap<&'_ str, usize>> {
        names_opt.map(|names| {
            names
                .iter()
                .enumerate()
                .filter_map(|(i, opt)| opt.as_deref().map(|name| (name, i)))
                .collect()
        })
    }

    fn resolve_capture_index(
        key: &str,
        caps_len: usize,
        names_map: Option<&PlHashMap<&str, usize>>,
    ) -> Option<usize> {
        if let Ok(idx) = key.parse::<usize>() {
            return (idx < caps_len).then_some(idx);
        }
        names_map
            .and_then(|m| m.get(key).copied())
            .filter(|&idx| idx < caps_len)
    }

    pub fn new(
        replacement: &'a str,
        caps_len: usize,
        names_opt: Option<&[Option<String>]>,
    ) -> Self {
        if !replacement.contains('$') {
            return Self {
                parts: vec![Pcre2ReplacementPart::Literal(replacement)],
            };
        }

        let names_map = Self::make_name_index(names_opt);

        let mut parts = Vec::new();
        let mut last_end = 0;
        let mut chars = replacement.char_indices().peekable();

        while let Some((i, ch)) = chars.next() {
            if ch == '$' {
                if i > last_end {
                    parts.push(Pcre2ReplacementPart::Literal(&replacement[last_end..i]));
                }
                // we consumed '$'
                last_end = i + 1;

                match chars.peek() {
                    Some(&(_, '$')) => {
                        parts.push(Pcre2ReplacementPart::Literal("$"));
                        chars.next(); // consume second '$'
                        last_end += 1;
                    },
                    Some(&(_, '{')) => {
                        chars.next(); // consume '{'
                        let mut name = String::new();
                        let mut end_index = i + 2; // after "${"
                        while let Some(&(j, c)) = chars.peek() {
                            end_index = j;
                            if c == '}' {
                                // consume '}'
                                chars.next();
                                end_index += 1; // position after '}'
                                break;
                            }
                            name.push(c);
                            chars.next();
                        }
                        let mut pushed = false;
                        if let Some(idx) =
                            Self::resolve_capture_index(&name, caps_len, names_map.as_ref())
                        {
                            parts.push(Pcre2ReplacementPart::Capture(idx));
                            pushed = true;
                        }
                        // advance last_end to after closing brace if we saw one
                        last_end = end_index;
                        if !pushed {
                            // if not resolved, treat token as empty (matches current numeric behavior)
                        }
                    },
                    Some(&(_, c)) if c.is_ascii_digit() => {
                        // $1, $12, etc
                        let mut num_str = String::new();
                        // consume first digit
                        num_str.push(c);
                        chars.next();
                        while let Some(&(_, next_c)) = chars.peek() {
                            if next_c.is_ascii_digit() {
                                num_str.push(next_c);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if let Some(idx) =
                            Self::resolve_capture_index(&num_str, caps_len, names_map.as_ref())
                        {
                            parts.push(Pcre2ReplacementPart::Capture(idx));
                        }
                        last_end = i + 1 + num_str.len();
                    },
                    Some(&(_, c)) if c.is_ascii_alphabetic() || c == '_' => {
                        // $name syntax: consume word chars [A-Za-z0-9_]*
                        let mut name = String::new();
                        // don't consume peeked char yet; read within loop
                        while let Some(&(_, next_c)) = chars.peek() {
                            if next_c.is_ascii_alphanumeric() || next_c == '_' {
                                name.push(next_c);
                                chars.next();
                            } else {
                                break;
                            }
                        }

                        if !name.is_empty() {
                            if let Some(idx) =
                                Self::resolve_capture_index(&name, caps_len, names_map.as_ref())
                            {
                                parts.push(Pcre2ReplacementPart::Capture(idx));
                            }
                            // advance beyond $name
                            last_end = i + 1 + name.len();
                        } else {
                            // treat as literal '$'
                            parts.push(Pcre2ReplacementPart::Literal("$"));
                        }
                    },
                    _ => {
                        // not a recognized token, treat as literal '$'
                        parts.push(Pcre2ReplacementPart::Literal("$"));
                    },
                }
            }
        }

        if last_end < replacement.len() {
            parts.push(Pcre2ReplacementPart::Literal(&replacement[last_end..]));
        }

        Self { parts }
    }

    pub fn expand_into(
        &self,
        result: &mut String,
        text: &str,
        locs: &pcre2::bytes::CaptureLocations,
    ) {
        for part in &self.parts {
            match part {
                Pcre2ReplacementPart::Literal(s) => result.push_str(s),
                Pcre2ReplacementPart::Capture(idx) => {
                    if let Some((start, end)) = locs.get(*idx) {
                        result.push_str(&text[start..end]);
                    }
                },
            }
        }
    }
}

macro_rules! dispatch {
    ($self:expr, $method:ident $(, $args:expr)*) => {
        match $self {
            RegexAdapter::Regex(re) => <Regex as RegexTrait>::$method(re, $($args),*),
            RegexAdapter::Fancy(re) => <FancyRegex as RegexTrait>::$method(re, $($args),*),
            #[cfg(feature = "pcre2")]
            RegexAdapter::Pcre2(re) => <Pcre2Regex as RegexTrait>::$method(re, $($args),*),
        }
    }
}

#[derive(Clone)]
pub enum RegexAdapter {
    Regex(Regex),
    Fancy(FancyRegex),
    #[cfg(feature = "pcre2")]
    Pcre2(Pcre2Regex),
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

    pub fn captures<'t>(&self, text: &'t str) -> Option<CaptureGroups<'t>> {
        dispatch!(self, captures, text)
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
    #[cfg(feature = "pcre2")]
    Pcre2(Pcre2CaptureLocations),
}

impl CaptureLocationsBuffer {
    fn regex_locations_mut(&mut self) -> Option<&mut RegexCaptureLocations> {
        match self.locations {
            Some(LocationsInner::Regex(ref mut locs)) => Some(locs),
            _ => None,
        }
    }
    #[cfg(feature = "pcre2")]
    fn pcre2_locations_mut(&mut self) -> Option<&mut Pcre2CaptureLocations> {
        match self.locations {
            Some(LocationsInner::Pcre2(ref mut locs)) => Some(locs),
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
            RegexAdapter::Fancy(_) => CaptureLocationsBuffer::default(),
            #[cfg(feature = "pcre2")]
            RegexAdapter::Pcre2(re) => CaptureLocationsBuffer {
                locations: Some(LocationsInner::Pcre2(re.capture_locations())),
            },
        }
    }

    pub fn captures_with_buffer<'t, 'b>(
        &'t self,
        text: &'t str,
        buffer: &'b mut CaptureLocationsBuffer,
    ) -> Option<FastCaptureResult<'t, 'b>> {
        match self {
            RegexAdapter::Regex(re) => buffer.regex_locations_mut().and_then(|locs| {
                if re.captures_read(locs, text).is_some() {
                    Some(FastCaptureResult::RegexLocations(text, locs))
                } else {
                    None
                }
            }),
            RegexAdapter::Fancy(re) => match re.captures(text) {
                Ok(Some(caps)) => Some(FastCaptureResult::FancyCaps(caps)),
                _ => None,
            },
            #[cfg(feature = "pcre2")]
            RegexAdapter::Pcre2(re) => buffer.pcre2_locations_mut().and_then(|locs| {
                if re
                    .captures_read_at(locs, text.as_bytes(), 0)
                    .ok()
                    .flatten()
                    .is_some()
                {
                    Some(FastCaptureResult::Pcre2Locations(text, locs))
                } else {
                    None
                }
            }),
        }
    }

    pub fn get_group_with_buffer<'t>(
        &'t self,
        text: &'t str,
        group_index: usize,
        buffer: &mut CaptureLocationsBuffer,
    ) -> Option<&'t str> {
        match self {
            RegexAdapter::Regex(re) => buffer.regex_locations_mut().and_then(|locs| {
                if re.captures_read(locs, text).is_some() {
                    locs.get(group_index).map(|(start, end)| &text[start..end])
                } else {
                    None
                }
            }),
            RegexAdapter::Fancy(re) => match re.captures(text) {
                Ok(Some(caps)) => caps.get(group_index).map(|m| m.as_str()),
                _ => None,
            },
            #[cfg(feature = "pcre2")]
            RegexAdapter::Pcre2(re) => buffer.pcre2_locations_mut().and_then(|locs| {
                if re
                    .captures_read_at(locs, text.as_bytes(), 0)
                    .ok()
                    .flatten()
                    .is_some()
                {
                    locs.get(group_index).map(|(start, end)| &text[start..end])
                } else {
                    None
                }
            }),
        }
    }
}

pub enum FastCaptureResult<'t, 'b> {
    RegexLocations(&'t str, &'b regex::CaptureLocations),
    FancyCaps(fancy_regex::Captures<'t>),
    #[cfg(feature = "pcre2")]
    Pcre2Locations(&'t str, &'b pcre2::bytes::CaptureLocations),
}

impl<'t, 'b> FastCaptureResult<'t, 'b> {
    /// Get a capture group by index
    pub fn get(&self, index: usize) -> Option<&'t str> {
        match self {
            FastCaptureResult::RegexLocations(text, locs) => {
                locs.get(index).map(|(start, end)| &text[start..end])
            },
            FastCaptureResult::FancyCaps(caps) => caps.get(index).map(|m| m.as_str()),
            #[cfg(feature = "pcre2")]
            FastCaptureResult::Pcre2Locations(text, locs) => {
                locs.get(index).map(|(start, end)| &text[start..end])
            },
        }
    }
}
