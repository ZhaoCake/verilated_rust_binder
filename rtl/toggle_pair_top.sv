module toggle_pair_top (
    input  logic clk,
    input  logic rst_n,
    input  logic en_a,
    input  logic en_b,
    output logic state_a,
    output logic state_b,
    output logic both_high
);

    toggle_cell u_toggle_a (
        .clk(clk),
        .rst_n(rst_n),
        .toggle_en(en_a),
        .state(state_a)
    );

    toggle_cell u_toggle_b (
        .clk(clk),
        .rst_n(rst_n),
        .toggle_en(en_b),
        .state(state_b)
    );

    always_comb begin
        both_high = state_a & state_b;
    end

endmodule
