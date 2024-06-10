module scaffold;
  wire \~LD ;
  wire \~CLR ;
  wire CLK;
  wire ENT;
  wire ENP;
  wire A;
  wire B;
  wire C;
  wire D;
  wire VCC;
  wire GND;
  wire RCO;
  wire QA;
  wire QB;
  wire QC;
  wire QD;

  \74162 dut (
      \~LD ,
      \~CLR ,
      CLK,
      ENT,
      ENP,
      A,
      B,
      C,
      D,
      VCC,
      GND,
      RCO,
      QA,
      QB,
      QC,
      QD
  );
  tb tb (
      \~LD ,
      \~CLR ,
      CLK,
      ENT,
      ENP,
      A,
      B,
      C,
      D,
      VCC,
      GND,
      RCO,
      QA,
      QB,
      QC,
      QD
  );
endmodule
