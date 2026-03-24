# Project Status and Extension Guide

This document explains two things:

1. What this repository has already completed.
2. How to add new SystemVerilog files and Rust-side simulation code in the current workflow.

## 1. What Has Been Completed

This repository has established an end-to-end automated flow:

`SystemVerilog RTL -> Verilator C++ model -> Auto-generated C ABI + Rust API -> Rust examples drive simulation`

Current capabilities:

- Rust can directly drive simulation without any C++ testbench.
- `build.rs` automatically scans all `.sv/.v` files under `rtl/` and includes them in Verilator compilation.
- The top module is selected via `VERILATOR_TOP` (default: `top`).
- `scripts/gen_verilator_binder.py` is called automatically to:
  - Parse port definitions from `V<top>.h`
  - Generate a C++ bridge (`vrb_new/eval/final/get/set`)
  - Generate a Rust `SimModel` wrapper (`set_xxx()/xxx()`)
- The Verilator-generated Makefile is executed automatically to build `V<top>__ALL.a` and link it to Rust.
- Available runnable examples include:
  - `top.sv` + `rust/examples/counter.rs`
  - `adder_top.sv` + `rust/examples/adder.rs`
  - `pulse_counter_top.sv` + `rust/examples/pulse_counter.rs`
  - `toggle_pair_top.sv` + `rust/examples/toggle_pair.rs`
  - `dpi_adder_top.sv` + `rust/examples/dpi_adder.rs`
- Rust native tests are supported:
  - Use `#[test]` directly in `rust/tests/*.rs`
  - Generate bindings for multiple tops in one build for integration tests
- Root-level `Makefile` provides command wrappers:
  - `make sim` / `make sim-counter`
  - `make sim-adder`
  - `make sim-pulse-counter`
  - `make sim-toggle-pair`
  - `make sim-dpi-adder`
  - `make rust-sim TOP_MODULE=... RUST_EXAMPLE=...`
  - `make rust-test TOP_MODULES=...`

## 2. Current Constraints and Notes

The automatic binder generator currently supports only:

- Scalar ports (`input/output/inout`)
- Bit width `<= 64`

Not supported:

- `>64-bit` ports (e.g. `VL_*W` in Verilator headers)

Using extra-wide ports will fail and stop during generation.

## 3. How to Add a New SystemVerilog Module

Recommended workflow:

### Step 1: Add RTL File

Put your new module file under `rtl/`, for example:

- `rtl/my_accum_top.sv`

Note: `build.rs` automatically collects `rtl/*.sv` and `rtl/*.v`, so you usually do not need to maintain a source list manually.

### Step 2: Confirm the Top Module Name

Make sure your top module name is explicit, for example:

- Module name: `my_accum_top`

At runtime, pass it through `TOP_MODULE` (internally mapped to `VERILATOR_TOP`).

### Step 3: Write a Rust Driver Example

Add a file under `rust/examples/`, for example:

- `rust/examples/my_accum.rs`

Minimal template:

```rust
use verilated_rust_binder::my_accum_top::SimModel;

fn main() {
    let mut dut = SimModel::new();

    // These methods are auto-generated from ports.
    // For example, ports clk/rst_n/en/out may map to:
    // dut.set_clk(...), dut.set_rst_n(...), dut.set_en(...), dut.out()

    dut.eval();
    println!("custom simulation done");
}
```

### Step 4: Run

At repository root:

```bash
make rust-sim TOP_MODULE=my_accum_top RUST_EXAMPLE=my_accum
```

To check only the build chain first:

```bash
make rust-check TOP_MODULE=my_accum_top
```

### Step 5 (Optional): Add a Shortcut Target in Makefile

If you want a shortcut like `make sim-adder`, add this to root `Makefile`:

```makefile
sim-my-accum: TOP_MODULE = my_accum_top
sim-my-accum: RUST_EXAMPLE = my_accum
sim-my-accum: rust-sim
```

Then run:

```bash
make sim-my-accum
```

## 4. How to Add a Rust Project (Two Ways)

"Add a Rust project" usually means one of the following:

- A. Add a new simulation entry in the current crate (recommended)
- B. Create a new standalone crate (advanced)

### A. Add a New Simulation Entry in the Current Crate (Recommended)

This is the lightest approach, essentially adding `rust/examples/*.rs`:

1. Create an example file, e.g. `rust/examples/foo.rs`
2. Use the explicit module path for the target top, e.g. `use verilated_rust_binder::<top_name>::SimModel;`
3. Run:

```bash
make rust-sim TOP_MODULE=<your_top_module> RUST_EXAMPLE=foo
```

Advantages:

- No need to modify `Cargo.toml` (Cargo auto-discovers examples)
- Reuses existing `build.rs` auto-binding flow

## 4.1 How to Test DUT with Rust Native Test Framework

