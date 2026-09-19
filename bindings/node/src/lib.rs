use napi_derive::napi;
use readable_uuid::{MAX_WORDS, ReadableUuid, WordSet};

#[napi(object)]
#[derive(Default)]
pub struct Options {
    pub word_set: Option<String>,
    pub words: Option<f64>,
    pub separator: Option<String>,
    pub custom_words: Option<Vec<String>>,
}

fn with_formatter<T>(
    options: Option<Options>,
    run: impl FnOnce(&ReadableUuid<'_>) -> napi::Result<T>,
) -> napi::Result<T> {
    let options = options.unwrap_or_default();
    let mut builder = ReadableUuid::builder();

    if options.word_set.is_some() && options.custom_words.is_some() {
        return Err(napi::Error::from_reason(
            "choose wordSet or customWords, not both",
        ));
    }
    let custom_words: Vec<&str>;

    if let Some(ref words) = options.custom_words {
        custom_words = words.iter().map(String::as_str).collect();
        builder = builder.custom_words(&custom_words);
    } else if let Some(ref name) = options.word_set {
        let set = WordSet::ALL
            .iter()
            .copied()
            .find(|s| s.name() == name)
            .ok_or_else(|| napi::Error::from_reason(format!("unknown word set: {name}")))?;
        builder = builder.word_set(set);
    }
    if let Some(words) = options.words {
        if !words.is_finite() || words.fract() != 0.0 || !(1.0..=MAX_WORDS as f64).contains(&words)
        {
            return Err(napi::Error::from_reason(format!(
                "words must be an integer between 1 and {MAX_WORDS}"
            )));
        }
        builder = builder.words(words as usize);
    }

    if let Some(ref separator) = options.separator {
        builder = builder.separator(separator);
    }

    let formatter = builder
        .build()
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    run(&formatter)
}

#[napi]
pub fn readable_uuid(uuid: String, options: Option<Options>) -> napi::Result<String> {
    with_formatter(options, |formatter| {
        formatter
            .format_str(&uuid)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    })
}

#[napi]
pub fn readable_uuid_batch(
    uuids: Vec<String>,
    options: Option<Options>,
) -> napi::Result<Vec<String>> {
    with_formatter(options, |formatter| {
        uuids
            .iter()
            .map(|id| {
                formatter
                    .format_str(id)
                    .map_err(|e| napi::Error::from_reason(e.to_string()))
            })
            .collect()
    })
}

#[napi(object)]
pub struct WordSetInfo {
    pub name: String,
    pub size: u32,
    pub bits_per_word: f64,
}

#[napi]
pub fn word_sets() -> Vec<WordSetInfo> {
    WordSet::ALL
        .iter()
        .copied()
        .map(|set| WordSetInfo {
            name: set.name().into(),
            size: set.words().len() as u32,
            bits_per_word: (set.words().len() as f64).log2(),
        })
        .collect()
}
