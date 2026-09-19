use std::{collections::HashSet, env, fs, path::PathBuf};
fn main() {
    let mut output = String::new();
    for (name, file) in [
        ("ENGLISH", "english-v1"),
        ("SHORT", "short-v1"),
        ("NATURE", "nature-v1"),
    ] {
        let path = format!("wordlists/{file}.txt");
        println!("cargo:rerun-if-changed={path}");
        let text = fs::read_to_string(path).unwrap();
        let words: Vec<_> = text.lines().collect();
        assert!(words.len() >= 2);
        assert_eq!(words.iter().collect::<HashSet<_>>().len(), words.len());
        assert!(
            words
                .iter()
                .all(|w| !w.is_empty() && w.bytes().all(|b| b.is_ascii_lowercase()))
        );
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
