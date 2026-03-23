use verilated_rust_binder::toggle_pair_top::SimModel;

fn tick(dut: &mut SimModel, en_a: bool, en_b: bool) {
    dut.set_clk(0);
    dut.set_en_a(u64::from(en_a));
    dut.set_en_b(u64::from(en_b));
    dut.eval();

    dut.set_clk(1);
    dut.eval();
}

fn main() {
    let mut dut = SimModel::new();

    dut.set_clk(0);
    dut.set_rst_n(0);
    dut.set_en_a(0);
    dut.set_en_b(0);
    dut.eval();
    dut.set_rst_n(1);

    let stimulus = [
        (true, false),
        (false, true),
        (true, true),
        (true, false),
        (false, true),
    ];

    for (cycle, (en_a, en_b)) in stimulus.into_iter().enumerate() {
        tick(&mut dut, en_a, en_b);
        println!(
            "cycle={cycle} en_a={} en_b={} state_a={} state_b={} both_high={}",
            u8::from(en_a),
            u8::from(en_b),
            dut.state_a(),
            dut.state_b(),
            dut.both_high()
        );
    }
}
