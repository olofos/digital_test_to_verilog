module scaffold ();
  wire CP;
  wire S0;
  wire S1;
  wire \~OE ;
  wire InotO0;
  wire InotO1;
  wire InotO2;
  wire InotO3;
  wire InotO4;
  wire InotO5;
  wire InotO6;
  wire InotO7;
  wire \~CET ;
  wire VCC;
  wire GND;
  wire \~TC ;

  \74f779 dut (
      CP,
      S0,
      S1,
      \~OE ,
      InotO0,
      InotO1,
      InotO2,
      InotO3,
      InotO4,
      InotO5,
      InotO6,
      InotO7,
      \~CET ,
      VCC,
      GND,
      \~TC
  );

  tb tb0 (
      CP,
      S0,
      S1,
      \~OE ,
      InotO0,
      InotO1,
      InotO2,
      InotO3,
      InotO4,
      InotO5,
      InotO6,
      InotO7,
      \~CET ,
      VCC,
      GND,
      \~TC
  );

endmodule
