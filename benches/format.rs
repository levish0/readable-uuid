use bip39::{Language, Mnemonic};
use criterion::{Criterion, criterion_group, criterion_main};
use readable_uuid::{ReadableUuid, Uuid};
use std::hint::black_box;

fn bench(criterion: &mut Criterion) {
    let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let codec = ReadableUuid::builder().separator(" ").build().unwrap();
    let phrase = codec.encode(&id);
    let mnemonic = Mnemonic::from_entropy(id.as_bytes()).unwrap().to_string();
    let niceware_phrase = niceware::bytes_to_passphrase(id.as_bytes())
        .unwrap()
        .join(" ");
    assert_eq!(
        niceware::passphrase_to_bytes(&niceware_phrase.split(' ').collect::<Vec<_>>()).unwrap(),
        id.as_bytes()
    );
    assert_eq!(codec.decode(&phrase).unwrap(), id);
    assert_eq!(
        Mnemonic::parse_in_normalized(Language::English, &mnemonic)
            .unwrap()
            .to_entropy(),
        id.as_bytes()
    );

    // Both encoders allocate a phrase; both decoders return the original UUID.
    // BIP39 also computes and validates its four-bit checksum.
    criterion.bench_function("readable-uuid/encode", |bench| {
        bench.iter(|| codec.encode(black_box(&id)))
    });
    criterion.bench_function("bip39/encode", |bench| {
        bench.iter(|| {
            Mnemonic::from_entropy(black_box(id.as_bytes()))
                .unwrap()
                .to_string()
        })
    });
    criterion.bench_function("readable-uuid/decode", |bench| {
        bench.iter(|| codec.decode(black_box(&phrase)).unwrap())
    });
    criterion.bench_function("niceware/encode", |bench| {
        bench.iter(|| {
            niceware::bytes_to_passphrase(black_box(id.as_bytes()))
                .unwrap()
                .join(" ")
        })
    });
    criterion.bench_function("niceware/decode", |bench| {
        bench.iter(|| {
            let words = black_box(&niceware_phrase).split(' ').collect::<Vec<_>>();
            Uuid::from_slice(&niceware::passphrase_to_bytes(&words).unwrap()).unwrap()
        })
    });
    criterion.bench_function("bip39/decode", |bench| {
        bench.iter(|| {
            let parsed =
                Mnemonic::parse_in_normalized(Language::English, black_box(&mnemonic)).unwrap();
            let (bytes, length) = parsed.to_entropy_array();
            Uuid::from_slice(&bytes[..length]).unwrap()
        })
    });
    let mut output = String::with_capacity(256);
    criterion.bench_function("readable-uuid/encode-reuse", |bench| {
        bench.iter(|| {
            output.clear();
            codec.encode_into(black_box(&id), &mut output);
            black_box(&output);
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
