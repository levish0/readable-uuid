use std::{collections::HashSet, env, fs, path::PathBuf};

const COMPONENT_COUNT: usize = 256;
const MIN_COMPONENT_BYTES: usize = 3;
const MAX_COMPONENT_BYTES: usize = 7;

fn main() {
    println!("cargo:rerun-if-changed=wordlists/modifiers.txt");
    println!("cargo:rerun-if-changed=wordlists/nouns.txt");

    let modifiers = read_components("wordlists/modifiers.txt");
    let nouns = read_components("wordlists/nouns.txt");
    validate_compounds(&modifiers, &nouns);

    let output = format!(
        "pub static MODIFIERS: &[&str; {COMPONENT_COUNT}] = &{modifiers:?};\n\
         pub static NOUNS: &[&str; {COMPONENT_COUNT}] = &{nouns:?};\n\
         pub const MODIFIERS_MIN: usize = {};\n\
         pub const MODIFIERS_MAX: usize = {};\n\
         pub const NOUNS_MIN: usize = {};\n\
         pub const NOUNS_MAX: usize = {};\n",
        minimum_length(&modifiers),
        maximum_length(&modifiers),
        minimum_length(&nouns),
        maximum_length(&nouns),
    );

    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("lists.rs"),
        output,
    )
    .unwrap();
}

fn read_components(path: &str) -> Vec<String> {
    let text = fs::read_to_string(path).expect("read word-list component file");
    let words: Vec<_> = text.lines().map(str::to_owned).collect();

    assert_eq!(
        words.len(),
        COMPONENT_COUNT,
        "{path}: expected exactly {COMPONENT_COUNT} entries"
    );
    assert!(
        words.windows(2).all(|pair| pair[0] < pair[1]),
        "{path}: entries must be strictly sorted"
    );

    for (index, word) in words.iter().enumerate() {
        assert!(
            (MIN_COMPONENT_BYTES..=MAX_COMPONENT_BYTES).contains(&word.len())
                && word.bytes().all(|byte| byte.is_ascii_lowercase()),
            "{path}: entry {index} must be {MIN_COMPONENT_BYTES}..={MAX_COMPONENT_BYTES} lowercase ASCII letters"
        );
    }

    words
}

fn validate_compounds(modifiers: &[String], nouns: &[String]) {
    let mut compounds = HashSet::with_capacity(COMPONENT_COUNT * COMPONENT_COUNT);

    for modifier in modifiers {
        for noun in nouns {
            let mut compound = String::with_capacity(modifier.len() + noun.len());
            compound.push_str(modifier);
            compound.push_str(noun);
            assert!(
                compounds.insert(compound),
                "component lists contain an ambiguous compound: {modifier} + {noun}"
            );
        }
    }
}

fn minimum_length(words: &[String]) -> usize {
    words.iter().map(String::len).min().unwrap()
}

fn maximum_length(words: &[String]) -> usize {
    words.iter().map(String::len).max().unwrap()
}
