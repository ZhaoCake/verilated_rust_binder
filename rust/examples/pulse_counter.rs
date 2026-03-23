use verilated_rust_binder::pulse_counter_top::SimModel;

fn tick(dut: &mut SimModel, pulse: bool) {
    dut.set_clk(0);
    dut.set_pulse(u64::from(pulse));
    dut.eval();

    dut.set_clk(1);
    dut.eval();
}

fn main() {
    let mut dut = SimModel::new();

    dut.set_clk(0);
    dut.set_rst_n(0);
    dut.set_pulse(0);
    dut.eval();

    dut.set_rst_n(1);

    for cycle in 0..12 {
        let pulse = cycle % 2 == 0;
        tick(&mut dut, pulse);
        println!(
            "cycle={cycle:>2} pulse={} count={} overflow={}",
            u8::from(pulse),
            dut.count(),
            dut.overflow()
        );
    }
}
