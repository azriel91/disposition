//! Development tasks for the `disposition` workspace.
//!
//! Run with `cargo run -p xtask -- <task>`.
//!
//! Tasks:
//!
//! * `build-worker` -- compiles `disposition_lsp_worker` to wasm and runs
//!   `wasm-bindgen` to emit the Web Worker bundle into the playground's
//!   `assets/lsp_worker/` folder. Required before serving the playground with
//!   `--features lsp-worker`.

use std::{
    path::{Path, PathBuf},
    process::{exit, Command},
};

use wasm_bindgen_cli_support::Bindgen;

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("build-worker") => build_worker(),
        other => {
            if let Some(task) = other {
                eprintln!("unknown task: {task}");
            }
            eprintln!("usage: cargo run -p xtask -- build-worker");
            exit(1);
        }
    }
}

/// Builds the LSP Web Worker wasm bundle into `assets/lsp_worker/`.
fn build_worker() {
    let workspace_root = workspace_root();
    let worker_wasm = workspace_root
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("disposition_lsp_worker.wasm");
    let out_dir = workspace_root
        .join("app")
        .join("playground")
        .join("assets")
        .join("lsp_worker");

    // 1. Compile the worker crate to wasm.
    run(
        &workspace_root,
        "cargo",
        &[
            "build",
            "-p",
            "disposition_lsp_worker",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
        ],
    );

    // 2. Generate the JS + wasm bundle the worker entry imports (ES module for
    //    `--target web`, no TypeScript). Run via the library so the bindgen version
    //    always matches the worker crate's `wasm-bindgen`.
    Bindgen::new()
        .input_path(&worker_wasm)
        .web(true)
        .expect("enable web target")
        .typescript(false)
        .generate(&out_dir)
        .unwrap_or_else(|error| {
            eprintln!("wasm-bindgen failed: {error}");
            exit(1);
        });

    println!("Built LSP worker bundle into {}", out_dir.display());
}

/// The workspace root, derived from this crate's manifest directory
/// (`<root>/xtask`).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask manifest dir has a parent")
        .to_path_buf()
}

/// Runs `program` with `args` in `dir`, exiting on failure.
fn run(dir: &Path, program: &str, args: &[&str]) {
    let status = Command::new(program)
        .current_dir(dir)
        .args(args)
        .status()
        .unwrap_or_else(|error| {
            eprintln!("failed to run {program}: {error}");
            exit(1);
        });
    if !status.success() {
        eprintln!("{program} {args:?} failed with {status}");
        exit(1);
    }
}
