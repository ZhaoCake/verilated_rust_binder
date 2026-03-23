use verilated_rust_binder::dpi_adder_top::SimModel;

fn main() {
    let mut dut = SimModel::new();

    let cases = [(0u64, 0u64), (1, 2), (15, 27), (120, 130), (0xffff_ffff, 1)];

    println!("Starting Rust DPI-driven adder simulation...");

    for (a, b) in cases {
        dut.set_a(a);
        dut.set_b(b);
        dut.eval();
        println!("a={a:#010x}, b={b:#010x} => sum={:#010x}", dut.sum());
    }
}
