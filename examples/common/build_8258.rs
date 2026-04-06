use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=TC32_LLVM_BIN");
    println!("cargo:rerun-if-env-changed=TC32_SDK_DIR");
    println!("cargo:rerun-if-env-changed=TC32_CLANG_COMPAT_DIR");

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"));
    let examples_dir = manifest_dir.parent().expect("example crate lives under examples/");
    let common_dir = examples_dir.join("common");
    let repo_root = examples_dir
        .parent()
        .and_then(Path::parent)
        .expect("workspace root lives under repository root");
    let sdk_dir = resolve_path("TC32_SDK_DIR", repo_root, || repo_root.join("tl_zigbee_sdk"));
    let clang_compat_dir = resolve_path("TC32_CLANG_COMPAT_DIR", repo_root, || {
        repo_root.join("test_lamp/cmake_example/clang_compat")
    });
    let llvm_bin =
        resolve_path("TC32_LLVM_BIN", repo_root, || repo_root.join("build-tc32-triple/bin"));
    let clang = llvm_bin.join("clang");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set"));
    let object_dir = out_dir.join("objects");
    fs::create_dir_all(&object_dir).expect("create object dir");

    let include_dirs = [
        common_dir.join("include"),
        sdk_dir.join("apps/common"),
        sdk_dir.join("proj"),
        sdk_dir.join("proj/common"),
        sdk_dir.join("platform"),
        sdk_dir.join("platform/chip_8258"),
    ];

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
        sdk_dir.join("platform/boot/link_cfg.S"),
        clang_compat_dir.join("mulsi3.c"),
        common_dir.join("support/irq_handler_stub.c"),
        common_dir.join("support/memset.c"),
        common_dir.join("support/indirect_call_r3.c"),
        common_dir.join("support/tc32_boot_init.c"),
        common_dir.join("support/cstartup_8258.S"),
        common_dir.join("support/platform_init_8258.c"),
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
        common_dir.join("include/app_cfg.h"),
        common_dir.join("include/board_8258_tb_03f.h"),
        common_dir.join("include/comm_cfg.h"),
        common_dir.join("include/version_cfg.h"),
        common_dir.join("boot_8258_minimal_lld.link"),
    ] {
        println!("cargo:rerun-if-changed={}", header.display());
    }

    let drivers = sdk_dir.join("platform/lib/libdrivers_8258.a");
    println!("cargo:rerun-if-changed={}", drivers.display());
    let filtered_drivers = filter_vendor_archive(&drivers, &out_dir, repo_root);
    let soft_fp = sdk_dir.join("platform/tc32/libsoft-fp.a");

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
    println!("cargo:rustc-link-arg=--start-group");
    println!("cargo:rustc-link-arg={}", filtered_drivers.display());
    println!("cargo:rustc-link-arg={}", soft_fp.display());
    println!("cargo:rustc-link-arg=--end-group");
}

fn filter_vendor_archive(drivers: &Path, out_dir: &Path, repo_root: &Path) -> PathBuf {
    let ar = repo_root.join("tc32-vendor/bin/tc32-elf-ar");
    let extract_dir = out_dir.join("libdrivers_8258_extract");
    let filtered = out_dir.join("libdrivers_8258_noanalog.a");

    if filtered.exists() {
        fs::remove_file(&filtered).expect("remove previous filtered drivers archive");
    }
    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir).expect("remove previous extracted drivers dir");
    }
    fs::create_dir_all(&extract_dir).expect("create extracted drivers dir");

    let mut extract = Command::new(&ar);
    extract.arg("x").arg(drivers);
    extract.current_dir(&extract_dir);
    run_named(&mut extract, "extract vendor drivers archive");

    let mut members: Vec<PathBuf> = fs::read_dir(&extract_dir)
        .expect("read extracted vendor drivers dir")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file())
        .filter(|path| {
            !matches!(
                path.file_name().and_then(|s| s.to_str()),
                Some("analog.o" | "clock.o" | "gpio.o" | "pm.o")
            )
        })
        .collect();
    members.sort();

    let mut archive = Command::new(&ar);
    archive.arg("crs").arg(&filtered);
    archive.args(&members);
    run_named(&mut archive, "create filtered vendor drivers archive");

    filtered
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

fn run_named(command: &mut Command, what: &str) {
    let status = command
        .status()
        .unwrap_or_else(|err| panic!("failed to launch {what}: {err}"));
    if !status.success() {
        panic!("{what} failed");
    }
}