This is also supported now.

### Recommended Directory

Put tests in:

- `rust/tests/*.rs`

For example:

- `rust/tests/pulse_counter_test.rs`

### Usage

If tests need multiple tops, generate them in one build:

```bash
VERILATOR_TOPS=top,adder_top,pulse_counter_top,toggle_pair_top,dpi_adder_top cargo test --manifest-path rust/Cargo.toml
```

Or use Makefile:

```bash
make rust-test
```

Run one test file only:

```bash
make rust-test TOP_MODULES=dpi_adder_top RUST_TEST=dpi_adder_test
```

Run one test function only:

```bash
make rust-test TOP_MODULES=toggle_pair_top RUST_TEST=toggle_pair_test RUST_TEST_FILTER=nested_modules_toggle_independently
```

### Test Code Style

Import the module for the target top directly in test code:

```rust
use verilated_rust_binder::pulse_counter_top::SimModel;

#[test]
fn example_test() {
  let mut dut = SimModel::new();
  dut.eval();
  assert_eq!(dut.count(), 0);
}
```

You get Rust-native testing benefits directly:

- `#[test]`
- `assert_eq!`
- `cargo test`
- Easy CI integration

### B. Create a New Standalone Rust Crate (Advanced)

If you really need isolation (e.g. multi-team or separate executables), use this approach:

1. Create a new crate directory in repo (e.g. `rust_foo/`)
2. Copy and adapt `rust/build.rs`, keeping these key steps:
   - Scan `rtl/`
   - Call `verilator`
   - Call `scripts/gen_verilator_binder.py`
   - Build and link `V<top>__ALL.a`
3. In the new crate `src/lib.rs`, keep:

```rust
include!(concat!(env!("OUT_DIR"), "/binder.rs"));
```

4. Add to the new crate `Cargo.toml`:

```toml
[build-dependencies]
cc = "1.1"
```

5. Select top by env var when running:

```bash
VERILATOR_TOP=<your_top_module> cargo run --manifest-path rust_foo/Cargo.toml --example <example_name>
```

Recommendation: unless you have explicit isolation needs, prefer approach A (new examples in current crate).

## 5. Common Troubleshooting

- Error: `No rtl source found`
  - Check whether `.sv/.v` files are under `rtl/`.
- Error: missing `set_xxx()` or `xxx()` method
  - First confirm top module is correct (`TOP_MODULE` matches module name).
  - Then confirm the port actually exists in that top module.
- Error: unsupported port width
  - Check whether any port is `>64-bit`; current generator does not support this.

## 6. Reusable One-Line Command Template

After adding any new module, the most common command pattern is:

```bash
make rust-sim TOP_MODULE=<sv_top_module_name> RUST_EXAMPLE=<rust_example_filename_without_suffix>
```

For example:

```bash
make rust-sim TOP_MODULE=adder_top RUST_EXAMPLE=adder
```

## 7. Are Nested Modules Supported?

Yes.

The repository already includes a dedicated nested-module example:

- Child module: `rtl/toggle_cell.sv`
- Top module: `rtl/toggle_pair_top.sv`

`toggle_pair_top` instantiates two `toggle_cell` instances internally:

- `u_toggle_a`
- `u_toggle_b`

Rust observes and drives through top-level ports only; no need to care about internal instance names:

```rust
use verilated_rust_binder::toggle_pair_top::SimModel;
```

Related test file:

- `rust/tests/toggle_pair_test.rs`

Run with:

```bash
make rust-test TOP_MODULES=top,adder_top,pulse_counter_top,toggle_pair_top,dpi_adder_top
```

## 8. How to Use `import "DPI-C"`

The repository now supports `import "DPI-C"`, which is the most common DPI path.

### SystemVerilog Side

Example:

```systemverilog
import "DPI-C" function int rust_add(input int a, input int b);
```

Then call it directly in module logic:

```systemverilog
sum = rust_add(int'(a), int'(b));
```

Current top-level example:

- `rtl/dpi_adder_top.sv`

### Rust Side

Export the same C ABI symbol:

```rust
#[unsafe(no_mangle)]
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
  a.wrapping_add(b)
}
```

Current example location:

- `rust/src/dpi.rs`

### Test Method

```bash
VERILATOR_TOPS=top,adder_top,pulse_counter_top,toggle_pair_top,dpi_adder_top cargo test --manifest-path rust/Cargo.toml --test dpi_adder_test
```

Or:

```bash
make rust-test TOP_MODULES=dpi_adder_top RUST_TEST=dpi_adder_test
```

### Current Boundaries

Current implementation supports:

- `import "DPI-C" function ...`

Advanced features not yet specially wrapped:

- `export "DPI-C"`
- DPI scope/context API
- Auto-parsing DPI prototypes and generating Rust declarations
