use verilated_rust_binder::toggle_pair_top::SimModel;

fn tick(dut: &mut SimModel, time: &mut u64, en_a: bool, en_b: bool) {
    dut.set_clk(0);
    dut.set_en_a(u64::from(en_a));
    dut.set_en_b(u64::from(en_b));
    dut.eval();
    dut.dump_vcd(*time);
    *time += 5;

    dut.set_clk(1);
    dut.eval();
    dut.dump_vcd(*time);
    *time += 5;
}

fn main() {
    let mut dut = SimModel::new();
    let mut time = 0;

    dut.enable_vcd("wave_toggle_pair.vcd", 99);

    dut.set_clk(0);
    dut.set_rst_n(0);
    dut.set_en_a(0);
    dut.set_en_b(0);
    dut.eval();
    dut.dump_vcd(time);
    time += 5;
    dut.set_rst_n(1);
    dut.eval();
    dut.dump_vcd(time);
    time += 5;

    let stimulus = [
        (true, false),
        (false, true),
        (true, true),
        (true, false),
        (false, true),
    ];

    for (cycle, (en_a, en_b)) in stimulus.into_iter().enumerate() {
        tick(&mut dut, &mut time, en_a, en_b);
        println!(
            "cycle={cycle} en_a={} en_b={} state_a={} state_b={} both_high={}",
            u8::from(en_a),
            u8::from(en_b),
            dut.state_a(),
            dut.state_b(),
            dut.both_high()
        );
    }

    dut.flush_vcd();
    dut.close_vcd();
    println!("waveform written to wave_toggle_pair.vcd");
}
