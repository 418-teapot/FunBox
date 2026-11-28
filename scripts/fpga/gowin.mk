FPGA_RTL := $(FUNBOX_RTL)
FPGA_PNR_OUTPUTS := $(FPGA_OUT_DIR)/pnr.json \
	$(FPGA_OUT_DIR)/report.json \
	$(FPGA_OUT_DIR)/pnr.log

FPGA_YOSYS_SCRIPT = read_verilog -sv -DSYNTHESIS \
	-DCLOCK_HZ=$(FPGA_CLOCK_HZ) $(FPGA_RTL) $(FPGA_BOARD_DIR)/top.sv; \
	synth_gowin -top Top -json $@

.PHONY: fpga fpga-synth fpga-pnr fpga-bitstream fpga-program

fpga: fpga-bitstream

fpga-synth: $(FPGA_OUT_DIR)/synth.json

fpga-pnr: $(FPGA_PNR_OUTPUTS)
	$(FPGA_PYTHON) scripts/pnr_report.py \
		--board '$(BOARD)' \
		--device '$(GOWIN_DEVICE)' \
		--report $(FPGA_OUT_DIR)/report.json \
		--log $(FPGA_OUT_DIR)/pnr.log

fpga-bitstream: $(FPGA_OUT_DIR)/bitstream.fs

fpga-program: $(FPGA_OUT_DIR)/bitstream.fs
	openFPGALoader -b $(OPENFPGA_BOARD) $<

$(FPGA_OUT_DIR)/synth.json: $(FPGA_RTL) \
	$(FPGA_BOARD_DIR)/top.sv $(FPGA_BOARD_DIR)/board.mk
	mkdir -p $(@D)
	yosys -p '$(FPGA_YOSYS_SCRIPT)'

$(FPGA_PNR_OUTPUTS) &: $(FPGA_OUT_DIR)/synth.json $(FPGA_BOARD_DIR)/pins.cst
	nextpnr-himbaechel --device '$(GOWIN_DEVICE)' --json $< \
		--quiet \
		--vopt cst=$(FPGA_BOARD_DIR)/pins.cst \
		--freq $(shell expr $(FPGA_CLOCK_HZ) / 1000000) \
		--report $(FPGA_OUT_DIR)/report.json \
		--detailed-timing-report \
		--log $(FPGA_OUT_DIR)/pnr.log \
		--write $(FPGA_OUT_DIR)/pnr.json

# Order-only: run the timing gate, but ignore output timestamps.
$(FPGA_OUT_DIR)/bitstream.fs: $(FPGA_PNR_OUTPUTS) | fpga-pnr
	gowin_pack -d $(GOWIN_PACK_DEVICE) -o $@ $<
