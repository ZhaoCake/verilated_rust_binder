use verilated_rust_binder::dpi_adder_top::SimModel;

#[test]
fn dpi_import_calls_into_rust_function() {
    let mut dut = SimModel::new();

    let cases = [
        (0u64, 0u64, 0u64),
        (1, 2, 3),
        (15, 27, 42),
        (120, 130, 250),
        (0xffff_ffff, 1, 0),
    ];

    for (a, b, expected) in cases {
        dut.set_a(a);
        dut.set_b(b);
        dut.eval();
        assert_eq!(dut.sum(), expected, "a={a:#x}, b={b:#x}");
    }
}
