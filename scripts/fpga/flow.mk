BOARD_DIR := boards/$(BOARD)
include $(BOARD_DIR)/board.mk

FPGA_DIR := out/fpga/$(BOARD)

include scripts/fpga/$(FLOW).mk
