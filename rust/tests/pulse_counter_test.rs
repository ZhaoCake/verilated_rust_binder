use verilated_rust_binder::pulse_counter_top::SimModel;

fn tick(dut: &mut SimModel, pulse: bool) {
    dut.set_clk(0);
    dut.set_pulse(u64::from(pulse));
    dut.eval();

    dut.set_clk(1);
    dut.eval();
}

fn reset(dut: &mut SimModel) {
    dut.set_clk(0);
    dut.set_rst_n(0);
    dut.set_pulse(0);
    dut.eval();
    dut.set_rst_n(1);
}

#[test]
fn pulse_increments_counter_only_when_asserted() {
    let mut dut = SimModel::new();
    reset(&mut dut);

    tick(&mut dut, false);
    assert_eq!(dut.count(), 0);
    assert_eq!(dut.overflow(), 0);

    tick(&mut dut, true);
    assert_eq!(dut.count(), 1);
    assert_eq!(dut.overflow(), 0);

    tick(&mut dut, false);
    assert_eq!(dut.count(), 1);
    assert_eq!(dut.overflow(), 0);

    tick(&mut dut, true);
    assert_eq!(dut.count(), 2);
    assert_eq!(dut.overflow(), 0);
}

#[test]
fn pulse_counter_wraps_and_raises_overflow_for_one_cycle() {
    let mut dut = SimModel::new();
    reset(&mut dut);

    for _ in 0..255 {
        tick(&mut dut, true);
    }

    assert_eq!(dut.count(), 255);
    assert_eq!(dut.overflow(), 0);

    tick(&mut dut, true);
    assert_eq!(dut.count(), 0);
    assert_eq!(dut.overflow(), 1);

    tick(&mut dut, false);
    assert_eq!(dut.count(), 0);
    assert_eq!(dut.overflow(), 0);
}
