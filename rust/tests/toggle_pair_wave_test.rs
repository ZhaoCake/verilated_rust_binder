use std::fs;

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

fn reset(dut: &mut SimModel) {
    dut.set_clk(0);
    dut.set_rst_n(0);
    dut.set_en_a(0);
    dut.set_en_b(0);
    dut.eval();
    dut.dump_vcd(0);
    dut.set_rst_n(1);
    dut.eval();
    dut.dump_vcd(5);
}

#[test]
fn toggle_pair_can_export_vcd_waveform() {
    let mut dut = SimModel::new();
    let mut time = 10;
    let wave_path = std::env::temp_dir().join(format!(
        "toggle_pair_wave_{}.vcd",
        std::process::id()
    ));

    let _ = fs::remove_file(&wave_path);

    dut.enable_vcd(&wave_path, 99);
    reset(&mut dut);

    tick(&mut dut, &mut time, true, false);
    tick(&mut dut, &mut time, false, true);
    tick(&mut dut, &mut time, true, true);

    dut.flush_vcd();
    dut.close_vcd();

    let metadata = fs::metadata(&wave_path).expect("wave file should be created");
    assert!(metadata.len() > 0, "wave file should not be empty");

    let content = fs::read_to_string(&wave_path).expect("wave file should be readable as text VCD");
    assert!(content.contains("$var"), "vcd should contain signal definitions");
    assert!(content.contains("state_a"), "vcd should contain state_a signal");
    assert!(content.contains("state_b"), "vcd should contain state_b signal");
    assert!(content.contains("both_high"), "vcd should contain both_high signal");

    let _ = fs::remove_file(&wave_path);
}