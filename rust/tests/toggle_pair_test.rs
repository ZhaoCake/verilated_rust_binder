use verilated_rust_binder::toggle_pair_top::SimModel;

fn tick(dut: &mut SimModel, en_a: bool, en_b: bool) {
    dut.set_clk(0);
    dut.set_en_a(u64::from(en_a));
    dut.set_en_b(u64::from(en_b));
    dut.eval();

    dut.set_clk(1);
    dut.eval();
}

fn reset(dut: &mut SimModel) {
    dut.set_clk(0);
    dut.set_rst_n(0);
    dut.set_en_a(0);
    dut.set_en_b(0);
    dut.eval();
    dut.set_rst_n(1);
}

#[test]
fn nested_modules_toggle_independently() {
    let mut dut = SimModel::new();
    reset(&mut dut);

    assert_eq!(dut.state_a(), 0);
    assert_eq!(dut.state_b(), 0);
    assert_eq!(dut.both_high(), 0);

    tick(&mut dut, true, false);
    assert_eq!(dut.state_a(), 1);
    assert_eq!(dut.state_b(), 0);
    assert_eq!(dut.both_high(), 0);

    tick(&mut dut, false, true);
    assert_eq!(dut.state_a(), 1);
    assert_eq!(dut.state_b(), 1);
    assert_eq!(dut.both_high(), 1);

    tick(&mut dut, true, true);
    assert_eq!(dut.state_a(), 0);
    assert_eq!(dut.state_b(), 0);
    assert_eq!(dut.both_high(), 0);
}

#[test]
fn nested_modules_hold_state_when_disabled() {
    let mut dut = SimModel::new();
    reset(&mut dut);

    tick(&mut dut, true, true);
    assert_eq!(dut.state_a(), 1);
    assert_eq!(dut.state_b(), 1);
    assert_eq!(dut.both_high(), 1);

    tick(&mut dut, false, false);
    assert_eq!(dut.state_a(), 1);
    assert_eq!(dut.state_b(), 1);
    assert_eq!(dut.both_high(), 1);
}
