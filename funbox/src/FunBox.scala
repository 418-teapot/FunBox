package funbox

import chisel3._
import circt.stage.ChiselStage

class FunBoxIO extends Bundle {
  val out = Output(UInt(8.W))
}

/** A minimal synthesizable circuit: drive an 8-bit output with the value 42. */
class FunBox extends Module {
  val io = IO(new FunBoxIO)

  io.out := 42.U
}

object FunBoxMain {
  def main(args: Array[String]): Unit = {
    require(args.length == 1, "Usage: FunBoxMain <output-directory>")
    val targetDir = args(0)

    ChiselStage.emitSystemVerilogFile(
      new FunBox,
      args = Array("--target-dir", targetDir)
    )

    println("FunBox: Chisel is working.")
    println(s"Verilog written to $targetDir/FunBox.sv")
  }
}
