import "DPI-C" function int rust_add(
  input int a,
  input int b
);

module dpi_adder_top (
    input  logic [31:0] a,
    input  logic [31:0] b,
    output logic [31:0] sum
);

  always_comb begin
    sum = rust_add(int'(a), int'(b));
  end

endmodule
