mod npm;

use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};
type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_owned()
}

fn run(command: &mut Command) -> Result {
    let status = command.status()?;
    if !status.success() {
        return Err(format!("{command:?} failed: {status}").into());
    }
    Ok(())
}

fn capture(command: &mut Command) -> Result<String> {
    let output = command.output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn version() -> Result<String> {
    let data: serde_json::Value =
        serde_json::from_str(&capture(Command::new("cargo").current_dir(root()).args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
        ]))?)?;
    let packages = data["packages"]
        .as_array()
        .ok_or("Missing Cargo packages")?;
    Ok(packages
        .iter()
        .find(|p| p["name"] == "readable-uuid")
        .and_then(|p| p["version"].as_str())
        .ok_or("Missing root package version")?
        .into())
}

fn publish_crate(dry_run: bool, flags: &[String]) -> Result {
    let mut command = Command::new("cargo");
    command
        .current_dir(root())
        .args(["publish", "-p", "readable-uuid"]);
    if dry_run {
        command.arg("--dry-run");
    }
    run(command.args(flags))
}

fn release(dry_run: bool, cargo_flags: &[String]) -> Result {
    if dry_run {
        publish_crate(true, cargo_flags)?;
        return npm::publish(true);
    }

    // Complete both registry preflights before the first upload. The actual
    // uploads remain sequential because registries do not provide a shared
    // transaction.
    publish_crate(true, cargo_flags)?;
    npm::publish(true)?;
    publish_crate(false, cargo_flags)?;
    npm::publish(false)
}

fn execute() -> Result {
    let args: Vec<_> = env::args().skip(1).collect();
    let rest = args.get(1..).unwrap_or_default();
    match args.first().map(String::as_str).unwrap_or("help") {
        "publish" | "publish-dry" => publish_crate(args[0] == "publish-dry", rest),
        "release" | "release-dry" => release(args[0] == "release-dry", rest),
        "npm-build" => npm::build(rest),
        "npm-pack" if rest.is_empty() => npm::pack(),
        "npm-publish" if rest.is_empty() || rest == ["--dry-run"] => npm::publish(!rest.is_empty()),
        "help" => {
            println!(concat!(
                "cargo xtask publish[-dry] [cargo flags]\n",
                "cargo xtask release[-dry] [cargo flags]\n",
                "cargo xtask npm-build [--target <triple>]\n",
                "cargo xtask npm-pack\n",
                "cargo xtask npm-publish [--dry-run]",
            ));
            Ok(())
        }
        _ => Err("Unknown command or arguments; run cargo xtask help".into()),
    }
}

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
