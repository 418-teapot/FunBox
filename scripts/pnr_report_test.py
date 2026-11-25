"""Behavior tests for the nextpnr terminal report."""

import contextlib
import io
import json
import os
import re
import tempfile
import unittest
from pathlib import Path

from pnr_report import (
    build_snapshot,
    main,
    render_report,
    select_resources,
)
from rich.console import Console

BOARD = "tang-primer-25k"
DEVICE = "GW5A-LV25MG121NC1/I0"
REPORT_PATH = "out/fpga/tang-primer-25k/report.json"
LOG_PATH = "out/fpga/tang-primer-25k/pnr.log"
SOURCE_LOCATIONS = (
    "source-alpha",
    "source-beta.sv:63.24-63.39",
)
SAMPLE_LOG = """\
Info: Packing
Warning: Failed to route net 'core.reset'
Warning: Second warning
Warning: Third warning
3 warnings, 0 errors
Info: Program finished normally.
"""


def report_dict():
    delays = (0.231, 0.142, 0.549, 0.054, 0.469, 0.045, 0.569, 0.048)
    kinds = (
        "clk-to-q",
        "routing",
        "logic",
        "setup",
        "logic",
        "routing",
        "logic",
        "routing",
    )
    segments = []
    for index, (delay, kind) in enumerate(zip(delays, kinds, strict=True)):
        segments.append(
            {
                "delay": delay,
                "type": kind,
                "from": {
                    "cell": f"core.reg{index}",
                    "port": "Q",
                    "loc": [index, 0],
                },
                "to": {
                    "cell": f"core.reg{index + 1}",
                    "port": "D",
                    "loc": [index + 1, 0],
                },
                "net": f"core.n{index + 1}",
                "sources": list(SOURCE_LOCATIONS) if index == 0 else [],
            }
        )
    return {
        "fmax": {"core.clock": {"achieved": 193.6483, "constraint": 50}},
        "utilization": {
            "ALU": {"available": 17280, "used": 32},
            "LUT4": {"available": 23040, "used": 24},
            "DFF": {"available": 23040, "used": 33},
            "IOB": {"available": 240, "used": 3},
            "BSRAM": {"available": 56, "used": 0},
            "GND": {"available": 1, "used": 1},
            "MUX2_LUT7": {"available": 10, "used": 0},
            "OSCA": {"available": 1, "used": 0},
        },
        "critical_paths": [
            {
                "from": "posedge core.clock",
                "to": "posedge core.clock",
                "path": segments,
            }
        ],
    }


def make_snapshot(report=None, log_text=SAMPLE_LOG):
    if report is None:
        report = report_dict()
    return build_snapshot(
        report,
        log_text,
        board=BOARD,
        device=DEVICE,
        report_path=REPORT_PATH,
        log_path=LOG_PATH if log_text is not None else None,
    )


def tiny_snapshot():
    report = {
        "fmax": {"clk": {"achieved": 20, "constraint": 10}},
        "utilization": {"LUT4": {"available": 10, "used": 1}},
        "critical_paths": [
            {
                "from": "posedge clk",
                "to": "posedge clk",
                "path": [
                    {
                        "type": "clk-to-q",
                        "delay": 0.5,
                        "from": {"cell": "a", "port": "Q", "loc": [1, 2]},
                        "to": {"cell": "b", "port": "D", "loc": [3, 4]},
                        "net": "n1",
                        "sources": [],
                    },
                    {
                        "type": "routing",
                        "delay": 1.5,
                        "from": {"cell": "a", "port": "Q", "loc": [1, 2]},
                        "to": {"cell": "b", "port": "D", "loc": [3, 4]},
                        "net": "n2",
                        "sources": [],
                    },
                ],
            }
        ],
    }
    log = (
        "Warning: check route\n"
        "1 warning, 0 errors\n"
        "Program finished normally.\n"
    )
    return build_snapshot(
        report,
        log,
        board="board",
        device="device",
        report_path="report.json",
        log_path="pnr.log",
    )


def render(snapshot=None, *, color=False, width=100, **options):
    output = io.StringIO()
    console = Console(
        file=output,
        width=width,
        height=200,
        color_system="standard" if color else None,
        force_terminal=color,
        legacy_windows=False,
    )
    render_report(console, snapshot or make_snapshot(), **options)
    return output.getvalue()


def line_with(text, output):
    matches = [line for line in output.splitlines() if text in line]
    if not matches:
        raise AssertionError(f"no line containing {text!r} in:\n{output}")
    return matches[0]


ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")


def has_bold(line):
    return "\x1b[1m" in line or "\x1b[1;" in line


