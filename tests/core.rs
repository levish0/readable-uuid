use readable_uuid::{CODEBOOK_SIZE, CODEWORDS_PER_UUID, Error, ReadableUuid, Uuid};

#[test]
fn all_codewords_roundtrip_with_case_variations() {
    let codec = ReadableUuid::default();
    assert_eq!(codec.word_count(), CODEWORDS_PER_UUID);
    assert_eq!(codec.dictionary_len(), CODEBOOK_SIZE);

    for pair in 0..=u16::MAX {
        let mut bytes = [0_u8; 16];
        bytes[..2].copy_from_slice(&pair.to_be_bytes());
        let id = Uuid::from_bytes(bytes);
        let phrase = codec.encode(&id);

        for text in [
            phrase.clone(),
            phrase.to_ascii_uppercase(),
            phrase.to_ascii_lowercase(),
        ] {
            assert_eq!(codec.decode(&text).unwrap(), id);
        }
    }
}

#[test]
fn mapping_matches_ordered_component_files() {
    let modifiers: Vec<_> = include_str!("../wordlists/modifiers.txt").lines().collect();
    let nouns: Vec<_> = include_str!("../wordlists/nouns.txt").lines().collect();
    let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let expected = id
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let noun = nouns[usize::from(pair[1])];
            format!(
                "{}{}{}",
                modifiers[usize::from(pair[0])],
                noun[..1].to_ascii_uppercase(),
                &noun[1..]
            )
        })
        .collect::<Vec<_>>()
        .join("-");

    assert_eq!(readable_uuid::encode(&id), expected);
    assert_eq!(readable_uuid::decode(&expected).unwrap(), id);
}

#[test]
fn uuid_spellings_and_output_buffers() {
    let codec = ReadableUuid::default();
    let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let expected = codec.encode(&id);

    for text in [
        "550E8400-E29B-41D4-A716-446655440000",
        "550e8400e29b41d4a716446655440000",
        "urn:uuid:550e8400-e29b-41d4-a716-446655440000",
        "{550e8400-e29b-41d4-a716-446655440000}",
    ] {
        assert_eq!(codec.encode_str(text).unwrap(), expected);
    }

    let mut buffer = String::from("prefix:");
    codec.encode_into(&id, &mut buffer);
    assert_eq!(buffer, format!("prefix:{expected}"));
    assert!(matches!(
        codec.encode_str("abc"),
        Err(Error::InvalidUuid(_))
    ));
}

#[test]
fn roundtrip_uuid_bits_and_deterministic_corpus() {
    for separator in ["-", "::", " "] {
        let codec = ReadableUuid::builder()
            .separator(separator)
            .build()
            .unwrap();

        for value in [0, 1, u128::MAX, 1 << 127] {
            let id = Uuid::from_u128(value);
            assert_eq!(codec.decode(&codec.encode(&id)).unwrap(), id);
        }
        for bit in 0..128 {
            let id = Uuid::from_u128(1 << bit);
            assert_eq!(codec.decode(&codec.encode(&id)).unwrap(), id);
        }

        let mut state = 0x550e8400e29b41d4a716446655440000_u128;
        for _ in 0..4096 {
            state ^= state << 23;
            state ^= state >> 17;
            state ^= state << 26;
            let id = Uuid::from_u128(state);
            assert_eq!(codec.decode(&codec.encode(&id)).unwrap(), id);
        }
    }
}

#[test]
fn invalid_configuration_and_phrases() {
    for separator in ["", "a", "\n", "é", &"-".repeat(33)] {
        assert_eq!(
            ReadableUuid::builder()
                .separator(separator)
                .build()
                .unwrap_err(),
            Error::InvalidSeparator
        );
    }

    let codec = ReadableUuid::default();
    let phrase = codec.encode(&Uuid::nil());
    for invalid in [
        String::new(),
        format!("{phrase}-ableAcorn"),
        phrase.split('-').skip(1).collect::<Vec<_>>().join("-"),
    ] {
        assert!(matches!(
            codec.decode(&invalid),
            Err(Error::InvalidCodewordCount { .. })
        ));
    }

    let mut codewords: Vec<_> = phrase.split('-').collect();
    codewords[3] = "unknown";
    assert_eq!(
        codec.decode(&codewords.join("-")),
        Err(Error::UnknownCodeword(3))
    );

    let spaced = ReadableUuid::builder().separator(" ").build().unwrap();
    let id = Uuid::max();
    assert_eq!(spaced.decode(&spaced.encode(&id)).unwrap(), id);
    assert!(
        spaced
            .decode(&spaced.encode(&id).replace(' ', "  "))
            .is_err()
    );
}

#[test]
fn frozen_vectors_and_word_lists() {
    let data: serde_json::Value = serde_json::from_str(include_str!("vectors.json")).unwrap();

    for row in data["vectors"].as_array().unwrap() {
        let codec = ReadableUuid::builder()
            .separator(row["separator"].as_str().unwrap())
            .build()
            .unwrap();
        let id = Uuid::parse_str(row["uuid"].as_str().unwrap()).unwrap();
        let phrase = row["label"].as_str().unwrap();

        assert_eq!(codec.word_count(), row["words"].as_u64().unwrap() as usize);
        assert_eq!(codec.encode(&id), phrase);
        assert_eq!(codec.decode(phrase).unwrap(), id);
    }

    for (name, source) in [
        ("modifiers", include_str!("../wordlists/modifiers.txt")),
        ("nouns", include_str!("../wordlists/nouns.txt")),
    ] {
        let canonical = format!("{}\n", source.lines().collect::<Vec<_>>().join("\n"));
        assert_eq!(
            blake3::hash(canonical.as_bytes()).to_hex().as_str(),
            data["dictionaries"][name].as_str().unwrap()
        );
    }
}
