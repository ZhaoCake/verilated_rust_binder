# SystemVerilog + Verilator + Rust Binder Template

这是一个纯 Rust 驱动的 Verilator 仿真模板：

`SV RTL -> Verilator 生成 C++ 模型 -> 自动生成 C ABI + Rust 封装 -> Rust 调用仿真`

仓库已移除 C++ testbench 流程，只保留 Rust 这一条仿真路径。

## Quick Start

```bash
# 激活开发环境
direnv allow   # 或 nix develop

# 默认示例：counter
make sim

# 第二个 RTL 示例：adder
make sim-adder
```

## Project Structure

```
.
├── flake.nix
├── Makefile
├── rtl/
│   ├── top.sv                          # counter RTL (top)
│   └── adder_top.sv                    # adder RTL (adder_top)
├── scripts/
│   └── gen_verilator_binder.py         # 解析 V<top>.h 并生成绑定
└── rust/
    ├── Cargo.toml
    ├── build.rs                        # 执行 verilator + 生成绑定 + 链接
    ├── src/lib.rs                      # include!(OUT_DIR/binder.rs)
    └── examples/
        ├── counter.rs                  # 驱动 top.sv
        └── adder.rs                    # 驱动 adder_top.sv
```

## Binder Workflow

执行 `cargo run` / `make rust-sim` 会自动执行：

1. `build.rs` 调用 `verilator --cc --top-module <TOP>` 生成 `V<TOP>.h/.cpp`
2. `scripts/gen_verilator_binder.py` 解析端口并生成：
   - C++ bridge（`extern "C"`：`new/eval/final/get/set`）
   - Rust `SimModel` API
3. `build.rs` 调用 `make -f V<TOP>.mk V<TOP>__ALL.a`
4. Rust 链接 Verilator 模型与 bridge，运行 Rust 示例

## Commands

```bash
make sim                            # 等价 make sim-counter
make sim-counter                    # TOP=top + example=counter
make sim-adder                      # TOP=adder_top + example=adder
make rust-sim TOP_MODULE=top RUST_EXAMPLE=counter
make rust-check TOP_MODULE=adder_top
```

## Limitations

当前自动生成器支持：

- `<= 64-bit` 端口
- `input/output/inout` 标量端口

若遇到 `>64-bit`（`VL_*W`）端口，会在生成阶段报错。
