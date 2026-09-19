use std::{env, fs, path::PathBuf};

const CODEBOOK_SIZE: usize = 65_536;
const MIN_CODEWORD_BYTES: usize = 3;
const MAX_CODEWORD_BYTES: usize = 12;

fn main() {
    const PATH: &str = "wordlists/codewords.txt";

    println!("cargo:rerun-if-changed={PATH}");

    let source = fs::read(PATH).expect("read codeword list");
    assert!(source.ends_with(b"\n"), "{PATH}: must end with a newline");
    assert!(!source.contains(&b'\r'), "{PATH}: must use LF line endings");

    let codewords: Vec<_> = source[..source.len() - 1]
        .split(|byte| *byte == b'\n')
        .collect();
    assert_eq!(
        codewords.len(),
        CODEBOOK_SIZE,
        "{PATH}: expected exactly {CODEBOOK_SIZE} entries"
    );

    for (index, codeword) in codewords.iter().enumerate() {
        assert!(
            (MIN_CODEWORD_BYTES..=MAX_CODEWORD_BYTES).contains(&codeword.len())
                && codeword.iter().all(u8::is_ascii_lowercase),
            "{PATH}: entry {index} must be {MIN_CODEWORD_BYTES}..={MAX_CODEWORD_BYTES} lowercase ASCII letters"
        );
    }
    assert!(
        codewords.windows(2).all(|pair| pair[0] < pair[1]),
        "{PATH}: entries must be unique and sorted by ASCII byte order"
    );

    let mut position = 0_u32;
    let mut offsets = Vec::with_capacity((CODEBOOK_SIZE + 1) * size_of::<u32>());
    for codeword in &codewords {
        offsets.extend_from_slice(&position.to_le_bytes());
        position += u32::try_from(codeword.len() + 1).expect("codeword source fits in u32");
    }
    offsets.extend_from_slice(&position.to_le_bytes());

    let output_directory = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(output_directory.join("codeword-offsets.bin"), offsets).unwrap();
}
