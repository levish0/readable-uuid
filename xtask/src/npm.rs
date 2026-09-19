use crate::{Result, capture, root, run, version};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const PACKAGE_FILES: [&str; 7] = [
    "api.js",
    "browser.js",
    "index.js",
    "index.d.ts",
    "README.md",
    "LICENSE",
    "package.json",
];
const WASM_FILES: [&str; 4] = [
    "readable_uuid_wasm.js",
    "readable_uuid_wasm.d.ts",
    "readable_uuid_wasm_bg.wasm",
    "readable_uuid_wasm_bg.wasm.d.ts",
];

pub fn build(args: &[String]) -> Result {
    if !args.is_empty() {
        return Err("npm-build does not accept arguments".into());
    }

    let output = output_dir();
    if output.exists() {
        fs::remove_dir_all(&output)?;
    }

    let wasm_output = output.join("wasm");
    fs::create_dir_all(&wasm_output)?;

    run(Command::new("wasm-pack")
        .current_dir(binding_dir())
        .args(["build", "--target", "web", "--release", "--no-pack"])
        .arg("--out-dir")
        .arg(&wasm_output))?;

    for generated in [".gitignore", "package.json", "README.md"] {
        let path = wasm_output.join(generated);
        if path.exists() {
            fs::remove_file(path)?;
        }
    }

    for file in ["api.js", "browser.js", "index.js", "index.d.ts"] {
        fs::copy(binding_dir().join("npm").join(file), output.join(file))?;
    }
    fs::copy(
        binding_dir().join("README.npm.md"),
        output.join("README.md"),
    )?;
    fs::copy(root().join("LICENSE"), output.join("LICENSE"))?;

    let mut package = read_json(&binding_dir().join("package.template.json"))?;
    package["version"] = json!(version()?);
    write_json(&output.join("package.json"), &package)?;

    validate_generated_package()?;
    println!("Generated {}", output.display());
    Ok(())
}

pub fn pack() -> Result {
    build(&[])?;
    pack_directory(&output_dir(), "readable-uuid.tgz")
}

pub fn publish(dry_run: bool) -> Result {
    if !dry_run {
        require_clean_worktree()?;
    }

    build(&[])?;
    validate_generated_package()?;

    let mut command = pnpm(&output_dir());
    command.args([
        "publish",
        "--access",
        "public",
        "--ignore-scripts",
        "--no-git-checks",
    ]);
    if dry_run {
        command.arg("--dry-run");
    }
    run(&mut command)?;

    if dry_run {
        println!("npm package checked; nothing published.");
    } else {
        println!("Published npm package.");
    }
    Ok(())
}

fn validate_generated_package() -> Result<Value> {
    let package = read_json(&output_dir().join("package.json"))?;
    let template = read_json(&binding_dir().join("package.template.json"))?;

    if package["version"] != version()? || package["exports"] != template["exports"] {
        return Err("Generated package is stale; run just npm-build".into());
    }

    for file in PACKAGE_FILES {
        require_nonempty_file(&output_dir().join(file))?;
    }
    for file in WASM_FILES {
        require_nonempty_file(&output_dir().join("wasm").join(file))?;
    }

    Ok(package)
}

fn pack_directory(directory: &Path, filename: &str) -> Result {
    run(pnpm(directory).args(["pack", "--out", filename]))
}

fn require_nonempty_file(path: &Path) -> Result {
    if !path.is_file() || fs::metadata(path)?.len() == 0 {
        return Err(format!("Missing or empty package file: {}", path.display()).into());
    }
    Ok(())
}

fn require_clean_worktree() -> Result {
    let status = capture(
        Command::new("git")
            .current_dir(root())
            .args(["status", "--porcelain"]),
    )?;
    if !status.is_empty() {
        return Err("Commit release sources before publishing".into());
    }
    Ok(())
}

fn binding_dir() -> PathBuf {
    root().join("bindings/wasm")
}

fn output_dir() -> PathBuf {
    binding_dir().join("pkg-npm")
}

fn pnpm(directory: &Path) -> Command {
    let executable = if cfg!(windows) { "pnpm.cmd" } else { "pnpm" };
    let mut command = Command::new(executable);
    command.current_dir(directory);
    command
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn write_json(path: &Path, value: &Value) -> Result {
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}
