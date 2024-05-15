//  A testbench for adder_Simple_tb
`timescale 1us/1ns

module adder_Simple_tb;
    reg [7:0] A;
    reg [7:0] B;
    wire [7:0] S;
    wire C;

  adder adder0 (
    .A(A),
    .B(B),
    .S(S),
    .C(C)
  );

    reg [24:0] patterns [0:0];
    integer i;

    initial begin
      patterns[0] = 25'b00000001_00000001_00000010_x;

      for (i = 0; i < 1; i = i + 1)
      begin
        A = patterns[i][24:17];
        B = patterns[i][16:9];
        #10;
        if (patterns[i][8:1] !== 8'hx)
        begin
          if (S !== patterns[i][8:1])
          begin
            $display("%d:S: (assertion error). Expected %h, found %h", i, patterns[i][8:1], S);
            $finish;
          end
        end
        if (patterns[i][0] !== 1'hx)
        begin
          if (C !== patterns[i][0])
          begin
            $display("%d:C: (assertion error). Expected %h, found %h", i, patterns[i][0], C);
            $finish;
          end
        end
      end

      $display("All tests passed.");
    end
    endmodule
