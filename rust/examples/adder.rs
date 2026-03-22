use verilated_rust_binder::SimModel;

fn main() {
    let mut dut = SimModel::new();

    let cases = [(0u64, 0u64), (1, 2), (15, 27), (120, 130), (255, 1)];

    println!("Starting Rust-driven adder simulation...");

    for (a, b) in cases {
        dut.set_a(a);
        dut.set_b(b);
        dut.eval();
        let sum = dut.sum();
        println!("a={:>3}, b={:>3} => sum={:>3}", a, b, sum);
    }

    println!("Adder simulation complete!");
}
