use std::{
    env,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::Result;

fn main() -> Result<()> {
    match env::args().nth(1) {
        Some(task) if task == "build" => build()?,
        _ => usage(),
    }

    Ok(())
}

fn usage() {
    println!(
        r#"
Usage: cargo xtask COMMAND

COMMANDS:

  build  -  Build the interface and the firmware, bundling them together
    "#
    );
}

fn build() -> Result<()> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = workspace.parent().unwrap().canonicalize()?;

    let client_path = workspace.join("client");
    let server_path = workspace.join("server");

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
