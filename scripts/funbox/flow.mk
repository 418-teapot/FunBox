FUNBOX_MILL ?= mill

# A real file target: phony would rebuild the FPGA on every run.
FUNBOX_RTL := out/funbox/verilog.dest/FunBox.sv
FUNBOX_RTL_INPUTS := build.mill .mill-jvm-version $(wildcard funbox/src/*.scala)

.PHONY: funbox funbox-test funbox-format funbox-check funbox-clean

funbox: $(FUNBOX_RTL)

$(FUNBOX_RTL): $(FUNBOX_RTL_INPUTS)
	$(FUNBOX_MILL) funbox.verilog

funbox-test:
	$(FUNBOX_MILL) funbox.test

funbox-format:
	$(FUNBOX_MILL) funbox.reformat
	$(FUNBOX_MILL) funbox.test.reformat

funbox-check:
	$(FUNBOX_MILL) funbox.checkFormat
	$(FUNBOX_MILL) funbox.test.checkFormat

funbox-clean:
	$(FUNBOX_MILL) clean
