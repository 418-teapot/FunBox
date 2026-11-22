.PHONY: synth pnr bitstream program

synth: $(FPGA_DIR)/synth.json
pnr: $(FPGA_DIR)/pnr.json
bitstream: $(FPGA_DIR)/bitstream.fs

program: $(FPGA_DIR)/bitstream.fs
	openFPGALoader -b $(OPENFPGA_BOARD) $<

$(FPGA_DIR)/synth.json: out/funbox/verilog.dest/FunBox.sv $(BOARD_DIR)/top.sv $(BOARD_DIR)/board.mk
	mkdir -p $(@D)
	yosys -p 'read_verilog -sv -DSYNTHESIS -DCLOCK_HZ=$(CLOCK_HZ) out/funbox/verilog.dest/FunBox.sv $(BOARD_DIR)/top.sv; synth_gowin -top Top -json $@'

$(FPGA_DIR)/pnr.json: $(FPGA_DIR)/synth.json $(BOARD_DIR)/pins.cst
	nextpnr-himbaechel --device '$(GOWIN_DEVICE)' --json $< \
		--vopt cst=$(BOARD_DIR)/pins.cst \
		--freq $(shell expr $(CLOCK_HZ) / 1000000) \
		--write $@

$(FPGA_DIR)/bitstream.fs: $(FPGA_DIR)/pnr.json
	gowin_pack -d $(GOWIN_PACK_DEVICE) -o $@ $<
