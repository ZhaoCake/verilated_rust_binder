module toggle_cell (
    input  logic clk,
    input  logic rst_n,
    input  logic toggle_en,
    output logic state
);

  always_ff @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
      state <= 1'b0;
    end else if (toggle_en) begin
      state <= ~state;
    end
  end

endmodule
