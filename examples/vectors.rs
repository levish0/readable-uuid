use readable_uuid::{ReadableUuid, WordSet};

fn main() {
    let mut vectors = Vec::new();
    let mut dictionaries = serde_json::Map::new();
    for &set in WordSet::ALL {
        dictionaries.insert(
            set.name().into(),
            serde_json::json!(
                blake3::hash(format!("{}\n", set.words().join("\n")).as_bytes())
                    .to_hex()
                    .to_string()
            ),
        );
        for id in [
            "00000000-0000-0000-0000-000000000000",
            "ffffffff-ffff-ffff-ffff-ffffffffffff",
            "550e8400-e29b-41d4-a716-446655440000",
            "01992000-1234-7000-8000-000000000001",
        ] {
            for words in [1, 4, 6, 17, 64] {
                let separator = if words == 6 { " " } else { "-" };
                let label = ReadableUuid::builder()
                    .word_set(set)
                    .words(words)
                    .separator(separator)
                    .build()
                    .unwrap()
                    .format_str(id)
                    .unwrap();
                vectors.push(serde_json::json!({
                    "uuid": id,
                    "set": set.name(),
                    "words": words,
                    "separator": separator,
                    "label": label,
                }));
            }
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"dictionaries":dictionaries,"vectors":vectors})
        )
        .unwrap()
    );
}
