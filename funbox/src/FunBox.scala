package funbox

import chisel3._
import circt.stage.ChiselStage

class FunBoxIO extends Bundle {
  val clockFrequencyHz = Input(UInt(32.W))
  val led = Output(Bool())
}

class FunBox extends Module {
  val io = IO(new FunBoxIO)

  private val halfPeriod = io.clockFrequencyHz >> 1

  private val counter = RegInit(0.U(io.clockFrequencyHz.getWidth.W))
  private val ledReg = RegInit(false.B)

  private val nextCounter = counter + 1.U

  when(nextCounter === halfPeriod) {
    counter := 0.U
    ledReg := !ledReg
  }.otherwise {
    counter := nextCounter
  }

  io.led := ledReg
}

object FunBoxMain {
  def main(args: Array[String]): Unit = {
    require(args.length == 1, "Usage: FunBoxMain <output-directory>")
    val targetDir = args(0)

    ChiselStage.emitSystemVerilogFile(
      new FunBox,
      args = Array("--target-dir", targetDir)
    )

    println(s"Verilog written to $targetDir/FunBox.sv")
  }
}