def title_line(title, output):
    for line in output.splitlines():
        plain = ANSI_RE.sub("", line)
        if title in plain and (plain.strip() == title or "─" in plain):
            return line
    raise AssertionError(f"no title line containing {title!r} in:\n{output}")


def normalize(output):
    """Ignore Rich's padding at line ends, but no visible report content."""
    return "\n".join(line.rstrip() for line in output.splitlines()) + "\n"


EXPECTED_TINY = """\
╭───────────────────────── FPGA Place & Route ─────────────────────────╮
│ Board          board                                                 │
│ Device         device                                                │
│ P&R status     SUCCESS                                               │
│ Timing status  PASS                                                  │
│ Warnings       1                                                     │
│ Errors         0                                                     │
╰──────────────────────────────────────────────────────────────────────╯

                         Timing
╭───────┬───────────┬───────────┬─────────────┬────────╮
│ Clock │    Target │  Achieved │ Worst slack │ Result │
├───────┼───────────┼───────────┼─────────────┼────────┤
│ clk   │ 10.00 MHz │ 20.00 MHz │   +50.00 ns │ PASS   │
╰───────┴───────────┴───────────┴─────────────┴────────╯

            Resource Utilization
╭──────────┬──────┬───────────┬─────────────╮
│ Resource │ Used │ Available │ Utilization │
├──────────┼──────┼───────────┼─────────────┤
│ LUT4     │    1 │        10 │      10.00% │
╰──────────┴──────┴───────────┴─────────────╯

╭────────────────────────── Critical Path 1 ───────────────────────────╮
│ Clock          posedge clk → posedge clk                             │
│ Total delay    2.000 ns                                              │
│ From           a.Q (1, 2)                                            │
│ To             b.D (3, 4)                                            │
│ Delay by type  routing 1.500 ns · clk-to-q 0.500 ns                  │
│ HDL sources    —                                                     │
╰──────────────────────────────────────────────────────────────────────╯

                               2 segments
╭──────────────┬──────────────┬────────────────────────────────────────╮
│ Type         │        Delay │ Net / destination                      │
├──────────────┼──────────────┼────────────────────────────────────────┤
│ routing      │     1.500 ns │ n2                                     │
│ clk-to-q     │     0.500 ns │ n1                                     │
╰──────────────┴──────────────┴────────────────────────────────────────╯

╭──────────────────────────── Warnings (1) ────────────────────────────╮
│ • check route                                                        │
╰──────────────────────────────────────────────────────────────────────╯

report.json    report.json
pnr.log        pnr.log
"""


class SnapshotTests(unittest.TestCase):
    def test_clock_uses_final_json_values_and_derives_slack(self):
        snapshot = make_snapshot()

        clock = snapshot.clocks[0]
        self.assertEqual(clock.name, "core.clock")
        self.assertEqual(clock.target_mhz, 50)
        self.assertAlmostEqual(clock.achieved_mhz, 193.6483)
        self.assertAlmostEqual(clock.slack_ns, 14.835999, places=5)
        self.assertEqual(clock.result, "PASS")
        self.assertEqual(snapshot.timing_status, "PASS")

    def test_failing_clock_does_not_change_successful_pnr(self):
        report = report_dict()
        report["fmax"]["core.clock"]["achieved"] = 40

        snapshot = make_snapshot(report)

        self.assertEqual(snapshot.pnr_status, "SUCCESS")
        self.assertEqual(snapshot.timing_status, "FAIL")
        self.assertLess(snapshot.clocks[0].slack_ns, 0)

    def test_invalid_clock_values_are_unknown(self):
        for fmax in (
            {},
            {"clk": {"achieved": 20}},
            {"clk": {"achieved": 0, "constraint": 10}},
            {"clk": {"achieved": -20, "constraint": 10}},
            {"clk": {"achieved": float("nan"), "constraint": 10}},
        ):
            with self.subTest(fmax=fmax):
                snapshot = make_snapshot({"fmax": fmax})
                self.assertEqual(snapshot.timing_status, "UNKNOWN")
                if snapshot.clocks:
                    self.assertIsNone(snapshot.clocks[0].slack_ns)
                    self.assertEqual(snapshot.clocks[0].result, "UNKNOWN")

    def test_any_failing_clock_fails_the_design(self):
        report = {
            "fmax": {
                "fast": {"achieved": 100, "constraint": 50},
                "slow": {"achieved": 20, "constraint": 50},
            }
        }

        snapshot = make_snapshot(report)

        self.assertEqual(
            [clock.result for clock in snapshot.clocks], ["PASS", "FAIL"]
        )
        self.assertEqual(snapshot.timing_status, "FAIL")

    def test_log_messages_declared_counts_and_status(self):
        snapshot = make_snapshot()

        self.assertEqual(snapshot.log.warning_count, 3)
        self.assertEqual(snapshot.log.error_count, 0)
        self.assertEqual(snapshot.pnr_status, "SUCCESS")
        self.assertIn("core.reset", snapshot.log.warnings[0])

    def test_missing_log_and_errors_are_distinct_from_timing(self):
        self.assertEqual(make_snapshot(log_text=None).pnr_status, "UNKNOWN")
        failed = make_snapshot(log_text="ERROR: broken\n0 warnings, 1 error\n")
        self.assertEqual(failed.pnr_status, "FAILED")
        self.assertEqual(failed.log.errors, ("broken",))
        self.assertEqual(failed.timing_status, "PASS")

    def test_path_preserves_all_sources_in_original_order(self):
        path = make_snapshot().critical_paths[0]

        self.assertEqual(path.origin, "core.reg0.Q (0, 0)")
        self.assertEqual(path.destination, "core.reg8.D (8, 0)")
        self.assertAlmostEqual(path.total_delay_ns, 2.107)
        self.assertEqual(path.hdl_sources, SOURCE_LOCATIONS)
        self.assertEqual(path.delay_by_type[0][0], "logic")

    def test_sources_are_opaque_ordered_unique_values(self):
        values = ("alpha", "/beta", "C:\\gamma", "relative/delta")
        report = {
            "critical_paths": [
                {
                    "path": [
                        {
                            "sources": [
                                values[0],
                                values[1],
                                values[0],
                                values[2],
                                values[3],
                            ]
                        }
                    ]
                }
            ]
        }

        snapshot = make_snapshot(report)

        self.assertEqual(snapshot.critical_paths[0].hdl_sources, values)

    def test_resource_selection_is_an_explicit_display_policy(self):
        resources = make_snapshot().resources
        default = select_resources(resources, include_all=False)

        self.assertEqual(
            [item.name for item in default],
            ["ALU", "LUT4", "DFF", "IOB", "BSRAM"],
        )
        self.assertEqual(len(select_resources(resources, include_all=True)), 8)
        self.assertAlmostEqual(default[0].percent, 0.185185, places=5)
        self.assertEqual(default[-1].percent, 0)


