ifeq ($(shell uname -s),Darwin)
COURSIER_ROOT ?= $(HOME)/Library/Caches/Coursier
else
COURSIER_ROOT ?= $(or $(XDG_CACHE_HOME),$(HOME)/.cache)/coursier
endif

FLOWS := funbox rust fpga tools

FPGA_TOOL_TARGETS := fpga fpga-synth fpga-pnr fpga-bitstream \
	fpga-program fpga-check

AVAILABLE_BOARDS := $(notdir $(wildcard boards/*))

.DEFAULT_GOAL := all

.PHONY: all build test check format clean expunge

all: check build test
build: funbox rust $(if $(BOARD),fpga)
test: funbox-test rust-test tools-test
check: funbox-check rust-check tools-check $(if $(BOARD),fpga-check)
format: funbox-format rust-format tools-format
clean: funbox-clean rust-clean fpga-clean

expunge: clean
	rm -rf -- "$(COURSIER_ROOT)"

ifneq ($(filter $(FPGA_TOOL_TARGETS),$(MAKECMDGOALS)),)
ifeq ($(strip $(BOARD)),)
$(error BOARD required, e.g. make fpga-bitstream BOARD=tang-primer-25k)
endif
endif

ifneq ($(strip $(BOARD)),)
ifeq ($(wildcard boards/$(BOARD)/board.mk),)
$(error unknown BOARD '$(BOARD)', available: $(AVAILABLE_BOARDS))
endif
endif

include $(addsuffix /flow.mk,$(addprefix scripts/,$(FLOWS)))
