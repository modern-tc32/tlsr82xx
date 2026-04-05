#!/usr/bin/env python3
from __future__ import annotations

import argparse
import ast
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, Iterable, List, Optional
from xml.dom import minidom
from xml.etree import ElementTree as ET


SECTION_NAME_RE = re.compile(
    r"([A-Za-z0-9_ /-]+?)\s+(?:registers|regs(?:\s+struct)?)\b",
    re.IGNORECASE,
)
DEFINE_RE = re.compile(r"^\s*#define\s+([A-Za-z_][A-Za-z0-9_]*)\s+(.+?)\s*$")
REG_DEFINE_RE = re.compile(
    r"^\s*#define\s+(reg_[A-Za-z0-9_]+)\s+REG_ADDR(8|16|32)\((.+)\)\s*$"
)
ENUM_START_RE = re.compile(r"^\s*enum\b")


@dataclass
class FieldDef:
    name: str
    bit_offset: int
    bit_width: int
    description: Optional[str] = None


@dataclass
class RegisterDef:
    name: str
    address: int
    width: int
    section: str
    aliases: List[str] = field(default_factory=list)
    fields: List[FieldDef] = field(default_factory=list)


@dataclass
class PeripheralDef:
    name: str
    description: str
    base_address: int
    registers: List[RegisterDef]


class ExpressionEvaluator:
    def __init__(self) -> None:
        self.raw: Dict[str, str] = {}
        self.values: Dict[str, int] = {}
        self._resolving: set[str] = set()

    def define(self, name: str, expr: str) -> None:
        if "(" in name:
            return
        expr = expr.split("//", 1)[0].split("/*", 1)[0].strip()
        if expr:
            self.raw[name] = expr

    def resolve(self, name: str) -> int:
        if name in self.values:
            return self.values[name]
        if name in self._resolving:
            raise ValueError(f"cyclic macro definition for {name}")
        if name not in self.raw:
            raise KeyError(name)
        self._resolving.add(name)
        value = self.eval_expr(self.raw[name])
        self._resolving.remove(name)
        self.values[name] = value
        return value

    def eval_expr(self, expr: str) -> int:
        expr = expr.strip()
        if not expr:
            raise ValueError("empty expression")
        expr = expr.replace("UL", "").replace("ul", "")
        expr = expr.replace("U", "").replace("u", "")
        tree = ast.parse(expr, mode="eval")
        return self._eval_node(tree.body)

    def _eval_node(self, node: ast.AST) -> int:
        if isinstance(node, ast.Constant) and isinstance(node.value, int):
            return int(node.value)
        if isinstance(node, ast.Name):
            return self.resolve(node.id)
        if isinstance(node, ast.BinOp):
            left = self._eval_node(node.left)
            right = self._eval_node(node.right)
            if isinstance(node.op, ast.Add):
                return left + right
            if isinstance(node.op, ast.Sub):
                return left - right
            if isinstance(node.op, ast.Mult):
                return left * right
            if isinstance(node.op, ast.Div):
                return left // right
            if isinstance(node.op, ast.FloorDiv):
                return left // right
            if isinstance(node.op, ast.LShift):
                return left << right
            if isinstance(node.op, ast.RShift):
                return left >> right
            if isinstance(node.op, ast.BitOr):
                return left | right
            if isinstance(node.op, ast.BitAnd):
                return left & right
            if isinstance(node.op, ast.BitXor):
                return left ^ right
            raise ValueError(f"unsupported binary operator in {ast.dump(node)}")
        if isinstance(node, ast.UnaryOp):
            operand = self._eval_node(node.operand)
            if isinstance(node.op, ast.Invert):
                return ~operand
            if isinstance(node.op, ast.UAdd):
                return +operand
            if isinstance(node.op, ast.USub):
                return -operand
            raise ValueError(f"unsupported unary operator in {ast.dump(node)}")
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
            func = node.func.id
            args = [self._eval_node(arg) for arg in node.args]
            if func == "BIT" and len(args) == 1:
                return 1 << args[0]
            if func == "BIT_RNG" and len(args) == 2:
                start, end = args
                return ((1 << (end - start + 1)) - 1) << start
            if func == "BIT_MASK_LEN" and len(args) == 1:
                return (1 << args[0]) - 1
            raise ValueError(f"unsupported macro call {func}")
        raise ValueError(f"unsupported expression node: {ast.dump(node)}")


def sanitize_identifier(value: str) -> str:
    value = value.strip().upper()
    value = re.sub(r"[^A-Z0-9]+", "_", value)
    value = re.sub(r"_+", "_", value).strip("_")
    if not value:
        return "UNNAMED"
    if value[0].isdigit():
        value = f"N_{value}"
    return value


def parse_section_name(line: str) -> Optional[str]:
    match = SECTION_NAME_RE.search(line)
    if not match:
        return None
    raw = match.group(1).strip().strip("*").strip()
    return raw or None


