# SystemVerilog + Verilator + Rust Binder Template

now we have chisel development environment with verilated rust binder!

[chisel_with_vrb](https://github.com/ZhaoCake/chisel_with_vrb)

---

This is a pure Rust-driven Verilator simulation template:

`SV RTL -> Verilator generates C++ model -> Auto-generated C ABI + Rust wrapper -> Rust-driven simulation`

The repository has removed the C++ testbench flow and keeps only the Rust simulation path.

## Quick Start

```bash
# Activate development environment
direnv allow   # or nix develop

# Default example: counter
make sim

# Second RTL example: adder
make sim-adder

# Timing module example
make sim-pulse-counter

# Nested module example
make sim-toggle-pair

# Export waveform for toggle_pair_top
make sim-toggle-pair

# DPI-C import example
make sim-dpi-adder

# Run tests with Rust native test framework
make rust-test TOP_MODULES=top,adder_top,pulse_counter_top,toggle_pair_top,dpi_adder_top

# Run one test file only
make rust-test TOP_MODULES=dpi_adder_top RUST_TEST=dpi_adder_test

# Run one test function only
make rust-test TOP_MODULES=toggle_pair_top RUST_TEST=toggle_pair_test RUST_TEST_FILTER=nested_modules_toggle_independently

# Run waveform export test only
make rust-test TOP_MODULES=toggle_pair_top RUST_TEST=toggle_pair_wave_test
```

## Project Structure

```
.
├── flake.nix
├── Makefile
├── README.md                           # English README
├── README_zh.md                        # Chinese README
├── rtl/
│   ├── top.sv                          # counter RTL (top)
│   ├── adder_top.sv                    # adder RTL (adder_top)
│   ├── pulse_counter_top.sv            # pulse-triggered counter
│   ├── toggle_cell.sv                  # nested-test submodule
│   ├── toggle_pair_top.sv              # top level instantiating two toggle_cells
│   └── dpi_adder_top.sv                # calls Rust function via DPI-C import
├── scripts/
│   └── gen_verilator_binder.py         # parses V<top>.h and generates bindings
└── rust/
    ├── Cargo.toml
    ├── build.rs                        # runs verilator + binding generation + linking
    ├── src/lib.rs                      # include!(OUT_DIR/bindings_manifest.rs)
    ├── src/dpi.rs                      # DPI-C functions exported from Rust
    └── examples/
        ├── counter.rs                  # drives top.sv
        ├── adder.rs                    # drives adder_top.sv
        ├── pulse_counter.rs            # drives pulse_counter_top.sv
        ├── toggle_pair.rs              # drives toggle_pair_top.sv
        └── dpi_adder.rs                # drives dpi_adder_top.sv
    └── tests/
        ├── pulse_counter_test.rs       # Rust native integration test
        ├── toggle_pair_test.rs         # nested-module integration test
        ├── toggle_pair_wave_test.rs    # waveform export integration test
        └── dpi_adder_test.rs           # DPI import integration test
```

## Binder Workflow

Running `cargo run`, `make rust-sim`, or `cargo test` automatically performs:

1. `build.rs` calls `verilator --cc --top-module <TOP>` to generate `V<TOP>.h/.cpp`
2. `scripts/gen_verilator_binder.py` parses ports and generates:
   - C++ bridge (`extern "C"`: `new/eval/final/get/set`)
   - Rust `SimModel` API
3. `build.rs` calls `make -f V<TOP>.mk V<TOP>__ALL.a`
4. Rust links the Verilator model and bridge, then runs the example

## Commands

```bash
make sim                            # equivalent to make sim-counter
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
make rust-test TOP_MODULES=toggle_pair_top RUST_TEST=toggle_pair_wave_test
```

## Rust Native Testing

The repository supports generating bindings for multiple tops in one run and using Rust's native test framework directly:

- Test location: `rust/tests/*.rs`
- Run with: `cargo test` or `make rust-test`
- Best for assertion-based verification instead of log-only checks

Common usage:

- Run all tests: `make rust-test`
- Run one test file: `make rust-test TOP_MODULES=dpi_adder_top RUST_TEST=dpi_adder_test`
- Run one test function: `make rust-test TOP_MODULES=toggle_pair_top RUST_TEST=toggle_pair_test RUST_TEST_FILTER=nested_modules_toggle_independently`
- Run waveform export test: `make rust-test TOP_MODULES=toggle_pair_top RUST_TEST=toggle_pair_wave_test`

Recent additions:

- `rtl/pulse_counter_top.sv`
- `rust/tests/pulse_counter_test.rs`
- `rtl/toggle_cell.sv`
- `rtl/toggle_pair_top.sv`
- `rust/tests/toggle_pair_test.rs`
- `rust/tests/toggle_pair_wave_test.rs`
- `rtl/dpi_adder_top.sv`
- `rust/src/dpi.rs`
- `rust/tests/dpi_adder_test.rs`

## Waveform Export Support

The repository now supports VCD waveform export through the generated Rust wrapper.

Available API on each generated `SimModel`:

- `enable_vcd(path, levels)`
- `dump_vcd(time)`
- `flush_vcd()`
- `close_vcd()`

Current verified example:

- `rust/examples/toggle_pair.rs`
- output file: `wave_toggle_pair.vcd`

Current verified test:

- `rust/tests/toggle_pair_wave_test.rs`

Typical usage pattern:

1. call `enable_vcd(...)`
2. after each `eval()`, call `dump_vcd(sim_time)`
3. call `flush_vcd()` / `close_vcd()` before exit

Note:

- waveform export is currently validated on `toggle_pair_top`
- the dedicated waveform test is kept in a separate test file to avoid native trace teardown conflicts inside the same test binary

## DPI-C Support Status

The repository currently supports the primary **`import "DPI-C"`** path.

How it works:

- Declare `import "DPI-C" function ...` in SystemVerilog
- Export same-name function with `extern "C"` in Rust
- Verilator calls Rust-exported symbols during simulation

Current example:

- `rtl/dpi_adder_top.sv`
- `rust/src/dpi.rs`
- `rust/tests/dpi_adder_test.rs`

Not yet specifically wrapped:

- `export "DPI-C"`
- Advanced DPI scope/context encapsulation

This is one of the key values of wrapping Verilated models with Rust:

- Native reuse of Rust `#[test]`
- Assertions like `assert_eq!`
- Easy CI integration

## Docs

- [Current Status and Extension Guide](docs/Current_Status_and_Extension_Guide.md)
- [项目现状与扩展指南 (Chinese)](docs/%E9%A1%B9%E7%9B%AE%E7%8E%B0%E7%8A%B6%E4%B8%8E%E6%89%A9%E5%B1%95%E6%8C%87%E5%8D%97.md)

## Limitations

Current automatic generator supports:

- `<= 64-bit` ports
- `input/output/inout` scalar ports

Ports with width `>64-bit` (`VL_*W`) fail during generation.
