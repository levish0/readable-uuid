use js_sys::Array;
use readable_uuid::ReadableUuid;
use wasm_bindgen::prelude::*;

fn error(message: impl ToString) -> JsError {
    JsError::new(&message.to_string())
}

fn with_codec<T>(
    separator: Option<&str>,
    run: impl FnOnce(&ReadableUuid<'_>) -> Result<T, JsError>,
) -> Result<T, JsError> {
    let mut builder = ReadableUuid::builder();

    if let Some(separator) = separator {
        builder = builder.separator(separator);
    }

    let codec = builder.build().map_err(error)?;
    run(&codec)
}

#[wasm_bindgen]
pub fn encode(uuid: &str, separator: Option<String>) -> Result<String, JsError> {
    with_codec(separator.as_deref(), |codec| {
        codec.encode_str(uuid).map_err(error)
    })
}

#[wasm_bindgen(js_name = encodeBatch)]
pub fn encode_batch(uuids: &Array, separator: Option<String>) -> Result<Array, JsError> {
    with_codec(separator.as_deref(), |codec| {
        let output = Array::new_with_length(uuids.length());
        for index in 0..uuids.length() {
            let uuid = uuids
                .get(index)
                .as_string()
                .ok_or_else(|| error(format!("UUID at index {index} must be a string")))?;
            output.set(index, codec.encode_str(&uuid).map_err(error)?.into());
        }
        Ok(output)
    })
}

#[wasm_bindgen]
pub fn decode(phrase: &str, separator: Option<String>) -> Result<String, JsError> {
    with_codec(separator.as_deref(), |codec| {
        codec
            .decode(phrase)
            .map(|uuid| uuid.to_string())
            .map_err(error)
    })
}

#[wasm_bindgen(js_name = decodeBatch)]
pub fn decode_batch(phrases: &Array, separator: Option<String>) -> Result<Array, JsError> {
    with_codec(separator.as_deref(), |codec| {
        let output = Array::new_with_length(phrases.length());
        for index in 0..phrases.length() {
            let phrase = phrases
                .get(index)
                .as_string()
                .ok_or_else(|| error(format!("Phrase at index {index} must be a string")))?;
            output.set(
                index,
                codec.decode(&phrase).map_err(error)?.to_string().into(),
            );
        }
        Ok(output)
    })
}
