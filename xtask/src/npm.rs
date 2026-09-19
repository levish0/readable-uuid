use crate::{Result, capture, root, run, version};
use serde_json::{Map, Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const PACKAGE_FILES: [&str; 4] = ["index.js", "index.d.ts", "README.md", "LICENSE"];

struct Artifact {
    platform: &'static str,
    filename: String,
    path: PathBuf,
}

pub fn build(args: &[String]) -> Result {
    let target = build_target(args)?;
    let platform = platform_suffix(&target)?;
    let mut package = read_json(&binding_dir().join("package.template.json"))?;

    if !configured_targets(&package)?
        .iter()
        .any(|value| value == &target)
    {
        return Err("Target not listed in package template".into());
    }

    let package_version = version()?;
    package["version"] = json!(package_version);

    fs::create_dir_all(output_dir())?;
    write_json(&output_dir().join("package.json"), &package)?;
    build_native_binding(&target)?;

    fs::copy(
        binding_dir().join("README.npm.md"),
        output_dir().join("README.md"),
    )?;
    fs::copy(root().join("LICENSE"), output_dir().join("LICENSE"))?;

    let artifact_name = format!("readable-uuid.{platform}");
    let binary = output_dir().join(format!("{artifact_name}.node"));
    require_nonempty_file(&binary)?;

    let provenance = json!({
        "version": package_version,
        "revision": git_revision()?,
        "target": target,
    });
    write_json(
        &output_dir().join(format!("{artifact_name}.json")),
        &provenance,
    )?;

    println!("Generated {}", output_dir().display());
    Ok(())
}

pub fn pack() -> Result {
    validate_generated_package()?;
    pack_directory(&output_dir(), "readable-uuid-local.tgz")
}

pub fn publish(dry_run: bool) -> Result {
    let package = validate_generated_package()?;
    let artifacts = validate_artifacts(&package)?;

    if !dry_run {
        require_clean_worktree()?;
    }

    let version = package["version"]
        .as_str()
        .ok_or("Missing package version")?;
    let stage = binding_dir().join("release").join(version);
    let mut directories = Vec::new();
    let mut optional_dependencies = Map::new();

    for artifact in &artifacts {
        let (directory, package_name) = stage_platform_package(&stage, &package, artifact)?;
        optional_dependencies.insert(package_name, json!(version));
        directories.push(directory);
    }

    directories.push(stage_loader_package(
        &stage,
        &package,
        optional_dependencies,
    )?);

    // Pack every package before the first upload. The root loader is published last.
    for directory in &directories {
        pack_directory(directory, "package.tgz")?;
    }

    if dry_run {
        println!("Release packages packed; nothing published.");
        return Ok(());
    }

    for directory in &directories {
        run(pnpm(directory).args([
            "publish",
            "--access",
            "public",
            "--ignore-scripts",
            "--no-git-checks",
        ]))?;
    }

    println!("Published platform packages and root package.");
    Ok(())
}

fn build_target(args: &[String]) -> Result<String> {
    match args {
        [] => {
            let compiler = capture(Command::new("rustc").arg("-vV"))?;
            compiler
                .lines()
                .find_map(|line| line.strip_prefix("host: "))
                .map(str::to_owned)
                .ok_or_else(|| "Missing rustc host".into())
        }
        [flag, target] if flag == "--target" => Ok(target.clone()),
        _ => Err("Expected npm-build [--target <triple>]".into()),
    }
}

fn build_native_binding(target: &str) -> Result {
    let cli = binding_dir().join("node_modules/@napi-rs/cli/dist/cli.js");
    if !cli.is_file() {
        return Err("Missing NAPI-RS CLI; run just js-install".into());
    }

    run(Command::new("node")
        .current_dir(binding_dir())
        .arg(cli)
        .args([
            "build",
            "--manifest-path",
            "Cargo.toml",
            "--package-json-path",
            "pkg-npm/package.json",
            "--output-dir",
            "pkg-npm",
            "--platform",
            "--release",
            "--target",
            target,
        ]))
}

fn validate_generated_package() -> Result<Value> {
    let package = read_json(&output_dir().join("package.json"))?;
    let template = read_json(&binding_dir().join("package.template.json"))?;

    if package["version"] != version()? || package["napi"] != template["napi"] {
        return Err("Generated package is stale; run just npm-build".into());
    }

    for file in PACKAGE_FILES {
        require_nonempty_file(&output_dir().join(file))?;
    }

    Ok(package)
}

fn validate_artifacts(package: &Value) -> Result<Vec<Artifact>> {
    let revision = git_revision()?;
    let mut artifacts = Vec::new();

    for target in configured_targets(package)? {
        let target = target.as_str().ok_or("Invalid target")?;
        let platform = platform_suffix(target)?;
        let filename = format!("readable-uuid.{platform}.node");
        let path = binding_dir().join("artifacts").join(&filename);

        require_nonempty_file(&path)?;
        let provenance = read_json(&path.with_extension("json"))?;
        if provenance["version"] != package["version"]
            || provenance["revision"] != revision
            || provenance["target"] != target
        {
            return Err(format!("Artifact version/revision/target mismatch: {filename}").into());
        }

        artifacts.push(Artifact {
            platform,
            filename,
            path,
        });
    }

    Ok(artifacts)
}

fn stage_platform_package(
    stage: &Path,
    package: &Value,
    artifact: &Artifact,
) -> Result<(PathBuf, String)> {
    let directory = stage.join(artifact.platform);
    let package_name = format!(
        "{}-{}",
        package["name"].as_str().ok_or("Missing package name")?,
        artifact.platform,
    );
    let mut parts = artifact.platform.split('-');
    let os = parts.next().ok_or("Missing platform OS")?;
    let cpu = parts.next().ok_or("Missing platform CPU")?;

    let mut metadata = json!({
        "name": package_name,
        "version": package["version"],
        "description": package["description"],
        "license": package["license"],
        "repository": package["repository"],
        "engines": package["engines"],
        "main": artifact.filename,
        "files": [artifact.filename, "LICENSE"],
        "os": [os],
        "cpu": [cpu],
    });
    if os == "linux" {
        metadata["libc"] = json!(["glibc"]);
    }

    fs::create_dir_all(&directory)?;
    write_json(&directory.join("package.json"), &metadata)?;
    fs::copy(&artifact.path, directory.join(&artifact.filename))?;
    fs::copy(root().join("LICENSE"), directory.join("LICENSE"))?;

    Ok((directory, package_name))
}

fn stage_loader_package(
    stage: &Path,
    package: &Value,
    optional_dependencies: Map<String, Value>,
) -> Result<PathBuf> {
    let directory = stage.join("main");
    let metadata = json!({
        "name": package["name"],
        "version": package["version"],
        "description": package["description"],
        "license": package["license"],
        "repository": package["repository"],
        "engines": package["engines"],
        "main": package["main"],
        "types": package["types"],
        "files": PACKAGE_FILES,
        "optionalDependencies": optional_dependencies,
    });

    fs::create_dir_all(&directory)?;
    for file in PACKAGE_FILES {
        fs::copy(output_dir().join(file), directory.join(file))?;
    }
    write_json(&directory.join("package.json"), &metadata)?;

    Ok(directory)
}

fn pack_directory(directory: &Path, filename: &str) -> Result {
    run(pnpm(directory).args(["pack", "--out", filename]))
}

fn require_nonempty_file(path: &Path) -> Result {
    if !path.is_file() || fs::metadata(path)?.len() == 0 {
        return Err(format!("Missing or empty artifact: {}", path.display()).into());
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

fn configured_targets(package: &Value) -> Result<&Vec<Value>> {
    package["napi"]["targets"]
        .as_array()
        .ok_or_else(|| "Missing napi.targets".into())
}

fn platform_suffix(target: &str) -> Result<&'static str> {
    match target {
        "x86_64-pc-windows-msvc" => Ok("win32-x64-msvc"),
        "x86_64-unknown-linux-gnu" => Ok("linux-x64-gnu"),
        "aarch64-unknown-linux-gnu" => Ok("linux-arm64-gnu"),
        "aarch64-apple-darwin" => Ok("darwin-arm64"),
        "x86_64-apple-darwin" => Ok("darwin-x64"),
        _ => Err(format!("Unsupported npm target: {target}").into()),
    }
}

fn binding_dir() -> PathBuf {
    root().join("bindings/node")
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

fn git_revision() -> Result<String> {
    capture(
        Command::new("git")
            .current_dir(root())
            .args(["rev-parse", "HEAD"]),
    )
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn write_json(path: &Path, value: &Value) -> Result {
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}
