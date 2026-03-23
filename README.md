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

# 新增时序模块示例
make sim-pulse-counter

# 嵌套模块示例
make sim-toggle-pair

# DPI-C import 示例
make sim-dpi-adder

# 使用 Rust 原生 test 框架验证
make rust-test TOP_MODULES=top,adder_top,pulse_counter_top,toggle_pair_top,dpi_adder_top

# 只运行某个测试文件
make rust-test TOP_MODULES=dpi_adder_top RUST_TEST=dpi_adder_test

# 只运行某个测试函数
make rust-test TOP_MODULES=toggle_pair_top RUST_TEST=toggle_pair_test RUST_TEST_FILTER=nested_modules_toggle_independently
```

## Project Structure

```
.
├── flake.nix
├── Makefile
├── rtl/
│   ├── top.sv                          # counter RTL (top)
│   ├── adder_top.sv                    # adder RTL (adder_top)
│   ├── pulse_counter_top.sv            # pulse 触发计数器
│   ├── toggle_cell.sv                  # 嵌套测试用子模块
│   └── toggle_pair_top.sv              # 顶层实例化两个 toggle_cell
│   └── dpi_adder_top.sv                # 通过 DPI-C import 调 Rust 函数
├── scripts/
│   └── gen_verilator_binder.py         # 解析 V<top>.h 并生成绑定
└── rust/
    ├── Cargo.toml
    ├── build.rs                        # 执行 verilator + 生成绑定 + 链接
    ├── src/lib.rs                      # include!(OUT_DIR/bindings_manifest.rs)
    ├── src/dpi.rs                      # Rust 导出的 DPI-C 函数
    └── examples/
        ├── counter.rs                  # 驱动 top.sv
        ├── adder.rs                    # 驱动 adder_top.sv
        ├── pulse_counter.rs            # 驱动 pulse_counter_top.sv
        ├── toggle_pair.rs              # 驱动 toggle_pair_top.sv
        └── dpi_adder.rs                # 驱动 dpi_adder_top.sv
    └── tests/
        ├── pulse_counter_test.rs       # Rust 原生集成测试
        ├── toggle_pair_test.rs         # 嵌套模块集成测试
        └── dpi_adder_test.rs           # DPI import 集成测试
```

## Binder Workflow

执行 `cargo run` / `make rust-sim` / `cargo test` 会自动执行：

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
make sim-pulse-counter              # TOP=pulse_counter_top + example=pulse_counter
make sim-toggle-pair                # TOP=toggle_pair_top + example=toggle_pair
make sim-dpi-adder                  # TOP=dpi_adder_top + example=dpi_adder
make rust-sim TOP_MODULE=top RUST_EXAMPLE=counter
make rust-check TOP_MODULE=adder_top
make rust-test TOP_MODULES=top,adder_top,pulse_counter_top,toggle_pair_top,dpi_adder_top
make rust-test TOP_MODULES=dpi_adder_top RUST_TEST=dpi_adder_test
make rust-test TOP_MODULES=toggle_pair_top RUST_TEST=toggle_pair_test RUST_TEST_FILTER=nested_modules_toggle_independently
```

## Rust 原生测试

现在仓库已经支持把多个 top 一次性生成绑定，然后直接走 Rust 自带测试框架：

- 测试文件位置：`rust/tests/*.rs`
- 运行方式：`cargo test` 或 `make rust-test`
- 适合做断言式验证，而不是只打印波形/日志

常用方式：

- 跑全部测试：`make rust-test`
- 跑单个测试文件：`make rust-test TOP_MODULES=dpi_adder_top RUST_TEST=dpi_adder_test`
- 跑单个测试函数：`make rust-test TOP_MODULES=toggle_pair_top RUST_TEST=toggle_pair_test RUST_TEST_FILTER=nested_modules_toggle_independently`

例如当前新增了：

- `rtl/pulse_counter_top.sv`
- `rust/tests/pulse_counter_test.rs`
- `rtl/toggle_cell.sv`
- `rtl/toggle_pair_top.sv`
- `rust/tests/toggle_pair_test.rs`
- `rtl/dpi_adder_top.sv`
- `rust/src/dpi.rs`
- `rust/tests/dpi_adder_test.rs`

## DPI-C 支持现状

当前仓库已经支持 **`import "DPI-C"`** 这一条主路径。

支持方式：

- SystemVerilog 中声明 `import "DPI-C" function ...`
- Rust 中使用 `extern "C"` 导出同名函数
- Verilator 在仿真时直接调用 Rust 导出的符号

当前示例：

- `rtl/dpi_adder_top.sv`
- `rust/src/dpi.rs`
- `rust/tests/dpi_adder_test.rs`

当前尚未专门封装：

- `export "DPI-C"`
- DPI scope/context 高级封装

这也是本仓库使用 Rust 封装 Verilated 模型的核心价值之一：

- 直接复用 Rust 的 `#[test]`
- 使用 `assert_eq!` 等断言
- 易于集成到 CI

## Docs

- [项目现状与扩展指南](docs/%E9%A1%B9%E7%9B%AE%E7%8E%B0%E7%8A%B6%E4%B8%8E%E6%89%A9%E5%B1%95%E6%8C%87%E5%8D%97.md)

## Limitations

当前自动生成器支持：

- `<= 64-bit` 端口
- `input/output/inout` 标量端口

若遇到 `>64-bit`（`VL_*W`）端口，会在生成阶段报错。
