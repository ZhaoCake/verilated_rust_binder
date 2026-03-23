use verilated_rust_binder::top::SimModel;

fn main() {
    let mut dut = SimModel::new();

    dut.set_clk(0);
    dut.set_rst_n(0);
    dut.set_en(0);
    dut.eval();

    println!("Starting Rust-driven simulation...");

    for cycle in 0..100 {
        dut.set_clk(0);
        dut.eval();

        dut.set_clk(1);
        dut.eval();

        if cycle == 5 {
            dut.set_rst_n(1);
        }

        if cycle == 10 {
            dut.set_en(1);
        }

        if cycle >= 10 && cycle % 5 == 0 {
            println!("Cycle {:>3}: count = 0x{:02x} ({})", cycle, dut.count(), dut.count());
        }
    }

    println!("Simulation complete!");
}
