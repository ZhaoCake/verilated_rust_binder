use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir
        .parent()
        .expect("rust crate must be under workspace root")
        .to_path_buf();
    let rtl_dir = workspace_dir.join("rtl");
    let generator = workspace_dir.join("scripts/gen_verilator_binder.py");
    let top = env::var("VERILATOR_TOP").unwrap_or_else(|_| "top".to_string());

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("missing OUT_DIR"));
    let verilator_dir = out_dir.join("verilator_obj_dir");
    fs::create_dir_all(&verilator_dir).expect("failed to create verilator output dir");

    let rtl_sources = collect_rtl_sources(&rtl_dir);
    if rtl_sources.is_empty() {
        panic!("No rtl source found in {}", rtl_dir.display());
    }

    println!("cargo:rerun-if-env-changed=VERILATOR_TOP");
    println!("cargo:rerun-if-env-changed=VERILATOR_ROOT");
    println!("cargo:rerun-if-changed={}", generator.display());
    for source in &rtl_sources {
        println!("cargo:rerun-if-changed={}", source.display());
    }

    run_verilator(&top, &rtl_sources, &rtl_dir, &verilator_dir);

    let header = verilator_dir.join(format!("V{top}.h"));
    let bridge_cpp = out_dir.join("verilator_rust_bridge.cpp");
    let generated_rs = out_dir.join("binder.rs");
    run_generator(&generator, &header, &top, &bridge_cpp, &generated_rs);

    run_make(&top, &verilator_dir);

    let verilator_root = detect_verilator_root();
    let verilator_include = verilator_root.join("include");
    let verilator_vltstd = verilator_root.join("include/vltstd");

    cc::Build::new()
        .cpp(true)
        .warnings(false)
        .flag_if_supported("-std=c++17")
        .file(&bridge_cpp)
        .file(verilator_include.join("verilated.cpp"))
        .file(verilator_include.join("verilated_threads.cpp"))
        .include(&verilator_dir)
        .include(verilator_include)
        .include(verilator_vltstd)
        .compile("vrb_bridge");

    println!("cargo:rustc-link-search=native={}", verilator_dir.display());
    println!("cargo:rustc-link-lib=static=V{top}__ALL");
    println!("cargo:rustc-link-lib=stdc++");
}

fn collect_rtl_sources(rtl_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    let entries = fs::read_dir(rtl_dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", rtl_dir.display()));

    for entry in entries {
        let path = entry.expect("failed to read dir entry").path();
        let extension = path.extension().and_then(|ext| ext.to_str());
        if matches!(extension, Some("sv") | Some("v")) {
            files.push(path);
        }
    }

    files.sort();
    files
}

fn run_verilator(top: &str, sources: &[PathBuf], rtl_dir: &Path, verilator_dir: &Path) {
    let mut command = Command::new("verilator");
    command
        .arg("--cc")
        .arg("--Mdir")
        .arg(verilator_dir)
        .arg("--top-module")
        .arg(top)
        .arg("-Wall")
        .arg("-Wno-fatal")
        .arg(format!("-I{}", rtl_dir.display()));

    for source in sources {
        command.arg(source);
    }

    run_checked(&mut command, "verilator code generation failed");
}

fn run_generator(generator: &Path, header: &Path, top: &str, out_cpp: &Path, out_rs: &Path) {
    let mut command = Command::new("python3");
    command
        .arg(generator)
        .arg("--header")
        .arg(header)
        .arg("--top")
        .arg(top)
        .arg("--out-cpp")
        .arg(out_cpp)
        .arg("--out-rs")
        .arg(out_rs);

    run_checked(&mut command, "binder generation script failed");
}

fn run_make(top: &str, verilator_dir: &Path) {
    let archive = verilator_dir.join(format!("V{top}__ALL.a"));
    let rust_archive = verilator_dir.join(format!("libV{top}__ALL.a"));

    let mut command = Command::new("make");
    command
        .arg("-C")
        .arg(verilator_dir)
        .arg("-f")
        .arg(format!("V{top}.mk"))
        .arg(format!("V{top}__ALL.a"));

    run_checked(&mut command, "building Verilator static archive failed");

    if !archive.exists() {
        panic!("expected archive not found: {}", archive.display());
    }

    fs::copy(&archive, &rust_archive).unwrap_or_else(|err| {
        panic!(
            "failed to create rust-style archive {} from {}: {err}",
            rust_archive.display(),
            archive.display()
        )
    });
}

fn detect_verilator_root() -> PathBuf {
    if let Ok(root) = env::var("VERILATOR_ROOT") {
        return PathBuf::from(root);
    }

    let output = Command::new("verilator")
        .arg("-V")
        .output()
        .expect("failed to run `verilator -V` for VERILATOR_ROOT detection");

    if !output.status.success() {
        panic!("`verilator -V` failed while detecting VERILATOR_ROOT");
    }

    let stdout = String::from_utf8(output.stdout)
        .expect("verilator -V stdout should be valid UTF-8 for parsing");

    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("VERILATOR_ROOT") {
            continue;
        }

        if let Some((_, value)) = trimmed.split_once('=') {
            return PathBuf::from(value.trim());
        }
    }

    panic!("Cannot detect VERILATOR_ROOT from `verilator -V` output")
}

fn run_checked(command: &mut Command, context: &str) {
    let status = command
        .status()
        .unwrap_or_else(|err| panic!("{context}: {err}"));

    if !status.success() {
        panic!("{context} (status: {status})");
    }
}