class RenderingTests(unittest.TestCase):
    def test_complete_plain_report_matches_golden(self):
        self.assertEqual(
            normalize(render(tiny_snapshot(), width=72)), EXPECTED_TINY
        )

    def test_every_section_title_is_bold(self):
        output = render(color=True, width=110)
        titles = (
            "FPGA Place & Route",
            "Timing",
            "Resource Utilization",
            "Critical Path 1",
            "8 segments — 5 slowest shown",
            "Warnings (3)",
        )

        for title in titles:
            with self.subTest(title=title):
                self.assertTrue(has_bold(title_line(title, output)))

    def test_header_and_total_delay_values_are_bold(self):
        output = render(color=True, width=110)

        for label, value in (
            ("Board", BOARD),
            ("Device", DEVICE),
            ("P&R status", "SUCCESS"),
            ("Timing status", "PASS"),
            ("Total delay", "2.107 ns"),
        ):
            with self.subTest(label=label):
                line = line_with(label, output)
                self.assertTrue(has_bold(line))
                self.assertIn(value, line)

    def test_slowest_segments_are_ordered_and_limited(self):
        output = render()
        positions = [
            output.index(f"core.n{index}") for index in (7, 3, 5, 1, 2)
        ]

        self.assertEqual(positions, sorted(positions))
        self.assertIn("8 segments — 5 slowest shown", output)
        for index in (4, 6, 8):
            self.assertNotIn(f"core.n{index}", output)

    def test_all_resources_option_and_hidden_hint(self):
        default = render()
        complete = render(include_all_resources=True)

        self.assertIn("BSRAM", default)
        self.assertNotIn("GND", default)
        self.assertIn("--all-resources", default)
        self.assertIn("GND", complete)
        self.assertIn("MUX2_LUT7", complete)
        self.assertNotIn("--all-resources", complete)

    def test_only_special_resources_still_offers_complete_view(self):
        report = {"utilization": {"GND": {"available": 1, "used": 1}}}

        default = render(make_snapshot(report))
        complete = render(make_snapshot(report), include_all_resources=True)

        self.assertNotIn("No utilization data", default)
        self.assertIn("--all-resources", default)
        self.assertIn("GND", complete)

    def test_all_hdl_sources_are_shown_without_path_heuristics(self):
        output = render(width=120)

        for source in SOURCE_LOCATIONS:
            self.assertIn(source, output)

    def test_long_identifiers_ellipsize_and_prose_wraps(self):
        report = report_dict()
        long_name = "core." + "generated_name_" * 20
        report["critical_paths"][0]["path"][0]["from"]["cell"] = long_name
        for segment in report["critical_paths"][0]["path"]:
            segment["net"] = long_name
            segment["sources"] = [
                SOURCE_LOCATIONS[1],
                SOURCE_LOCATIONS[1] + "-second",
            ]

        output = render(make_snapshot(report), width=80)

        self.assertTrue(all(len(line) <= 80 for line in output.splitlines()))
        self.assertIn("…", output)
        self.assertNotIn(long_name, output)
        self.assertIn(SOURCE_LOCATIONS[1], " ".join(output.split()))

    def test_missing_data_is_explicit_not_a_fake_pass(self):
        output = render(make_snapshot({}, log_text=None))

        for message in (
            "No timing data",
            "No utilization data",
            "No critical paths",
            "No log file given",
        ):
            self.assertIn(message, output)
        self.assertNotIn("PASS", output)
        self.assertIn("UNKNOWN", output)

    def test_errors_have_their_own_panel_and_status(self):
        output = render(
            make_snapshot(log_text="ERROR: broken\n0 warnings, 1 error\n")
        )

        self.assertIn("Errors (1)", output)
        self.assertIn("broken", output)
        self.assertIn("FAILED", line_with("P&R status", output))

    def test_non_terminal_output_has_no_ansi(self):
        output = render()

        self.assertNotIn("\x1b[", output)
        self.assertEqual(output, render())