def split_enum_items(enum_body: str) -> Iterable[str]:
    current: List[str] = []
    depth = 0
    for char in enum_body:
        if char in "({[":
            depth += 1
        elif char in ")}]":
            depth -= 1
        if char == "," and depth == 0:
            item = "".join(current).strip()
            if item:
                yield item
            current = []
        else:
            current.append(char)
    tail = "".join(current).strip()
    if tail:
        yield tail


def mask_to_offset_width(mask: int) -> Optional[tuple[int, int]]:
    if mask <= 0:
        return None
    bit_offset = (mask & -mask).bit_length() - 1
    shifted = mask >> bit_offset
    width = shifted.bit_length()
    if shifted != (1 << width) - 1:
        return None
    return bit_offset, width


def parse_enum_fields(enum_text: str, evaluator: ExpressionEvaluator) -> List[FieldDef]:
    body_match = re.search(r"\{(.*)\}", enum_text, re.DOTALL)
    if not body_match:
        return []
    enum_body = re.sub(r"//.*?$", "", body_match.group(1), flags=re.MULTILINE)
    enum_body = re.sub(r"/\*.*?\*/", "", enum_body, flags=re.DOTALL)
    fields: List[FieldDef] = []
    field_masks: Dict[str, int] = {}
    for item in split_enum_items(enum_body):
        item = item.strip()
        if not item or "=" not in item:
            continue
        name, expr = [part.strip() for part in item.split("=", 1)]
        if "(" in name:
            continue
        if not any(token in expr for token in ("BIT(", "BIT_RNG(")) and expr not in field_masks:
            if expr not in field_masks:
                continue
        try:
            if expr in field_masks:
                mask = field_masks[expr]
            else:
                mask = evaluator.eval_expr(expr)
        except Exception:
            continue
        offset_width = mask_to_offset_width(mask)
        if not offset_width:
            continue
        bit_offset, bit_width = offset_width
        field_masks[name] = mask
        fields.append(
            FieldDef(
                name=sanitize_identifier(name),
                bit_offset=bit_offset,
                bit_width=bit_width,
            )
        )
    return fields


def load_evaluator(paths: Iterable[Path]) -> ExpressionEvaluator:
    evaluator = ExpressionEvaluator()
    for path in paths:
        for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
            match = DEFINE_RE.match(line)
            if not match:
                continue
            name, expr = match.groups()
            if "(" in name:
                continue
            evaluator.define(name, expr)
    return evaluator


def parse_registers(register_h: Path, evaluator: ExpressionEvaluator) -> List[RegisterDef]:
    lines = register_h.read_text(encoding="utf-8", errors="ignore").splitlines()
    registers: List[RegisterDef] = []
    current_section = "misc"
    last_register: Optional[RegisterDef] = None
    i = 0
    while i < len(lines):
        line = lines[i]
        section_name = parse_section_name(line)
        if section_name:
            current_section = section_name
            i += 1
            continue

        match = REG_DEFINE_RE.match(line)
        if match:
            name, width_str, expr = match.groups()
            try:
                offset = evaluator.eval_expr(expr)
            except Exception:
                i += 1
                continue
            register = RegisterDef(
                name=sanitize_identifier(name.removeprefix("reg_")),
                address=evaluator.resolve("REG_BASE_ADDR") + offset,
                width=int(width_str),
                section=current_section,
            )
            registers.append(register)
            last_register = register
            i += 1
            continue

        if re.match(r"^\s*#define\s+reg_[A-Za-z0-9_]+\s*\(", line):
            last_register = None
            i += 1
            continue

        define_match = DEFINE_RE.match(line)
        if define_match:
            i += 1
            continue

        if ENUM_START_RE.match(line):
            enum_lines = [line]
            i += 1
            while i < len(lines):
                enum_lines.append(lines[i])
                if "};" in lines[i] or "}" in lines[i]:
                    break
                i += 1
            fields = parse_enum_fields("\n".join(enum_lines), evaluator)
            if fields:
                target = select_enum_target(registers, current_section, last_register, fields)
                if target is not None:
                    target.fields.extend(fields)
            i += 1
            continue

        i += 1
    return registers


def select_enum_target(
    registers: List[RegisterDef],
    current_section: str,
    last_register: Optional[RegisterDef],
    fields: List[FieldDef],
) -> Optional[RegisterDef]:
    max_bit = max(field.bit_offset + field.bit_width for field in fields)
    if last_register is not None and max_bit <= last_register.width:
        return last_register

    for register in reversed(registers):
        if register.section != current_section:
            continue
        if max_bit <= register.width:
            return register

    return last_register


def coalesce_aliases(registers: List[RegisterDef]) -> List[RegisterDef]:
    merged: Dict[tuple[str, int], RegisterDef] = {}
    ordered: List[RegisterDef] = []
    for reg in registers:
        key = (reg.section, reg.address)
        existing = merged.get(key)
        if existing is None:
            merged[key] = reg
            ordered.append(reg)
            continue
        existing.aliases.append(reg.name)
        if not existing.fields and reg.fields:
            existing.fields = reg.fields
        if reg.width > existing.width:
            existing.width = reg.width
    for reg in ordered:
        reg.aliases = sorted(set(reg.aliases))
    return ordered


