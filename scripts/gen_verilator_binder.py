#!/usr/bin/env python3

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass
class Port:
    name: str
    direction: str
    width: int


PORT_RE = re.compile(
    r"VL_(INOUT|IN|OUT)(W|8|16|64)?\s*\(&([A-Za-z_][A-Za-z0-9_]*)\s*,\s*(\d+)\s*,\s*(\d+)(?:\s*,\s*(\d+))?\s*\);"
)


def parse_ports(header_path: Path) -> list[Port]:
    ports: list[Port] = []
    for line in header_path.read_text(encoding="utf-8").splitlines():
        match = PORT_RE.search(line)
        if not match:
            continue

        direction, type_suffix, name, msb_raw, lsb_raw, _words = match.groups()
        msb = int(msb_raw)
        lsb = int(lsb_raw)
        width = msb - lsb + 1

        if type_suffix == "W" or width > 64:
            raise ValueError(
                f"Port '{name}' width {width} is unsupported (only <=64 bits supported currently)."
            )

        ports.append(Port(name=name, direction=direction.lower(), width=width))

    if not ports:
        raise ValueError(
            f"No ports parsed from {header_path}. Ensure top module has explicit IO and Verilator completed."
        )

    return ports


def rust_mask(width: int) -> str:
    if width >= 64:
        return "u64::MAX"
    return hex((1 << width) - 1)


def render_cpp(top: str, ports: list[Port]) -> str:
    lines = [
        "#include <cstdint>",
        f"#include \"V{top}.h\"",
        "#include \"verilated.h\"",
        "",
        'extern "C" {',
        f"using Model = V{top};",
        "",
        "Model* vrb_new() {",
        "    return new Model;",
        "}",
        "",
        "void vrb_delete(Model* model) {",
        "    delete model;",
        "}",
        "",
        "void vrb_eval(Model* model) {",
        "    model->eval();",
        "}",
        "",
        "void vrb_final(Model* model) {",
        "    model->final();",
        "}",
        "",
    ]

    for port in ports:
        if port.direction in {"in", "inout"}:
            lines.extend(
                [
                    f"void vrb_set_{port.name}(Model* model, uint64_t value) {{",
                    f"    model->{port.name} = value;",
                    "}",
                    "",
                ]
            )

        lines.extend(
            [
                f"uint64_t vrb_get_{port.name}(const Model* model) {{",
                f"    return static_cast<uint64_t>(model->{port.name});",
                "}",
                "",
            ]
        )

    lines.append("}")
    lines.append("")
    return "\n".join(lines)


def render_rust(ports: list[Port]) -> str:
    lines = [
        "use std::ffi::c_void;",
        "",
        "#[allow(non_camel_case_types)]",
        "type vrb_model_t = c_void;",
        "",
        'unsafe extern "C" {',
        "    fn vrb_new() -> *mut vrb_model_t;",
        "    fn vrb_delete(model: *mut vrb_model_t);",
        "    fn vrb_eval(model: *mut vrb_model_t);",
        "    fn vrb_final(model: *mut vrb_model_t);",
    ]

    for port in ports:
        if port.direction in {"in", "inout"}:
            lines.append(f"    fn vrb_set_{port.name}(model: *mut vrb_model_t, value: u64);")
        lines.append(f"    fn vrb_get_{port.name}(model: *const vrb_model_t) -> u64;")

    lines.extend(
        [
            "}",
            "",
            "pub struct SimModel {",
            "    raw: *mut vrb_model_t,",
            "}",
            "",
            "impl SimModel {",
            "    pub fn new() -> Self {",
            "        let raw = unsafe { vrb_new() };",
            "        assert!(!raw.is_null(), \"verilator model allocation failed\");",
            "        Self { raw }",
            "    }",
            "",
            "    pub fn eval(&mut self) {",
            "        unsafe { vrb_eval(self.raw) };",
            "    }",
            "",
            "    pub fn port_width(name: &str) -> Option<u32> {",
            "        match name {",
        ]
    )

    for port in ports:
        lines.append(f'            "{port.name}" => Some({port.width}),')

    lines.extend(
        [
            "            _ => None,",
            "        }",
            "    }",
            "",
        ]
    )

    for port in ports:
        if port.direction in {"in", "inout"}:
            mask = rust_mask(port.width)
            lines.extend(
                [
                    f"    pub fn set_{port.name}(&mut self, value: u64) {{",
                    f"        let value = value & {mask};",
                    f"        unsafe {{ vrb_set_{port.name}(self.raw, value) }};",
                    "    }",
                    "",
                ]
            )

        lines.extend(
            [
                f"    pub fn {port.name}(&self) -> u64 {{",
                f"        unsafe {{ vrb_get_{port.name}(self.raw) }}",
                "    }",
                "",
            ]
        )

    lines.extend(
        [
            "}",
            "",
            "impl Drop for SimModel {",
            "    fn drop(&mut self) {",
            "        if self.raw.is_null() {",
            "            return;",
            "        }",
            "",
            "        unsafe {",
            "            vrb_final(self.raw);",
            "            vrb_delete(self.raw);",
            "        }",
            "        self.raw = std::ptr::null_mut();",
            "    }",
            "}",
            "",
        ]
    )

    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate Rust/C++ FFI binder from Verilator top header."
    )
    parser.add_argument("--header", type=Path, required=True)
    parser.add_argument("--top", required=True)
    parser.add_argument("--out-cpp", type=Path, required=True)
    parser.add_argument("--out-rs", type=Path, required=True)
    args = parser.parse_args()

    try:
        ports = parse_ports(args.header)
    except ValueError as err:
        print(f"[gen_verilator_binder] {err}", file=sys.stderr)
        return 1

    args.out_cpp.parent.mkdir(parents=True, exist_ok=True)
    args.out_rs.parent.mkdir(parents=True, exist_ok=True)
    args.out_cpp.write_text(render_cpp(args.top, ports), encoding="utf-8")
    args.out_rs.write_text(render_rust(ports), encoding="utf-8")

    print(
        f"[gen_verilator_binder] generated {len(ports)} ports: "
        + ", ".join(port.name for port in ports)
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
