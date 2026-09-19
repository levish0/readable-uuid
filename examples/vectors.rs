use readable_uuid::ReadableUuid;

fn main() {
    let mut vectors = Vec::new();
    let mut dictionaries = serde_json::Map::new();
    for (name, source) in [
        ("modifiers", include_str!("../wordlists/modifiers.txt")),
        ("nouns", include_str!("../wordlists/nouns.txt")),
    ] {
        let canonical = format!("{}\n", source.lines().collect::<Vec<_>>().join("\n"));
        dictionaries.insert(
            name.into(),
            serde_json::json!(blake3::hash(canonical.as_bytes()).to_hex().to_string()),
        );
    }
    for id in [
        "00000000-0000-0000-0000-000000000000",
        "ffffffff-ffff-ffff-ffff-ffffffffffff",
        "550e8400-e29b-41d4-a716-446655440000",
        "01992000-1234-7000-8000-000000000001",
    ] {
        for separator in ["-", " ", "::"] {
            let codec = ReadableUuid::builder()
                .separator(separator)
                .build()
                .unwrap();
            vectors.push(serde_json::json!({
                "uuid": id, "words": codec.word_count(), "separator": separator,
                "label": codec.encode_str(id).unwrap(),
            }));
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "dictionaries": dictionaries, "vectors": vectors,
        }))
        .unwrap()
    );
}
