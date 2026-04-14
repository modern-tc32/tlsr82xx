use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=TC32_LLVM_BIN");
    println!("cargo:rerun-if-env-changed=TC32_AR");

    if env::var_os("CARGO_FEATURE_CHIP_8258").is_none() {
        return;
    }

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"));
    let workspace_root = manifest_dir.parent().expect("workspace root");
    let llvm_bin = resolve_path("TC32_LLVM_BIN", workspace_root, || {
        workspace_root
            .parent()
            .expect("repo root")
            .join("toolchains/tc32-stage2/llvm/bin")
    });
    let clang = llvm_bin.join("clang");
    let ar = resolve_ar(workspace_root);

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set"));
    let object_dir = out_dir.join("irq-asm");
    std::fs::create_dir_all(&object_dir).expect("create object dir");

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

    let mut objects = Vec::new();
    if env::var_os("CARGO_FEATURE_CUSTOM_IRQ_ENTRY").is_none() {
        let irq_entry = manifest_dir.join("asm/irq_entry_8258_tc32.S");
        println!("cargo:rerun-if-changed={}", irq_entry.display());
        objects.push(compile_asm(&clang, &asm_flags, &irq_entry, &object_dir));
    }
    if env::var_os("CARGO_FEATURE_CUSTOM_IRQ_HANDLER").is_none() {
        let irq_handler = manifest_dir.join("asm/irq_handler_8258_tc32.S");
        println!("cargo:rerun-if-changed={}", irq_handler.display());
        objects.push(compile_asm(&clang, &asm_flags, &irq_handler, &object_dir));
    }

    if objects.is_empty() {
        let vendor_pm_rc = workspace_root.parent().expect("repo root").join("drivers/pm_32k_rc.o");
        if !vendor_pm_rc.exists() {
            return;
        }
        println!("cargo:rerun-if-changed={}", vendor_pm_rc.display());
        objects.push(vendor_pm_rc);
        let vendor_pm_xtal = workspace_root
            .parent()
            .expect("repo root")
            .join("drivers/pm_32k_xtal.o");
        if vendor_pm_xtal.exists() {
            println!("cargo:rerun-if-changed={}", vendor_pm_xtal.display());
            objects.push(vendor_pm_xtal);
        }
    } else {
        let vendor_pm_rc = workspace_root.parent().expect("repo root").join("drivers/pm_32k_rc.o");
        if vendor_pm_rc.exists() {
            println!("cargo:rerun-if-changed={}", vendor_pm_rc.display());
            objects.push(vendor_pm_rc);
        }
        let vendor_pm_xtal = workspace_root
            .parent()
            .expect("repo root")
            .join("drivers/pm_32k_xtal.o");
        if vendor_pm_xtal.exists() {
            println!("cargo:rerun-if-changed={}", vendor_pm_xtal.display());
            objects.push(vendor_pm_xtal);
        }
    }

    let lib_name = "tlsr82xx_hal_irq_asm_8258";
    let archive = out_dir.join(format!("lib{lib_name}.a"));

    let mut ar_cmd = Command::new(ar);
    ar_cmd.arg("crs").arg(&archive);
    for object in &objects {
        ar_cmd.arg(object);
    }
    run(&mut ar_cmd, "archive irq asm");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static={lib_name}");
}

fn compile_asm(clang: &Path, asm_flags: &[&str], source: &Path, object_dir: &Path) -> PathBuf {
    let object = object_dir.join(
        source
            .file_name()
            .expect("source file name")
            .to_string_lossy()
            .to_string()
            + ".o",
    );
    let mut command = Command::new(clang);
    command.arg("--target=tc32").arg("-c").args(asm_flags);
    command.arg("-o").arg(&object).arg(source);
    run(&mut command, source.to_string_lossy().as_ref());
    object
}

fn resolve_path(key: &str, workspace_root: &Path, default: impl FnOnce() -> PathBuf) -> PathBuf {
    match env::var_os(key) {
        Some(value) => {
            let path = PathBuf::from(value);
            if path.is_absolute() {
                path
            } else {
                workspace_root.join(path)
            }
        }
        None => default(),
    }
}

fn resolve_ar(workspace_root: &Path) -> PathBuf {
    if let Some(value) = env::var_os("TC32_AR") {
        let path = PathBuf::from(value);
        if path.is_absolute() {
            return path;
        }
        return workspace_root.join(path);
    }

    let vendor = workspace_root.join("../tc32-vendor/bin/tc32-elf-ar");
    if vendor.exists() {
        return vendor;
    }

    PathBuf::from("ar")
}

fn run(command: &mut Command, what: &str) {
    let status = command
        .status()
        .unwrap_or_else(|err| panic!("failed to launch {what}: {err}"));
    if !status.success() {
        panic!("{what} failed");
    }
}
