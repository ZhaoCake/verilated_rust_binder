module pulse_counter_top (
    input  logic       clk,
    input  logic       rst_n,
    input  logic       pulse,
    output logic [7:0] count,
    output logic       overflow
);

    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            count <= 8'h00;
            overflow <= 1'b0;
        end else begin
            overflow <= 1'b0;
            if (pulse) begin
                if (count == 8'hff) begin
                    count <= 8'h00;
                    overflow <= 1'b1;
                end else begin
                    count <= count + 8'h01;
                end
            end
        end
    end

endmodule
