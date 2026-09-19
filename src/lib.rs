//! Deterministic English word aliases for UUIDs. Labels are lossy, not unique IDs.
//!
//! ```
//! use readable_uuid::{ReadableUuid, WordSet, Uuid};
//! let formatter = ReadableUuid::builder()
//!     .word_set(WordSet::ShortV1)
//!     .words(6)
//!     .build()?;
//! assert_eq!(formatter.format(&Uuid::nil()).split('-').count(), 6);
//! # Ok::<(), readable_uuid::Error>(())
//! ```
use std::{collections::HashSet, fmt};
pub use uuid::Uuid;

mod limits;
pub use limits::{MAX_DICTIONARY_WORDS, MAX_WORD_BYTES};

mod lists {
    include!(concat!(env!("OUT_DIR"), "/lists.rs"));
}

/// Version of the complete conversion protocol: domain, UUID byte order, XOF
/// consumption, little-endian candidates and rejection sampling.
pub const ALGORITHM: &str = "blake3-v1";

/// Resource limit, not an entropy guarantee.
pub const MAX_WORDS: usize = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum WordSet {
    #[default]
    EnglishV1,
    ShortV1,
    NatureV1,
}

impl WordSet {
    /// Available dictionaries. New sets may be added without changing this type.
    pub const ALL: &'static [Self] = &[Self::EnglishV1, Self::ShortV1, Self::NatureV1];

    pub const fn name(self) -> &'static str {
        match self {
            Self::EnglishV1 => "english-v1",
            Self::ShortV1 => "short-v1",
            Self::NatureV1 => "nature-v1",
        }
    }

    pub const fn words(self) -> &'static [&'static str] {
        match self {
            Self::EnglishV1 => lists::ENGLISH,
            Self::ShortV1 => lists::SHORT,
            Self::NatureV1 => lists::NATURE,
        }
    }

    pub const fn max_word_len(self) -> usize {
        match self {
            Self::EnglishV1 => lists::ENGLISH_MAX,
            Self::ShortV1 => lists::SHORT_MAX,
            Self::NatureV1 => lists::NATURE_MAX,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    InvalidWordCount,
    InvalidDictionarySize,
    InvalidWord(usize),
    WordTooLong(usize),
    DuplicateWord(usize),
    InvalidSeparator,
    InvalidUuid(uuid::Error),
    OutputTooLong,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWordCount => write!(f, "word count must be 1..={MAX_WORDS}"),
            Self::InvalidDictionarySize => {
                write!(
                    f,
                    "dictionary must contain 2..={MAX_DICTIONARY_WORDS} words"
                )
            }
            Self::InvalidWord(i) => write!(f, "word {i} must contain only lowercase ASCII letters"),
            Self::WordTooLong(index) => write!(f, "word {index} exceeds {MAX_WORD_BYTES} bytes"),
            Self::DuplicateWord(i) => write!(f, "duplicate word at index {i}"),
            Self::InvalidSeparator => {
                f.write_str("separator must be 1..=32 ASCII punctuation or space bytes")
            }
            Self::InvalidUuid(e) => e.fmt(f),
            Self::OutputTooLong => f.write_str("output length exceeds the supported capacity"),
        }
    }
}

impl std::error::Error for Error {}

/// Validated reusable formatter. Custom dictionaries are borrowed without copying.
#[derive(Debug, Clone)]
pub struct ReadableUuid<'a> {
    dictionary: &'a [&'a str],
    words: usize,
    separator: &'a str,
    max_len: usize,
}

#[derive(Debug, Clone)]
pub struct Builder<'a> {
    dictionary: &'a [&'a str],
    words: usize,
    separator: &'a str,
    custom: bool,
    longest: Option<usize>,
}

impl Default for ReadableUuid<'static> {
    fn default() -> Self {
        Self::builder().build().expect("valid defaults")
    }
}

