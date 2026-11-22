ifeq ($(shell uname -s),Darwin)
COURSIER_ROOT ?= $(HOME)/Library/Caches/Coursier
else
COURSIER_ROOT ?= $(or $(XDG_CACHE_HOME),$(HOME)/.cache)/coursier
endif

.PHONY: all verilog test format check clean expunge

all: check verilog test

verilog:
	mill funbox.verilog

test:
	mill funbox.test

format:
	mill funbox.reformat
	mill funbox.test.reformat

check:
	mill funbox.checkFormat
	mill funbox.test.checkFormat

clean:
	mill clean

expunge: clean
	rm -rf -- "$(COURSIER_ROOT)"

BOARD_TARGETS := synth pnr bitstream program

ifneq ($(filter $(BOARD_TARGETS),$(MAKECMDGOALS)),)

  ifeq ($(strip $(BOARD)),)
    $(error BOARD required, e.g. make bitstream BOARD=tang-primer-25k)
  endif

  ifeq ($(wildcard boards/$(BOARD)/board.mk),)
    $(error unknown BOARD '$(BOARD)', available: $(notdir $(wildcard boards/*)))
  endif

  .PHONY: $(BOARD_TARGETS)

$(BOARD_TARGETS):
	mill funbox.verilog
	$(MAKE) -f scripts/fpga/flow.mk $@ BOARD=$(BOARD)

endif
