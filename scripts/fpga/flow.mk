FPGA_PYTHON ?= uv run python

.PHONY: fpga-clean

fpga-clean:
	rm -rf -- out/fpga

ifneq ($(strip $(BOARD)),)

include boards/$(BOARD)/board.mk

FPGA_BOARD_DIR := boards/$(BOARD)
FPGA_OUT_DIR := out/fpga/$(BOARD)

.PHONY: fpga-check

fpga-check: fpga-pnr

include scripts/fpga/$(FPGA_FLOW).mk

endif
