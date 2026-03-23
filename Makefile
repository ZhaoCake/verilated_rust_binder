# SystemVerilog + Verilator + Rust Binder Makefile
.PHONY: all sim sim-counter sim-adder sim-pulse-counter sim-toggle-pair sim-dpi-adder rust-sim rust-check rust-test lint format clean help

TOP_MODULE ?= top
RUST_EXAMPLE ?= counter
TOP_MODULES ?= top,adder_top,pulse_counter_top,toggle_pair_top,dpi_adder_top
RUST_TEST ?=
RUST_TEST_FILTER ?=
SRC_DIR = rtl
RUST_DIR = rust

VSRCS = $(wildcard $(SRC_DIR)/*.sv $(SRC_DIR)/*.v)

all: sim

sim: sim-counter

sim-counter: TOP_MODULE = top
sim-counter: RUST_EXAMPLE = counter
sim-counter: rust-sim

sim-adder: TOP_MODULE = adder_top
sim-adder: RUST_EXAMPLE = adder
sim-adder: rust-sim

sim-pulse-counter: TOP_MODULE = pulse_counter_top
sim-pulse-counter: RUST_EXAMPLE = pulse_counter
sim-pulse-counter: rust-sim

sim-toggle-pair: TOP_MODULE = toggle_pair_top
sim-toggle-pair: RUST_EXAMPLE = toggle_pair
sim-toggle-pair: rust-sim

sim-dpi-adder: TOP_MODULE = dpi_adder_top
sim-dpi-adder: RUST_EXAMPLE = dpi_adder
sim-dpi-adder: rust-sim

rust-sim: $(VSRCS)
	@echo "🦀 Running Rust simulation (TOP=$(TOP_MODULE), EXAMPLE=$(RUST_EXAMPLE))..."
	@VERILATOR_TOP=$(TOP_MODULE) cargo run --manifest-path $(RUST_DIR)/Cargo.toml --example $(RUST_EXAMPLE)

rust-check: $(VSRCS)
	@echo "🦀 Checking Rust binder crate (TOP=$(TOP_MODULE))..."
	@VERILATOR_TOP=$(TOP_MODULE) cargo check --manifest-path $(RUST_DIR)/Cargo.toml

rust-test: $(VSRCS)
	@echo "🦀 Running Rust tests (TOPS=$(TOP_MODULES), TEST=$(RUST_TEST), FILTER=$(RUST_TEST_FILTER))..."
	@VERILATOR_TOPS="$(TOP_MODULES)" cargo test --manifest-path $(RUST_DIR)/Cargo.toml $(if $(RUST_TEST),--test $(RUST_TEST),) $(RUST_TEST_FILTER) -- --nocapture

lint:
	@echo "🔍 Running Verible linter..."
	@verible-verilog-lint $(VSRCS)

format:
	@echo "✨ Formatting SystemVerilog files..."
	@verible-verilog-format --inplace $(VSRCS)

clean:
	@echo "🧹 Cleaning build artifacts..."
	@rm -rf build obj_dir rust/target

help:
	@echo "SystemVerilog + Verilator + Rust Binder Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  make sim            - Run default Rust simulation (counter)"
	@echo "  make sim-counter    - Run counter RTL with Rust"
	@echo "  make sim-adder      - Run adder RTL with Rust"
	@echo "  make sim-pulse-counter - Run pulse_counter RTL with Rust"
	@echo "  make sim-toggle-pair - Run nested-module toggle_pair RTL with Rust"
	@echo "  make sim-dpi-adder  - Run DPI import adder RTL with Rust"
	@echo "  make rust-sim       - Run Rust simulation with TOP_MODULE/RUST_EXAMPLE"
	@echo "  make rust-check     - Type-check Rust binder crate"
	@echo "  make rust-test TOP_MODULES=top,adder_top,pulse_counter_top,toggle_pair_top,dpi_adder_top - Run all Rust tests"
	@echo "  make rust-test TOP_MODULES=dpi_adder_top RUST_TEST=dpi_adder_test - Run one test file"
	@echo "  make rust-test TOP_MODULES=toggle_pair_top RUST_TEST=toggle_pair_test RUST_TEST_FILTER=nested_modules_toggle_independently - Run one test function"
	@echo "  make lint           - Run Verible linter"
	@echo "  make format         - Format SystemVerilog with Verible"
	@echo "  make clean          - Remove build artifacts"
	@echo ""
	@echo "Examples:"
	@echo "  make rust-sim TOP_MODULE=adder_top RUST_EXAMPLE=adder"
	@echo "  make rust-check TOP_MODULE=top"
	@echo "  make rust-test TOP_MODULES=top,adder_top,pulse_counter_top,toggle_pair_top,dpi_adder_top"
	@echo "  make rust-test TOP_MODULES=dpi_adder_top RUST_TEST=dpi_adder_test"
