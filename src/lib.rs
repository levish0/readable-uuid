//! Reversible English codewords for UUIDs.
//!
//! ```
//! use readable_uuid::{decode, encode, Uuid};
//! let id = Uuid::nil();
//! assert_eq!(decode(&encode(&id))?, id);
//! # Ok::<(), readable_uuid::Error>(())
//! ```
use std::{cmp::Ordering, fmt, sync::LazyLock};

pub use uuid::Uuid;

/// Number of codewords in the fixed built-in codebook.
pub const CODEBOOK_SIZE: usize = 65_536;

/// Number of codewords used to represent one UUID.
pub const CODEWORDS_PER_UUID: usize = 8;

const MIN_CODEWORD_BYTES: usize = 3;
const MAX_CODEWORD_BYTES: usize = 12;
const CODEWORD_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/wordlists/codewords.txt"
));
const CODEWORD_OFFSETS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/codeword-offsets.bin"));

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

        for (position, pair) in id.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            if position != 0 {
                output.push_str(self.separator);
            }

            let index = usize::from(u16::from_be_bytes([pair[0], pair[1]]));
            output.push_str(codeword_at(index));
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
            let value = decode_codeword(codeword).ok_or(Error::UnknownCodeword(position))?;
            bytes[position * 2..position * 2 + 2].copy_from_slice(&value.to_be_bytes());
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

fn codeword_at(index: usize) -> &'static str {
    let start = codeword_offset(index);
    let end = codeword_offset(index + 1) - 1;
    std::str::from_utf8(&CODEWORD_BYTES[start..end])
        .expect("build script validated ASCII codewords")
}

fn codeword_offset(index: usize) -> usize {
    let start = index * size_of::<u32>();
    let bytes: [u8; 4] = CODEWORD_OFFSETS[start..start + 4]
        .try_into()
        .expect("build script generated every codeword offset");
    u32::from_le_bytes(bytes) as usize
}

fn decode_codeword(input: &str) -> Option<u16> {
    if !(MIN_CODEWORD_BYTES..=MAX_CODEWORD_BYTES).contains(&input.len())
        || !input.bytes().all(|byte| byte.is_ascii_alphabetic())
    {
        return None;
    }

    let mut start = 0;
    let mut end = CODEBOOK_SIZE;
    while start < end {
        let middle = start + (end - start) / 2;
        match compare_codeword(codeword_at(middle), input) {
            Ordering::Less => start = middle + 1,
            Ordering::Greater => end = middle,
            Ordering::Equal => return Some(middle as u16),
        }
    }
    None
}

fn compare_codeword(canonical: &str, input: &str) -> Ordering {
    canonical
        .bytes()
        .zip(input.bytes())
        .map(|(left, right)| left.cmp(&right.to_ascii_lowercase()))
        .find(|ordering| *ordering != Ordering::Equal)
        .unwrap_or_else(|| canonical.len().cmp(&input.len()))
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

        let max_len = MAX_CODEWORD_BYTES * CODEWORDS_PER_UUID
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
