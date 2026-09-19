use std::{collections::HashSet, env, fs, path::PathBuf};

#[path = "src/limits.rs"]
mod limits;

fn main() {
    println!("cargo:rerun-if-changed=src/limits.rs");

    let mut output = String::new();
    for (name, file) in [
        ("ENGLISH", "english-v1"),
        ("SHORT", "short-v1"),
        ("NATURE", "nature-v1"),
    ] {
        let path = format!("wordlists/{file}.txt");
        println!("cargo:rerun-if-changed={path}");
        let text = fs::read_to_string(&path).expect("read built-in dictionary");
        let words: Vec<_> = text.lines().collect();
        assert!(
            (2..=limits::MAX_DICTIONARY_WORDS).contains(&words.len()),
            "{path}: dictionary size must be 2..={}",
            limits::MAX_DICTIONARY_WORDS,
        );

        let mut seen = HashSet::with_capacity(words.len());
        for (index, word) in words.iter().enumerate() {
            assert!(
                !word.is_empty() && word.bytes().all(|byte| byte.is_ascii_lowercase()),
                "{path}: word {index} must contain only lowercase ASCII letters",
            );
            assert!(
                word.len() <= limits::MAX_WORD_BYTES,
                "{path}: word {index} exceeds {} bytes",
                limits::MAX_WORD_BYTES,
            );
            assert!(seen.insert(word), "{path}: duplicate word at index {index}");
        }

        output.push_str(&format!("pub static {name}: &[&str] = &{words:?};\n"));
        output.push_str(&format!(
            "pub const {name}_MAX: usize = {};\n",
            words.iter().map(|w| w.len()).max().unwrap()
        ));
    }
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("lists.rs"),
        output,
    )
    .unwrap();
}
