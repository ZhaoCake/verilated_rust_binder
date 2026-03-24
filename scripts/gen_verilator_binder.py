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


def symbol_prefix(top: str) -> str:
    sanitized = re.sub(r"[^A-Za-z0-9_]", "_", top)
    return f"vrb_{sanitized}"


def render_cpp(top: str, ports: list[Port]) -> str:
    prefix = symbol_prefix(top)
    lines = [
        "#include <cstdint>",
        "#include <string>",
        f"#include \"V{top}.h\"",
        "#include \"verilated.h\"",
        "#include \"verilated_vcd_c.h\"",
        "",
        'extern "C" {',
        f"using Model = V{top};",
        "",
        "struct ModelHandle {",
        "    Model* model;",
        "    VerilatedVcdC* trace;",
        "};",
        "",
        f"ModelHandle* {prefix}_new() {{",
        "    return new ModelHandle{new Model, nullptr};",
        "}",
        "",
        f"void {prefix}_delete(ModelHandle* handle) {{",
        "    if (handle == nullptr) {",
        "        return;",
        "    }",
        "",
        "    if (handle->trace != nullptr) {",
        "        handle->trace->close();",
        "        delete handle->trace;",
        "        handle->trace = nullptr;",
        "    }",
        "",
        "    delete handle->model;",
        "    delete handle;",
        "}",
        "",
        f"void {prefix}_eval(ModelHandle* handle) {{",
        "    handle->model->eval();",
        "}",
        "",
        f"void {prefix}_final(ModelHandle* handle) {{",
        "    handle->model->final();",
        "}",
        "",
        f"void {prefix}_trace_open(ModelHandle* handle, const char* path, int levels) {{",
        "    if (handle->trace != nullptr) {",
        "        handle->trace->close();",
        "        delete handle->trace;",
        "        handle->trace = nullptr;",
        "    }",
        "",
        "    Verilated::traceEverOn(true);",
        "    handle->trace = new VerilatedVcdC;",
        "    handle->model->trace(handle->trace, levels);",
        "    handle->trace->open(path);",
        "}",
        "",
        f"void {prefix}_trace_dump(ModelHandle* handle, uint64_t time) {{",
        "    if (handle->trace != nullptr) {",
        "        handle->trace->dump(static_cast<vluint64_t>(time));",
        "    }",
        "}",
        "",
        f"void {prefix}_trace_flush(ModelHandle* handle) {{",
        "    if (handle->trace != nullptr) {",
        "        handle->trace->flush();",
        "    }",
        "}",
        "",
        f"void {prefix}_trace_close(ModelHandle* handle) {{",
        "    if (handle->trace != nullptr) {",
        "        handle->trace->close();",
        "        delete handle->trace;",
        "        handle->trace = nullptr;",
        "    }",
        "}",
        "",
    ]

    for port in ports:
        if port.direction in {"in", "inout"}:
            lines.extend(
                [
                    f"void {prefix}_set_{port.name}(ModelHandle* handle, uint64_t value) {{",
                    f"    handle->model->{port.name} = value;",
                    "}",
                    "",
                ]
            )

        lines.extend(
            [
                f"uint64_t {prefix}_get_{port.name}(const ModelHandle* handle) {{",
                f"    return static_cast<uint64_t>(handle->model->{port.name});",
                "}",
                "",
            ]
        )

    lines.append("}")
    lines.append("")
    return "\n".join(lines)


def render_rust(top: str, ports: list[Port]) -> str:
    prefix = symbol_prefix(top)
    lines = [
        "use std::ffi::c_void;",
        "use std::ffi::CString;",
        "use std::path::Path;",
        "",
        "#[allow(non_camel_case_types)]",
        "type vrb_model_t = c_void;",
        "",
        'unsafe extern "C" {',
        f"    fn {prefix}_new() -> *mut vrb_model_t;",
        f"    fn {prefix}_delete(model: *mut vrb_model_t);",
        f"    fn {prefix}_eval(model: *mut vrb_model_t);",
        f"    fn {prefix}_final(model: *mut vrb_model_t);",
        f"    fn {prefix}_trace_open(model: *mut vrb_model_t, path: *const i8, levels: i32);",
        f"    fn {prefix}_trace_dump(model: *mut vrb_model_t, time: u64);",
        f"    fn {prefix}_trace_flush(model: *mut vrb_model_t);",
        f"    fn {prefix}_trace_close(model: *mut vrb_model_t);",
    ]

    for port in ports:
        if port.direction in {"in", "inout"}:
            lines.append(f"    fn {prefix}_set_{port.name}(model: *mut vrb_model_t, value: u64);")
        lines.append(f"    fn {prefix}_get_{port.name}(model: *const vrb_model_t) -> u64;")

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
            f"        let raw = unsafe {{ {prefix}_new() }};",
            "        assert!(!raw.is_null(), \"verilator model allocation failed\");",
            "        Self { raw }",
            "    }",
            "",
            "    pub fn eval(&mut self) {",
            f"        unsafe {{ {prefix}_eval(self.raw) }};",
            "    }",
            "",
            "    pub fn enable_vcd<P: AsRef<Path>>(&mut self, path: P, levels: i32) {",
            "        let path = path.as_ref().to_string_lossy();",
            "        let path = CString::new(path.as_bytes()).expect(\"vcd path contains interior NUL byte\");",
            f"        unsafe {{ {prefix}_trace_open(self.raw, path.as_ptr(), levels) }};",
            "    }",
            "",
            "    pub fn dump_vcd(&mut self, time: u64) {",
            f"        unsafe {{ {prefix}_trace_dump(self.raw, time) }};",
            "    }",
            "",
            "    pub fn flush_vcd(&mut self) {",
            f"        unsafe {{ {prefix}_trace_flush(self.raw) }};",
            "    }",
            "",
            "    pub fn close_vcd(&mut self) {",
            f"        unsafe {{ {prefix}_trace_close(self.raw) }};",
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
                    f"        unsafe {{ {prefix}_set_{port.name}(self.raw, value) }};",
                    "    }",
                    "",
                ]
            )

        lines.extend(
            [
                f"    pub fn {port.name}(&self) -> u64 {{",
                f"        unsafe {{ {prefix}_get_{port.name}(self.raw) }}",
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
            f"            {prefix}_final(self.raw);",
            f"            {prefix}_delete(self.raw);",
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
    args.out_rs.write_text(render_rust(args.top, ports), encoding="utf-8")

    print(
        f"[gen_verilator_binder] generated {len(ports)} ports: "
        + ", ".join(port.name for port in ports)
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
