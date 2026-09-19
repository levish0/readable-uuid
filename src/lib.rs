//! Reversible English codewords for UUIDs.
//!
//! ```
//! use readable_uuid::{decode, encode, Uuid};
//! let id = Uuid::nil();
//! assert_eq!(decode(&encode(&id))?, id);
//! # Ok::<(), readable_uuid::Error>(())
//! ```
use std::{fmt, sync::LazyLock};

pub use uuid::Uuid;

mod lists {
    include!(concat!(env!("OUT_DIR"), "/lists.rs"));
}

/// Number of codewords in the fixed built-in codebook.
pub const CODEBOOK_SIZE: usize = 65_536;

/// Number of codewords used to represent one UUID.
pub const CODEWORDS_PER_UUID: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    InvalidSeparator,
    InvalidUuid(uuid::Error),
    InvalidCodewordCount { actual: usize },
    UnknownCodeword(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSeparator => {
                formatter.write_str("separator must be 1..=32 ASCII punctuation or space bytes")
            }
            Self::InvalidUuid(error) => error.fmt(formatter),
            Self::InvalidCodewordCount { actual } => {
                write!(
                    formatter,
                    "expected {CODEWORDS_PER_UUID} codewords, received {actual}"
                )
            }
            Self::UnknownCodeword(index) => write!(formatter, "unknown codeword at index {index}"),
        }
    }
}

impl std::error::Error for Error {}

/// Reusable UUID codec with a configurable separator.
#[derive(Debug, Clone)]
pub struct ReadableUuid<'a> {
    separator: &'a str,
    max_len: usize,
}

#[derive(Debug, Clone)]
pub struct Builder<'a> {
    separator: &'a str,
}

impl Default for ReadableUuid<'static> {
    fn default() -> Self {
        Self::builder().build().expect("valid defaults")
    }
}

impl<'a> ReadableUuid<'a> {
    pub fn builder() -> Builder<'a> {
        Builder { separator: "-" }
    }

    pub fn encode(&self, id: &Uuid) -> String {
        let mut output = String::with_capacity(self.max_len);
        self.encode_into(id, &mut output);
        output
    }

    pub fn encode_str(&self, id: &str) -> Result<String, Error> {
        Ok(self.encode(&Uuid::try_parse(id).map_err(Error::InvalidUuid)?))
    }

    /// Appends a phrase; clear the buffer first to replace its contents.
    pub fn encode_into(&self, id: &Uuid, output: &mut String) {
        output.reserve(self.max_len);

        for (position, pair) in id.as_bytes().chunks_exact(2).enumerate() {
            if position != 0 {
                output.push_str(self.separator);
            }

            output.push_str(lists::MODIFIERS[usize::from(pair[0])]);
            let noun = lists::NOUNS[usize::from(pair[1])];
            output.push(char::from(noun.as_bytes()[0].to_ascii_uppercase()));
            output.push_str(&noun[1..]);
        }
    }

    /// Decodes exactly eight built-in codewords.
    ///
    /// ASCII case differences are accepted. There is no checksum: replacing a
    /// valid codeword can produce a different UUID.
    pub fn decode(&self, phrase: &str) -> Result<Uuid, Error> {
        let actual = phrase.split(self.separator).count();
        if actual != CODEWORDS_PER_UUID {
            return Err(Error::InvalidCodewordCount { actual });
        }

        let mut bytes = [0_u8; 16];
        for (position, codeword) in phrase.split(self.separator).enumerate() {
            let pair = decode_compound(codeword).ok_or(Error::UnknownCodeword(position))?;
            bytes[position * 2..position * 2 + 2].copy_from_slice(&pair);
        }

        Ok(Uuid::from_bytes(bytes))
    }

    pub const fn dictionary_len(&self) -> usize {
        CODEBOOK_SIZE
    }

    pub const fn word_count(&self) -> usize {
        CODEWORDS_PER_UUID
    }
}

fn decode_compound(codeword: &str) -> Option<[u8; 2]> {
    const MAX_COMPOUND_BYTES: usize = lists::MODIFIERS_MAX + lists::NOUNS_MAX;

    if codeword.len() > MAX_COMPOUND_BYTES
        || !codeword.bytes().all(|byte| byte.is_ascii_alphabetic())
    {
        return None;
    }

    let mut lowercase = [0_u8; MAX_COMPOUND_BYTES];
    for (output, byte) in lowercase.iter_mut().zip(codeword.bytes()) {
        *output = byte.to_ascii_lowercase();
    }
    let normalized = std::str::from_utf8(&lowercase[..codeword.len()]).ok()?;

    let lookup = |boundary| {
        let modifier = lists::MODIFIERS
            .binary_search(&&normalized[..boundary])
            .ok()?;
        let noun = lists::NOUNS.binary_search(&&normalized[boundary..]).ok()?;
        Some([modifier as u8, noun as u8])
    };

    if let Some(boundary) = codeword.bytes().position(|byte| byte.is_ascii_uppercase())
        && boundary != 0
        && let Some(pair) = lookup(boundary)
    {
        return Some(pair);
    }

    let first_boundary = lists::MODIFIERS_MIN.max(codeword.len().saturating_sub(lists::NOUNS_MAX));
    let last_boundary = lists::MODIFIERS_MAX.min(codeword.len().saturating_sub(lists::NOUNS_MIN));
    if first_boundary > last_boundary {
        return None;
    }

    let mut found = None;
    for boundary in first_boundary..=last_boundary {
        if let Some(pair) = lookup(boundary) {
            if found.is_some() {
                return None;
            }
            found = Some(pair);
        }
    }
    found
}

impl<'a> Builder<'a> {
    pub fn separator(mut self, separator: &'a str) -> Self {
        self.separator = separator;
        self
    }

    pub fn build(self) -> Result<ReadableUuid<'a>, Error> {
        if self.separator.is_empty()
            || self.separator.len() > 32
            || !self
                .separator
                .bytes()
                .all(|byte| byte.is_ascii_punctuation() || byte == b' ')
        {
            return Err(Error::InvalidSeparator);
        }

        let max_len = (lists::MODIFIERS_MAX + lists::NOUNS_MAX) * CODEWORDS_PER_UUID
            + self.separator.len() * (CODEWORDS_PER_UUID - 1);

        Ok(ReadableUuid {
            separator: self.separator,
            max_len,
        })
    }
}

static DEFAULT: LazyLock<ReadableUuid<'static>> = LazyLock::new(ReadableUuid::default);

/// Encodes a UUID as eight built-in codewords separated by hyphens.
pub fn encode(id: &Uuid) -> String {
    DEFAULT.encode(id)
}

/// Decodes eight built-in codewords separated by hyphens.
pub fn decode(phrase: &str) -> Result<Uuid, Error> {
    DEFAULT.decode(phrase)
}
