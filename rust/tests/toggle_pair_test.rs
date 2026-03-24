use verilated_rust_binder::toggle_pair_top::SimModel;
use std::sync::{Mutex, OnceLock};

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

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
    let _guard = test_lock().lock().expect("test lock poisoned");
    let mut dut = SimModel::new();
    let mut time = 0;
    reset(&mut dut);

    assert_eq!(dut.state_a(), 0);
    assert_eq!(dut.state_b(), 0);
    assert_eq!(dut.both_high(), 0);

    tick(&mut dut, &mut time, true, false);
    assert_eq!(dut.state_a(), 1);
    assert_eq!(dut.state_b(), 0);
    assert_eq!(dut.both_high(), 0);

    tick(&mut dut, &mut time, false, true);
    assert_eq!(dut.state_a(), 1);
    assert_eq!(dut.state_b(), 1);
    assert_eq!(dut.both_high(), 1);

    tick(&mut dut, &mut time, true, true);
    assert_eq!(dut.state_a(), 0);
    assert_eq!(dut.state_b(), 0);
    assert_eq!(dut.both_high(), 0);
}

#[test]
fn nested_modules_hold_state_when_disabled() {
    let _guard = test_lock().lock().expect("test lock poisoned");
    let mut dut = SimModel::new();
    let mut time = 0;
    reset(&mut dut);

    tick(&mut dut, &mut time, true, true);
    assert_eq!(dut.state_a(), 1);
    assert_eq!(dut.state_b(), 1);
    assert_eq!(dut.both_high(), 1);

    tick(&mut dut, &mut time, false, false);
    assert_eq!(dut.state_a(), 1);
    assert_eq!(dut.state_b(), 1);
    assert_eq!(dut.both_high(), 1);
}