class CliTests(unittest.TestCase):
    def write_inputs(self, directory):
        report_path = Path(directory) / "report.json"
        log_path = Path(directory) / "pnr.log"
        report_path.write_text(json.dumps(report_dict()), encoding="utf-8")
        log_path.write_text(SAMPLE_LOG, encoding="utf-8")
        return report_path, log_path

    def run_cli(self, *args):
        stdout, stderr = io.StringIO(), io.StringIO()
        with (
            contextlib.redirect_stdout(stdout),
            contextlib.redirect_stderr(stderr),
        ):
            code = main([*args, "--width", "100"])
        return code, stdout.getvalue(), stderr.getvalue()

    def test_success_renders_metadata_without_derived_files(self):
        with tempfile.TemporaryDirectory() as directory:
            report_path, log_path = self.write_inputs(directory)

            code, stdout, stderr = self.run_cli(
                "--report",
                str(report_path),
                "--log",
                str(log_path),
                "--board",
                BOARD,
                "--device",
                DEVICE,
            )

            self.assertEqual(code, 0, stderr)
            self.assertEqual(stderr, "")
            self.assertEqual(
                sorted(os.listdir(directory)), ["pnr.log", "report.json"]
            )
            for value in (BOARD, DEVICE, "SUCCESS", "PASS", str(report_path)):
                self.assertIn(value, stdout)

    def test_all_resources_and_segment_limit_are_forwarded(self):
        with tempfile.TemporaryDirectory() as directory:
            report_path, log_path = self.write_inputs(directory)

            _, all_output, _ = self.run_cli(
                "--report",
                str(report_path),
                "--log",
                str(log_path),
                "--all-resources",
            )
            _, zero_output, _ = self.run_cli(
                "--report",
                str(report_path),
                "--log",
                str(log_path),
                "--max-segments",
                "0",
            )

            self.assertIn("GND", all_output)
            self.assertIn("8 segments — 0 slowest shown", zero_output)
            self.assertNotIn("core.n1", zero_output)

    def test_missing_report_is_a_clean_error(self):
        code, stdout, stderr = self.run_cli(
            "--report", "/nonexistent/report.json"
        )

        self.assertEqual(code, 2)
        self.assertEqual(stdout, "")
        self.assertIn("cannot read", stderr)

    def test_missing_log_is_a_clean_error(self):
        with tempfile.TemporaryDirectory() as directory:
            report_path, _ = self.write_inputs(directory)

            code, stdout, stderr = self.run_cli(
                "--report", str(report_path), "--log", "/nonexistent/pnr.log"
            )

            self.assertEqual(code, 2)
            self.assertEqual(stdout, "")
            self.assertIn("cannot read", stderr)
            self.assertIn("pnr.log", stderr)

    def test_invalid_json_and_non_object_json_are_rejected(self):
        for content, message in (
            ("{bad", "not valid JSON"),
            ("[]", "JSON object"),
        ):
            with (
                self.subTest(content=content),
                tempfile.TemporaryDirectory() as directory,
            ):
                report_path = Path(directory) / "report.json"
                report_path.write_text(content, encoding="utf-8")

                code, stdout, stderr = self.run_cli(
                    "--report", str(report_path)
                )

                self.assertEqual(code, 2)
                self.assertEqual(stdout, "")
                self.assertIn(message, stderr)


if __name__ == "__main__":
    unittest.main()
