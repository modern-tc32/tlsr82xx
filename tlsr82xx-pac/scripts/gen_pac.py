#!/usr/bin/env python3
from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path


SUPPORTED_CHIPS = ("8258", "8278", "826x")


def require_tool(name: str) -> str:
    path = shutil.which(name)
    if path is None:
        raise RuntimeError(f"required tool not found in PATH: {name}")
    return path


def run(cmd: list[str], cwd: Path) -> None:
    subprocess.run(cmd, cwd=cwd, check=True)


def generate_pac(chip: str, svd_path: Path, out_dir: Path) -> Path:
    require_tool("svd2rust")
    require_tool("form")

    work_dir = out_dir / chip
    src_dir = work_dir / "src"
    src_dir.mkdir(parents=True, exist_ok=True)

    run(
        [
            "svd2rust",
            "--input",
            str(svd_path),
            "--target",
            "generic",
        ],
        cwd=work_dir,
    )

    lib_rs = work_dir / "lib.rs"
    if not lib_rs.exists():
        raise RuntimeError(f"svd2rust did not produce {lib_rs}")

    run(["form", "-i", "lib.rs", "-o", "src/"], cwd=work_dir)

    mod_rs = src_dir / "lib.rs"
    if mod_rs.exists():
        mod_rs.unlink()
    lib_rs.rename(mod_rs)

    cargo_toml = work_dir / "Cargo.toml"
    cargo_toml.write_text(
        "\n".join(
            [
                "[package]",
                f'name = "tlsr82xx-pac-{chip}"',
                'version = "0.1.0"',
                'edition = "2021"',
                'license = "Apache-2.0"',
                'publish = false',
                "",
                "[dependencies]",
                'critical-section = { version = "1", optional = true }',
                'vcell = "0.1.3"',
                "",
                "[features]",
                'default = []',
                "",
            ]
        )
        + "\n",
        encoding="utf-8",
    )

    return work_dir


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate Rust PAC sources from SVD")
    parser.add_argument("--chip", required=True, choices=SUPPORTED_CHIPS)
    parser.add_argument("--svd", required=True, type=Path)
    parser.add_argument("--out-dir", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    svd_path = args.svd.resolve()
    out_dir = args.out_dir.resolve()

    if not svd_path.exists():
        print(f"error: SVD file does not exist: {svd_path}", file=sys.stderr)
        return 2

    try:
        work_dir = generate_pac(args.chip, svd_path, out_dir)
    except Exception as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    print(work_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