impl<'a> ReadableUuid<'a> {
    pub fn builder() -> Builder<'a> {
        Builder {
            dictionary: WordSet::EnglishV1.words(),
            words: 4,
            separator: "-",
            custom: false,
            longest: Some(WordSet::EnglishV1.max_word_len()),
        }
    }

    pub fn format(&self, id: &Uuid) -> String {
        let mut output = String::with_capacity(self.max_len);
        self.write_into(id, &mut output);
        output
    }

    pub fn format_str(&self, id: &str) -> Result<String, Error> {
        Ok(self.format(&Uuid::try_parse(id).map_err(Error::InvalidUuid)?))
    }

    /// Appends a label; clear the buffer first to replace its contents.
    pub fn write_into(&self, id: &Uuid, output: &mut String) {
        output.reserve(self.max_len);
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"readable-uuid/blake3-v1\0");
        hasher.update(id.as_bytes());
        let mut stream = hasher.finalize_xof();
        let dictionary_size = self.dictionary.len() as u64;
        let mut block = [0_u8; 64];
        let mut offset = block.len();
        for position in 0..self.words {
            let index = sample_index(dictionary_size, || {
                if offset == block.len() {
                    stream.fill(&mut block);
                    offset = 0;
                }
                let value = u32::from_le_bytes(block[offset..offset + 4].try_into().unwrap());
                offset += 4;
                value
            });
            if position != 0 {
                output.push_str(self.separator);
            }
            output.push_str(self.dictionary[index]);
        }
    }

    pub fn dictionary_len(&self) -> usize {
        self.dictionary.len()
    }

    /// Log2 of word-sequence space, not input entropy or guaranteed uniqueness.
    pub fn combination_bits(&self) -> f64 {
        self.words as f64 * (self.dictionary.len() as f64).log2()
    }
}

impl<'a> Builder<'a> {
    pub fn word_set(mut self, set: WordSet) -> Self {
        self.dictionary = set.words();
        self.custom = false;
        self.longest = Some(set.max_word_len());
        self
    }

    /// Borrows an ordered dictionary, validated once by [`Self::build`].
    ///
    /// Entries must be unique lowercase ASCII words of 1..=64 bytes. The caller
    /// owns the list and must preserve its contents and order for stable labels.
    pub fn custom_words(mut self, words: &'a [&'a str]) -> Self {
        self.dictionary = words;
        self.custom = true;
        self.longest = None;
        self
    }

    pub fn words(mut self, words: usize) -> Self {
        self.words = words;
        self
    }

    pub fn separator(mut self, separator: &'a str) -> Self {
        self.separator = separator;
        self
    }

    pub fn build(self) -> Result<ReadableUuid<'a>, Error> {
        if !(1..=MAX_WORDS).contains(&self.words) {
            return Err(Error::InvalidWordCount);
        }
        if !(2..=MAX_DICTIONARY_WORDS).contains(&self.dictionary.len()) {
            return Err(Error::InvalidDictionarySize);
        }
        if self.separator.is_empty()
            || self.separator.len() > 32
            || !self
                .separator
                .bytes()
                .all(|b| b.is_ascii_punctuation() || b == b' ')
        {
            return Err(Error::InvalidSeparator);
        }
        if self.custom {
            let mut seen = HashSet::with_capacity(self.dictionary.len());
            for (i, word) in self.dictionary.iter().enumerate() {
                if word.len() > MAX_WORD_BYTES {
                    return Err(Error::WordTooLong(i));
                }
                if word.is_empty() || !word.bytes().all(|b| b.is_ascii_lowercase()) {
                    return Err(Error::InvalidWord(i));
                }
                if !seen.insert(word) {
                    return Err(Error::DuplicateWord(i));
                }
            }
        }
        let longest = self
            .longest
            .unwrap_or_else(|| self.dictionary.iter().map(|w| w.len()).max().unwrap());
        let max_len = longest
            .checked_mul(self.words)
            .and_then(|v| v.checked_add(self.separator.len() * (self.words - 1)))
            .ok_or(Error::OutputTooLong)?;
        Ok(ReadableUuid {
            dictionary: self.dictionary,
            words: self.words,
            separator: self.separator,
            max_len,
        })
    }
}

pub fn readable_uuid(id: &Uuid) -> String {
    static DEFAULT: std::sync::LazyLock<ReadableUuid<'static>> =
        std::sync::LazyLock::new(ReadableUuid::default);
    DEFAULT.format(id)
}

fn sample_index(dictionary_size: u64, mut next: impl FnMut() -> u32) -> usize {
    let acceptance_limit = (1_u64 << 32) / dictionary_size * dictionary_size;
    loop {
        let value = u64::from(next());
        if value < acceptance_limit {
            return (value % dictionary_size) as usize;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::sample_index;
    #[test]
    fn rejection_boundary_and_power_of_two() {
        let mut candidates = [u32::MAX, u32::MAX - 1].into_iter();
        assert_eq!(sample_index(3, || candidates.next().unwrap()), 2);
        assert!(candidates.next().is_none());
        assert_eq!(sample_index(65536, || u32::MAX), 65535);
        let limit = ((1_u64 << 32) / 553 * 553) as u32;
        let mut candidates = [limit, limit - 1].into_iter();
        assert_eq!(sample_index(553, || candidates.next().unwrap()), 552);
        assert!(candidates.next().is_none());
    }
}
