module Top (
  input  clock,
  input  reset_n,
  output io_led
);

  // Dock 的 KEY0 低电平有效且无外部上拉；FunBox 的 reset 为高有效，需反相接入。
  wire reset = ~reset_n;

  FunBox core (
    .clock(clock),
    .reset(reset),
    .io_clockFrequencyHz(`CLOCK_HZ),
    .io_led(io_led)
  );

endmodule
