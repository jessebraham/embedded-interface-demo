use std::{
    env,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::Result;

fn main() -> Result<()> {
    // The Cargo workspace is the parent directory of the path containing the
    // 'xtask' package's Cargo manifest (ie. Cargo.toml).
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = workspace.parent().unwrap().canonicalize()?;

    // Both the 'client' and 'server' projects are children of the workspace
    // directory.
    let client_path = workspace.join("client");
    let server_path = workspace.join("server");

    // If a valid task has been given, execute it. Otherwise, print the usage
    // message and terminate.
    if let Some(task) = env::args().nth(1) {
        match task.as_str() {
            "build" => build(&client_path, &server_path)?,
            "flash" => flash(&client_path, &server_path)?,
            _ => usage(),
        }
    } else {
        usage();
    }

    Ok(())
}

fn usage() {
    println!(
        r#"
Usage: cargo xtask TASK

TASKS:

  build  -  Build the interface and the firmware, bundling them together
  flash  -  Upload the firmware to the connected device, building if necessary
    "#
    );
}

// ---------------------------------------------------------------------------
// Tasks

fn build(client_path: &PathBuf, server_path: &PathBuf) -> Result<()> {
    eprintln!("\nBuilding 'client' project...");
    build_client(&client_path)?;

    eprintln!("\nCopying distribution artifact to server resources...");
    let src = client_path.join("dist/index.html.gz");
    let dst = server_path.join("resources/index.html.gz");
    fs::copy(src, dst)?;

    eprintln!("\nBuilding 'server' project...\n");
    build_server(&server_path)?;

    Ok(())
}

fn flash(client_path: &PathBuf, server_path: &PathBuf) -> Result<()> {
    println!("foo");

    if !release_artifact_exists(&server_path)? {
        eprintln!("\nUnable to locate relase binary, building...");
        build(&client_path, &server_path)?;
    }

    eprintln!("\nFlashing firmware to device...\n");
    cargo_espflash(&server_path)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Helper Functions

fn build_client(path: &PathBuf) -> Result<()> {
    Command::new("npm")
        .args(["run", "prod"])
        .current_dir(path)
        .stdout(Stdio::inherit())
        .output()?;

    Ok(())
}

fn build_server(path: &PathBuf) -> Result<()> {
    Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(path)
        .stderr(Stdio::inherit())
        .output()?;

    Ok(())
}

fn cargo_espflash(path: &PathBuf) -> Result<()> {
    Command::new("cargo")
        .args(["espflash", "--release", "--monitor"])
        .current_dir(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    Ok(())
}

fn release_artifact_exists(server_path: &PathBuf) -> Result<bool> {
    let workspace_path = server_path.parent().unwrap();
    let bin_path = workspace_path.join("target/riscv32imc-esp-espidf/release/server");

    let exists = match bin_path.canonicalize() {
        Ok(path) => path.exists() && path.is_file(),
        Err(_) => false,
    };

    Ok(exists)
}
