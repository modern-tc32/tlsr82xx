use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=TC32_LLVM_BIN");
    println!("cargo:rerun-if-env-changed=TC_BLE_SDK_ROOT");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let examples_dir = manifest_dir.parent().expect("example crate lives under examples/");
    let common_dir = examples_dir.join("common");
    let repo_root = examples_dir
        .parent()
        .and_then(Path::parent)
        .expect("workspace root lives under repository root");

    let llvm_bin =
        resolve_path("TC32_LLVM_BIN", repo_root, || repo_root.join("build-tc32-triple/bin"));
    let sdk_root = resolve_path("TC_BLE_SDK_ROOT", repo_root, || {
        repo_root
            .parent()
            .expect("repo root must have parent")
            .join("tc_ble_single_sdk/tc_ble_single_sdk")
    });
    let clang = llvm_bin.join("clang");
    let vendor_gcc = repo_root.join("tc32-vendor/bin/tc32-elf-gcc");
    let ble_sample_root = manifest_dir.join("vendor_ble_sample");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let object_dir = out_dir.join("objects");
    fs::create_dir_all(&object_dir).expect("create object dir");

    let include_dirs = [
        ble_sample_root.clone(),
        ble_sample_root.join("vendor/ble_sample"),
        sdk_root.clone(),
        sdk_root.join("common"),
        sdk_root.join("vendor/common"),
        sdk_root.join("vendor/common/boards"),
        sdk_root.join("drivers/B85"),
        sdk_root.join("drivers/B85/driver_ext"),
        sdk_root.join("drivers/B85/flash"),
        sdk_root.join("stack/ble"),
        common_dir.join("include"),
    ];

    let c_flags = [
        "-DMCU_CORE_8258=1",
        "-DMCU_STARTUP_8258=1",
        "-D__PROJECT_8258_BLE_SAMPLE__=1",
        "-DCHIP_TYPE=CHIP_TYPE_825x",
        "-D__TL_LIB_8258__=1",
        "-D__TLSR_RISCV_EN__=0",
        "-D__NO_INLINE__=__attribute__((noinline))",
        "-D__STATIC_INLINE=static inline",
        "-O2",
        "-ffunction-sections",
        "-fdata-sections",
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
        "-Wno-macro-redefined",
    ];
    let asm_flags = [
        "-x",
        "assembler-with-cpp",
        "-fomit-frame-pointer",
        "-fshort-enums",
        "-fdata-sections",
        "-ffunction-sections",
        "-ffreestanding",
        "-nostdlib",
    ];
    let mut sources: Vec<PathBuf> = vec![
        common_dir.join("support/link_cfg.S"),
        sdk_root.join("boot/B85/cstartup_825x.S"),
        manifest_dir.join("support/tlkapi_debug_stub.c"),
        sdk_root.join("common/div_mod.S"),
        ble_sample_root.join("main.c"),
        ble_sample_root.join("app.c"),
        ble_sample_root.join("app_att.c"),
        ble_sample_root.join("app_ui.c"),
        sdk_root.join("common/utility.c"),
        sdk_root.join("common/string.c"),
    ];

    sources.extend(
        glob_c_sources(&sdk_root.join("vendor/common"))
            .into_iter()
            .filter(|p| p.file_name().and_then(|s| s.to_str()) != Some("tlkapi_debug.c")),
    );
    sources.extend(glob_c_sources(&sdk_root.join("drivers/B85")));
    sources.extend(glob_c_sources(&sdk_root.join("drivers/B85/driver_ext")));
    sources.extend(glob_c_sources(&sdk_root.join("drivers/B85/flash")));

    let mut objects = Vec::new();
    for source in &sources {
        if !source.exists() {
            continue;
        }
        println!("cargo:rerun-if-changed={}", source.display());
        let ext = source.extension().and_then(|s| s.to_str()).unwrap_or("");
        let object = object_dir.join(object_name(source));
        if source == &sdk_root.join("common/div_mod.S") {
            let mut gcc = Command::new(&vendor_gcc);
            gcc.arg("-c")
                .arg("-x")
                .arg("assembler-with-cpp")
                .arg("-DMCU_CORE_8258=1")
                .arg("-DMCU_STARTUP_8258=1")
                .arg("-D__TL_LIB_8258__=1")
                .arg("-ffunction-sections")
                .arg("-fdata-sections")
                .arg("-o")
                .arg(&object)
                .arg(source);
            run(&mut gcc, source);
            objects.push(object);
            continue;
        }
        if ext.eq_ignore_ascii_case("S")
            && (source == &common_dir.join("support/link_cfg.S"))
        {
            let mut command = Command::new(&clang);
            command.arg("--target=tc32").arg("-c");
            command.args(asm_flags);
            for dir in &include_dirs {
                command.arg("-I").arg(dir);
            }
            command.arg("-o").arg(&object).arg(source);
            run(&mut command, source);
            objects.push(object);
            continue;
        }
        if source == &sdk_root.join("drivers/B85/pm.c") {
            // Keep PM path from prebuilt vendor library for startup stability.
            continue;
        }
        if ext.eq_ignore_ascii_case("S")
            || source.starts_with(&sdk_root)
            || source.starts_with(&ble_sample_root)
        {
            let mut gcc = Command::new(&vendor_gcc);
            gcc.arg("-c");
            if ext.eq_ignore_ascii_case("S") {
                gcc.arg("-x").arg("assembler-with-cpp");
                gcc.arg("-ffunction-sections").arg("-fdata-sections");
            } else {
                gcc.arg("-DMCU_CORE_8258=1")
                    .arg("-DMCU_STARTUP_8258=1")
                    .arg("-D__PROJECT_8258_BLE_SAMPLE__=1")
                    .arg("-DCHIP_TYPE=CHIP_TYPE_825x")
                    .arg("-D__TL_LIB_8258__=1")
                    .arg("-D__TLSR_RISCV_EN__=0")
                    .arg("-D__NO_INLINE__=__attribute__((noinline))")
                    .arg("-D__STATIC_INLINE=static inline")
                    .arg("-O2")
                    .arg("-ffunction-sections")
                    .arg("-fdata-sections")
                    .arg("-fpack-struct")
                    .arg("-fshort-enums")
                    .arg("-std=gnu99")
                    .arg("-fshort-wchar")
                    .arg("-fms-extensions")
                    .arg("-ffreestanding")
                    .arg("-nostdlib")
                    .arg("-fno-unwind-tables")
                    .arg("-fno-asynchronous-unwind-tables")
                    .arg("-fno-exceptions");
            }
            gcc.arg("-DMCU_CORE_8258=1")
                .arg("-DMCU_STARTUP_8258=1")
                .arg("-D__PROJECT_8258_BLE_SAMPLE__=1")
                .arg("-DCHIP_TYPE=CHIP_TYPE_825x")
                .arg("-D__TL_LIB_8258__=1")
                .arg("-D__TLSR_RISCV_EN__=0");
            for dir in &include_dirs {
                gcc.arg("-I").arg(dir);
            }
            gcc.arg("-o").arg(&object).arg(source);
            run(&mut gcc, source);
            objects.push(object);
            continue;
        }
        {
            let mut command = Command::new(&clang);
            command.arg("--target=tc32").arg("-c");
            command.args(c_flags);
            for dir in &include_dirs {
                command.arg("-I").arg(dir);
            }
            command.arg("-o").arg(&object).arg(source);
            run(&mut command, source);
            objects.push(object);
        }
    }

    let sdk_libs = [
        sdk_root.join("proj_lib/liblt_825x.a"),
        sdk_root.join("proj_lib/liblt_general_stack.a"),
    ];
    for lib in &sdk_libs {
        println!("cargo:rerun-if-changed={}", lib.display());
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

    for lib in &sdk_libs {
        println!("cargo:rustc-link-arg={}", lib.display());
    }
}

fn glob_c_sources(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("c") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
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
    let mut normalized = String::new();
    for ch in source.to_string_lossy().chars() {
        let mapped = if ch.is_ascii_alphanumeric() { ch } else { '_' };
        normalized.push(mapped);
    }
    OsString::from(format!("{normalized}.o"))
}

fn run(command: &mut Command, source: &Path) {
    let status = command
        .status()
        .unwrap_or_else(|err| panic!("failed to launch compiler for {}: {err}", source.display()));
    if !status.success() {
        panic!("compilation failed for {}", source.display());
    }
}
