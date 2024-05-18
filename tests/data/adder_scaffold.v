module scaffold;
  wire [7:0] \A ;
  wire [7:0] \B ;
  wire [7:0] \S ;
  wire \C ;

  adder adder0 (
      \A ,
      \B ,
      \S ,
      \C
  );
  tb tb0 (
      \A ,
      \B ,
      \S ,
      \C
  );
endmodule
