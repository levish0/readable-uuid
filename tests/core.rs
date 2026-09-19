use readable_uuid::{Error, MAX_WORDS, ReadableUuid, Uuid, WordSet};

#[test]
fn dictionaries_are_valid_and_short_is_short() {
    for set in WordSet::ALL {
        ReadableUuid::builder()
            .custom_words(set.words())
            .build()
            .unwrap();
        assert!(set.words().windows(2).all(|w| w[0] < w[1]));
    }
    assert!(WordSet::ShortV1.words().iter().all(|w| w.len() <= 4));
}

#[test]
fn canonical_input_and_appending() {
    let formatter = ReadableUuid::default();
    let expected = formatter
        .format_str("550e8400-e29b-41d4-a716-446655440000")
        .unwrap();
    for text in [
        "550E8400-E29B-41D4-A716-446655440000",
        "550e8400e29b41d4a716446655440000",
        "urn:uuid:550e8400-e29b-41d4-a716-446655440000",
        "{550e8400-e29b-41d4-a716-446655440000}",
    ] {
        assert_eq!(formatter.format_str(text).unwrap(), expected);
    }
    let mut buffer = String::from("prefix:");
    formatter.write_into(
        &Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        &mut buffer,
    );
    assert_eq!(buffer, format!("prefix:{expected}"));
    assert!(formatter.format_str("abc").is_err());
}

#[test]
fn prefix_is_stable_across_xof_blocks() {
    for set in WordSet::ALL {
        let long = ReadableUuid::builder()
            .word_set(set)
            .words(MAX_WORDS)
            .build()
            .unwrap()
            .format(&Uuid::max());
        for count in [1, 4, 6, 16, 17, 32, 64] {
            let formatter = ReadableUuid::builder()
                .word_set(set)
                .words(count)
                .build()
                .unwrap();
            assert_eq!(
                formatter.format(&Uuid::max()),
                long.split('-').take(count).collect::<Vec<_>>().join("-")
            );
        }
    }
}

#[test]
fn invalid_configuration() {
    for n in [0, 65, usize::MAX] {
        assert_eq!(
            ReadableUuid::builder().words(n).build().unwrap_err(),
            Error::InvalidWordCount
        );
    }
    for words in [&[][..], &["one"][..]] {
        assert_eq!(
            ReadableUuid::builder()
                .custom_words(words)
                .build()
                .unwrap_err(),
            Error::InvalidDictionarySize
        );
    }
    for words in [
        ["", "two"],
        ["One", "two"],
        ["one-two", "three"],
        ["one two", "three"],
    ] {
        assert!(matches!(
            ReadableUuid::builder().custom_words(&words).build(),
            Err(Error::InvalidWord(_))
        ));
    }
    assert_eq!(
        ReadableUuid::builder()
            .custom_words(&["one", "one"])
            .build()
            .unwrap_err(),
        Error::DuplicateWord(1)
    );
    for separator in ["", "original_words", "\n"] {
        assert_eq!(
            ReadableUuid::builder()
                .separator(separator)
                .build()
                .unwrap_err(),
            Error::InvalidSeparator
        );
    }
}

#[test]
fn custom_order_and_combination_space() {
    let original_words = ["red", "green", "blue"];
    let rotated_words = ["blue", "red", "green"];
    let original_formatter = ReadableUuid::builder()
        .custom_words(&original_words)
        .words(64)
        .build()
        .unwrap();
    let rotated_formatter = ReadableUuid::builder()
        .custom_words(&rotated_words)
        .words(64)
        .build()
        .unwrap();
    let original_label = original_formatter.format(&Uuid::nil());
    let rotated_label = rotated_formatter.format(&Uuid::nil());
    for (original_word, rotated_word) in original_label.split('-').zip(rotated_label.split('-')) {
        assert_eq!(
            original_words.iter().position(|w| *w == original_word),
            rotated_words.iter().position(|w| *w == rotated_word)
        );
    }
    assert!((original_formatter.combination_bits() - 64.0 * 3.0_f64.log2()).abs() < 1e-10);
}

#[test]
fn frozen_vectors_and_dictionaries() {
    let data: serde_json::Value = serde_json::from_str(include_str!("vectors.json")).unwrap();
    for row in data["vectors"].as_array().unwrap() {
        let set = WordSet::ALL
            .into_iter()
            .find(|s| s.name() == row["set"].as_str().unwrap())
            .unwrap();
        let formatter = ReadableUuid::builder()
            .word_set(set)
            .words(row["words"].as_u64().unwrap() as usize)
            .separator(row["separator"].as_str().unwrap())
            .build()
            .unwrap();
        assert_eq!(
            formatter.format_str(row["uuid"].as_str().unwrap()).unwrap(),
            row["label"].as_str().unwrap()
        );
    }
    for set in WordSet::ALL {
        let canonical = format!("{}\n", set.words().join("\n"));
        assert_eq!(
            blake3::hash(canonical.as_bytes()).to_hex().as_str(),
            data["dictionaries"][set.name()].as_str().unwrap()
        );
    }
}
