package funbox

import chisel3._
import chisel3.simulator.EphemeralSimulator._
import org.scalatest.flatspec.AnyFlatSpec

class FunBoxSpec extends AnyFlatSpec {
  "FunBox" should "drive io.out with 42" in {
    simulate(new FunBox) { dut =>
      dut.io.out.expect(42.U)
    }
  }
}
