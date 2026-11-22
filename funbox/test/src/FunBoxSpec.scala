package funbox

import chisel3._
import chisel3.simulator.EphemeralSimulator._
import org.scalatest.flatspec.AnyFlatSpec

class FunBoxSpec extends AnyFlatSpec {
  "FunBox" should "blink io.led every three cycles for a 6 Hz clock" in {
    simulate(new FunBox) { dut =>
      dut.io.clockFrequencyHz.poke(6.U)
      dut.reset.poke(true.B)
      dut.clock.step()
      dut.reset.poke(false.B)

      val onePeriod = Seq(false, false, false, true, true, true)
      (onePeriod ++ onePeriod).foreach { expected =>
        dut.io.led.expect(expected.B)
        dut.clock.step()
      }
    }
  }

  it should "flip io.led every cycle for a 2 Hz clock" in {
    simulate(new FunBox) { dut =>
      dut.io.clockFrequencyHz.poke(2.U)
      dut.reset.poke(true.B)
      dut.clock.step()
      dut.reset.poke(false.B)

      Seq(false, true, false, true, false, true, false, true).foreach {
        expected =>
          dut.io.led.expect(expected.B)
          dut.clock.step()
      }
    }
  }
}