def build_peripherals(registers: List[RegisterDef]) -> List[PeripheralDef]:
    grouped: Dict[str, List[RegisterDef]] = {}
    for reg in registers:
        grouped.setdefault(reg.section, []).append(reg)

    peripherals: List[PeripheralDef] = []
    used_names: Dict[str, int] = {}
    for section, regs in grouped.items():
        regs.sort(key=lambda item: (item.address, item.name))
        base_address = min(reg.address for reg in regs)
        base_name = sanitize_identifier(section)
        count = used_names.get(base_name, 0)
        used_names[base_name] = count + 1
        periph_name = base_name if count == 0 else f"{base_name}_{count + 1}"
        peripherals.append(
            PeripheralDef(
                name=periph_name,
                description=f"Auto-generated from section '{section}'",
                base_address=base_address,
                registers=regs,
            )
        )
    peripherals.sort(key=lambda item: item.base_address)
    return peripherals


def prettify_xml(root: ET.Element) -> str:
    raw = ET.tostring(root, encoding="utf-8")
    return minidom.parseString(raw).toprettyxml(indent="  ")


def build_svd(chip: str, peripherals: List[PeripheralDef]) -> str:
    root = ET.Element("device", {"schemaVersion": "1.3"})
    ET.SubElement(root, "name").text = f"TLSR{chip.upper()}"
    ET.SubElement(root, "version").text = "0.1"
    ET.SubElement(root, "description").text = f"Auto-generated SVD for Telink TLSR{chip.upper()}"
    ET.SubElement(root, "addressUnitBits").text = "8"
    ET.SubElement(root, "width").text = "32"

    peripherals_el = ET.SubElement(root, "peripherals")
    for peripheral in peripherals:
        peripheral_el = ET.SubElement(peripherals_el, "peripheral")
        ET.SubElement(peripheral_el, "name").text = peripheral.name
        ET.SubElement(peripheral_el, "description").text = peripheral.description
        ET.SubElement(peripheral_el, "baseAddress").text = hex(peripheral.base_address)

        registers_el = ET.SubElement(peripheral_el, "registers")
        for reg in peripheral.registers:
            register_el = ET.SubElement(registers_el, "register")
            ET.SubElement(register_el, "name").text = reg.name
            description = f"Auto-generated from {reg.name.lower()}"
            if reg.aliases:
                description += f"; aliases: {', '.join(reg.aliases)}"
            ET.SubElement(register_el, "description").text = description
            ET.SubElement(register_el, "addressOffset").text = hex(reg.address - peripheral.base_address)
            ET.SubElement(register_el, "size").text = str(reg.width)

            if reg.fields:
                fields_el = ET.SubElement(register_el, "fields")
                for field in sorted(reg.fields, key=lambda item: (item.bit_offset, item.name)):
                    field_el = ET.SubElement(fields_el, "field")
                    ET.SubElement(field_el, "name").text = field.name
                    if field.description:
                        ET.SubElement(field_el, "description").text = field.description
                    ET.SubElement(field_el, "bitOffset").text = str(field.bit_offset)
                    ET.SubElement(field_el, "bitWidth").text = str(field.bit_width)
    return prettify_xml(root)


def chip_paths(sdk_root: Path, chip: str) -> tuple[Path, Path]:
    chip_dir = sdk_root / "platform" / f"chip_{chip}"
    register_h = chip_dir / "register.h"
    bsp_h = chip_dir / "bsp.h"
    if not register_h.exists():
        raise FileNotFoundError(f"missing register header: {register_h}")
    if not bsp_h.exists():
        raise FileNotFoundError(f"missing bsp header: {bsp_h}")
    return register_h, bsp_h


def generate_svd(sdk_root: Path, chip: str) -> str:
    register_h, bsp_h = chip_paths(sdk_root, chip)
    evaluator = load_evaluator([bsp_h, sdk_root / "proj" / "common" / "bit.h", register_h])
    try:
        evaluator.resolve("REG_BASE_ADDR")
    except Exception as exc:
        raise RuntimeError(f"failed to resolve REG_BASE_ADDR for chip {chip}") from exc

    registers = parse_registers(register_h, evaluator)
    registers = coalesce_aliases(registers)
    if not registers:
        raise RuntimeError(f"no static registers found in {register_h}")
    peripherals = build_peripherals(registers)
    return build_svd(chip, peripherals)


def parse_args(argv: Optional[List[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate CMSIS-SVD from a Telink SDK tree")
    parser.add_argument("--sdk", required=True, type=Path, help="Path to the Telink SDK root")
    parser.add_argument(
        "--chip",
        required=True,
        choices=("8258", "8278", "826x"),
        help="Target chip family from platform/chip_*",
    )
    parser.add_argument("--output", required=True, type=Path, help="Output .svd file")
    return parser.parse_args(argv)


def main(argv: Optional[List[str]] = None) -> int:
    args = parse_args(argv)
    sdk_root = args.sdk.resolve()
    if not sdk_root.exists():
        print(f"error: SDK path does not exist: {sdk_root}", file=sys.stderr)
        return 2

    try:
        svd_text = generate_svd(sdk_root, args.chip)
    except Exception as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(svd_text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
