TOOLS_PYTHON ?= uv run python
TOOLS_RUFF ?= uv run ruff

.PHONY: tools-test tools-check tools-format

tools-test:
	$(TOOLS_PYTHON) scripts/pnr_report_test.py

tools-check:
	$(TOOLS_RUFF) check .
	$(TOOLS_RUFF) format --check .

tools-format:
	$(TOOLS_RUFF) format .
