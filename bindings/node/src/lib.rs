use napi_derive::napi;
use readable_uuid::ReadableUuid;

#[napi(object)]
#[derive(Default)]
pub struct Options {
    pub separator: Option<String>,
}

fn with_codec<T>(
    options: Option<Options>,
    run: impl FnOnce(&ReadableUuid<'_>) -> napi::Result<T>,
) -> napi::Result<T> {
    let options = options.unwrap_or_default();
    let mut builder = ReadableUuid::builder();

    if let Some(ref separator) = options.separator {
        builder = builder.separator(separator);
    }

    let codec = builder
        .build()
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    run(&codec)
}

#[napi]
pub fn encode(uuid: String, options: Option<Options>) -> napi::Result<String> {
    with_codec(options, |codec| {
        codec
            .encode_str(&uuid)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    })
}

#[napi]
pub fn encode_batch(uuids: Vec<String>, options: Option<Options>) -> napi::Result<Vec<String>> {
    with_codec(options, |codec| {
        uuids
            .iter()
            .map(|id| {
                codec
                    .encode_str(id)
                    .map_err(|e| napi::Error::from_reason(e.to_string()))
            })
            .collect()
    })
}

#[napi]
pub fn decode(phrase: String, options: Option<Options>) -> napi::Result<String> {
    with_codec(options, |codec| {
        codec
            .decode(&phrase)
            .map(|uuid| uuid.to_string())
            .map_err(|error| napi::Error::from_reason(error.to_string()))
    })
}

#[napi]
pub fn decode_batch(phrases: Vec<String>, options: Option<Options>) -> napi::Result<Vec<String>> {
    with_codec(options, |codec| {
        phrases
            .iter()
            .map(|phrase| {
                codec
                    .decode(phrase)
                    .map(|uuid| uuid.to_string())
                    .map_err(|error| napi::Error::from_reason(error.to_string()))
            })
            .collect()
    })
}
