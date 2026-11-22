module Top (
  input  clock,
  input  reset,
  output io_led
);

  FunBox core (
    .clock(clock),
    .reset(reset),
    .io_clockFrequencyHz(`CLOCK_HZ),
    .io_led(io_led)
  );

endmodule
