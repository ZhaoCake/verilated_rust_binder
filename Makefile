# SystemVerilog + Verilator + Rust Binder Makefile
.PHONY: all sim sim-counter sim-adder sim-pulse-counter rust-sim rust-check rust-test lint format clean help

TOP_MODULE ?= top
RUST_EXAMPLE ?= counter
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

rust-sim: $(VSRCS)
	@echo "🦀 Running Rust simulation (TOP=$(TOP_MODULE), EXAMPLE=$(RUST_EXAMPLE))..."
	@VERILATOR_TOP=$(TOP_MODULE) cargo run --manifest-path $(RUST_DIR)/Cargo.toml --example $(RUST_EXAMPLE)

rust-check: $(VSRCS)
	@echo "🦀 Checking Rust binder crate (TOP=$(TOP_MODULE))..."
	@VERILATOR_TOP=$(TOP_MODULE) cargo check --manifest-path $(RUST_DIR)/Cargo.toml

rust-test: $(VSRCS)
	@echo "🦀 Running Rust tests (TOPS=$(TOP_MODULES))..."
	@VERILATOR_TOPS="$(TOP_MODULES)" cargo test --manifest-path $(RUST_DIR)/Cargo.toml --test pulse_counter_test -- --nocapture

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
	@echo "  make rust-sim       - Run Rust simulation with TOP_MODULE/RUST_EXAMPLE"
	@echo "  make rust-check     - Type-check Rust binder crate"
	@echo "  make rust-test TOP_MODULES=top,adder_top,pulse_counter_top - Run Rust integration tests"
	@echo "  make lint           - Run Verible linter"
	@echo "  make format         - Format SystemVerilog with Verible"
	@echo "  make clean          - Remove build artifacts"
	@echo ""
	@echo "Examples:"
	@echo "  make rust-sim TOP_MODULE=adder_top RUST_EXAMPLE=adder"
	@echo "  make rust-check TOP_MODULE=top"
	@echo "  make rust-test TOP_MODULES=top,adder_top,pulse_counter_top"
