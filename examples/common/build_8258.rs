use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=TC32_LLVM_BIN");
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"));
    let examples_dir = manifest_dir.parent().expect("example crate lives under examples/");
    let common_dir = examples_dir.join("common");
    let repo_root = examples_dir
        .parent()
        .and_then(Path::parent)
        .expect("workspace root lives under repository root");
    let llvm_bin =
        resolve_path("TC32_LLVM_BIN", repo_root, || repo_root.join("build-tc32-triple/bin"));
    let clang = llvm_bin.join("clang");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set"));
    let object_dir = out_dir.join("objects");
    fs::create_dir_all(&object_dir).expect("create object dir");

    let include_dirs = [common_dir.join("include")];

    let common_flags = [
        "-DMCU_CORE_8258=1",
        "-DMCU_STARTUP_8258=1",
        "-O2",
        "-ffunction-sections",
        "-fdata-sections",
        "-Wall",
        "-fpack-struct",
        "-fshort-enums",
        "-std=gnu99",
        "-fshort-wchar",
        "-fms-extensions",
        "-ffreestanding",
        "-nostdlib",
        "-fno-unwind-tables",
        "-fno-asynchronous-unwind-tables",
        "-fno-exceptions",
    ];

    let asm_flags = [
        "-x",
        "assembler-with-cpp",
        "-fomit-frame-pointer",
        "-fshort-enums",
        "-Wall",
        "-Wpacked",
        "-Wcast-align",
        "-fdata-sections",
        "-ffunction-sections",
        "-fno-use-cxa-atexit",
        "-fno-threadsafe-statics",
        "-ffreestanding",
        "-nostdlib",
    ];

    let sources = [
        common_dir.join("support/link_cfg.S"),
        common_dir.join("support/cstartup_8258.S"),
    ];
    let mut objects = Vec::new();
    for source in &sources {
        println!("cargo:rerun-if-changed={}", source.display());
        let ext = source.extension().and_then(|s| s.to_str()).unwrap_or("");
        let object = object_dir.join(object_name(source));
        let mut command = Command::new(&clang);
        command.arg("--target=tc32").arg("-c");
        if ext.eq_ignore_ascii_case("S") {
            command.args(asm_flags);
        } else {
            command.args(common_flags);
        }
        for dir in &include_dirs {
            command.arg("-I").arg(dir);
        }
        command.arg("-o").arg(&object).arg(source);
        run(&mut command, source);
        objects.push(object);
    }

    for header in [
        common_dir.join("include/comm_cfg.h"),
        common_dir.join("include/version_cfg.h"),
        common_dir.join("boot_8258_minimal_lld.link"),
    ] {
        println!("cargo:rerun-if-changed={}", header.display());
    }

    println!("cargo:rustc-link-arg=--gc-sections");
    println!("cargo:rustc-link-arg=-u");
    println!("cargo:rustc-link-arg=ss_apsmeSwitchKeyReq");
    println!("cargo:rustc-link-arg=-z");
    println!("cargo:rustc-link-arg=max-page-size=0x8000");
    println!("cargo:rustc-link-arg=-z");
    println!("cargo:rustc-link-arg=common-page-size=0x8000");
    println!(
        "cargo:rustc-link-arg=-T{}",
        common_dir.join("boot_8258_minimal_lld.link").display()
    );
    for object in &objects {
        println!("cargo:rustc-link-arg={}", object.display());
    }

}

fn resolve_path(key: &str, repo_root: &Path, default: impl FnOnce() -> PathBuf) -> PathBuf {
    match env::var_os(key) {
        Some(value) => {
            let path = PathBuf::from(value);
            if path.is_absolute() {
                path
            } else {
                repo_root.join(path)
            }
        }
        None => default(),
    }
}

fn object_name(source: &Path) -> OsString {
    let file_name = source.file_name().expect("source file name");
    let mut name = file_name.to_os_string();
    name.push(".o");
    name
}

fn run(command: &mut Command, source: &Path) {
    let status = command
        .status()
        .unwrap_or_else(|err| panic!("failed to launch compiler for {}: {err}", source.display()));
    if !status.success() {
        panic!("compilation failed for {}", source.display());
    }
}
