module scaffold;
  wire Cn;
  wire \~A0 ;
  wire \~A1 ;
  wire \~A2 ;
  wire \~A3 ;
  wire \~B0 ;
  wire \~B1 ;
  wire \~B2 ;
  wire \~B3 ;
  wire S0;
  wire S1;
  wire S2;
  wire S3;
  wire M;
  wire VCC;
  wire GND;
  wire \~G ;
  wire \~P ;
  wire \~F0 ;
  wire \~F1 ;
  wire \~F2 ;
  wire \~F3 ;
  wire \A=B ;
  wire \Cn+4 ;

  \74181 dut (
      Cn,
      \~A0 ,
      \~A1 ,
      \~A2 ,
      \~A3 ,
      \~B0 ,
      \~B1 ,
      \~B2 ,
      \~B3 ,
      S0,
      S1,
      S2,
      S3,
      M,
      VCC,
      GND,
      \~G ,
      \~P ,
      \~F0 ,
      \~F1 ,
      \~F2 ,
      \~F3 ,
      \A=B ,
      \Cn+4
  );
  tb tb (
      \S3 ,
      \S2 ,
      \S1 ,
      \S0 ,
      \M ,
      \Cn ,
      \~A3 ,
      \~A2 ,
      \~A1 ,
      \~A0 ,
      \~B3 ,
      \~B2 ,
      \~B1 ,
      \~B0 ,
      \~F3 ,
      \~F2 ,
      \~F1 ,
      \~F0
  );
endmodule
