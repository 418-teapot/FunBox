"""Render nextpnr's report.json and pnr.log as a terminal summary."""

from __future__ import annotations

import argparse
import json
import math
import re
import sys
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rich import box
from rich.console import Console, Group
from rich.panel import Panel
from rich.table import Table
from rich.text import Text

MISSING = "—"
SPECIAL_RESOURCES = frozenset({"GND", "VCC", "GSR", "PINCFG"})
KEY_RESOURCE_PREFIXES = (
    "LUT",
    "DFF",
    "FF",
    "IOB",
    "MULT",
    "BSRAM",
    "BRAM",
    "SSRAM",
    "DSP",
)


@dataclass(frozen=True)
class ClockResult:
    name: str
    target_mhz: float | None
    achieved_mhz: float | None

    @property
    def slack_ns(self) -> float | None:
        if (
            self.target_mhz is None
            or self.achieved_mhz is None
            or self.target_mhz <= 0
            or self.achieved_mhz <= 0
        ):
            return None
        return 1000.0 / self.target_mhz - 1000.0 / self.achieved_mhz

    @property
    def result(self) -> str:
        slack = self.slack_ns
        if slack is None:
            return "UNKNOWN"
        return "PASS" if slack >= 0 else "FAIL"


@dataclass(frozen=True)
class ResourceUsage:
    name: str
    used: int
    available: int

    @property
    def percent(self) -> float | None:
        if not self.available:
            return None
        return 100.0 * self.used / self.available


@dataclass(frozen=True)
class PathSegment:
    kind: str
    delay_ns: float
    label: str


@dataclass(frozen=True)
class CriticalPath:
    event: str
    origin: str
    destination: str
    total_delay_ns: float
    delay_by_type: tuple[tuple[str, float], ...]
    hdl_sources: tuple[str, ...]
    segments: tuple[PathSegment, ...]


@dataclass(frozen=True)
class LogSummary:
    warnings: tuple[str, ...] = ()
    errors: tuple[str, ...] = ()
    finished_normally: bool = False
    declared_warnings: int = 0
    declared_errors: int = 0

    @property
    def warning_count(self) -> int:
        return max(len(self.warnings), self.declared_warnings)

    @property
    def error_count(self) -> int:
        return max(len(self.errors), self.declared_errors)

    @property
    def status(self) -> str:
        if self.error_count or not self.finished_normally:
            return "FAILED"
        return "SUCCESS"


@dataclass(frozen=True)
class ReportSnapshot:
    """Everything the renderer needs, with no file or terminal dependencies."""

    clocks: tuple[ClockResult, ...]
    resources: tuple[ResourceUsage, ...]
    critical_paths: tuple[CriticalPath, ...]
    log: LogSummary | None
    board: str | None = None
    device: str | None = None
    report_path: str | None = None
    log_path: str | None = None

    @property
    def timing_status(self) -> str:
        results = {clock.result for clock in self.clocks}
        if "FAIL" in results:
            return "FAIL"
        if not results or "UNKNOWN" in results:
            return "UNKNOWN"
        return "PASS"

    @property
    def pnr_status(self) -> str:
        return self.log.status if self.log else "UNKNOWN"


def _number(value: Any) -> float | None:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return None
    number = float(value)
    return number if math.isfinite(number) else None


def _integer(value: Any) -> int:
    number = _number(value)
    return int(number) if number is not None else 0


def _mapping(value: Any) -> Mapping[str, Any]:
    return value if isinstance(value, Mapping) else {}


def _entries(value: Any) -> tuple[Mapping[str, Any], ...]:
    if not isinstance(value, (list, tuple)):
        return ()
    return tuple(item for item in value if isinstance(item, Mapping))


def _pin(raw: Any, *, include_location: bool = False) -> str:
    endpoint = _mapping(raw)
    if not endpoint:
        return MISSING
    label = f"{endpoint.get('cell', '')}.{endpoint.get('port', '')}"
    location = endpoint.get("loc")
    if (
        include_location
        and isinstance(location, (list, tuple))
        and len(location) == 2
    ):
        label += f" ({location[0]}, {location[1]})"
    return label


def _parse_path(raw: Mapping[str, Any]) -> CriticalPath:
    entries = _entries(raw.get("path"))
    delays: dict[str, float] = {}
    sources: dict[str, None] = {}
    segments = []

    for entry in entries:
        kind = str(entry.get("type", ""))
        delay = _number(entry.get("delay")) or 0.0
        delays[kind] = delays.get(kind, 0.0) + delay
        for source in entry.get("sources") or ():
            sources.setdefault(str(source), None)
        label = entry.get("net") or _pin(entry.get("to"))
        segments.append(PathSegment(kind, delay, str(label)))

    origin = (
        _pin(entries[0].get("from"), include_location=True)
        if entries
        else MISSING
    )
    destination = (
        _pin(entries[-1].get("to"), include_location=True)
        if entries
        else MISSING
    )
    return CriticalPath(
        event=f"{raw.get('from', '')} → {raw.get('to', '')}",
        origin=origin,
        destination=destination,
        total_delay_ns=sum(segment.delay_ns for segment in segments),
        delay_by_type=tuple(
            sorted(delays.items(), key=lambda item: item[1], reverse=True)
        ),
        hdl_sources=tuple(sources),
        segments=tuple(segments),
    )


TOTALS_RE = re.compile(r"^\s*(\d+)\s+warnings?,\s*(\d+)\s+errors?\s*$")
WARNING_RE = re.compile(r"^\s*Warning:\s*(.+?)\s*$")
ERROR_RE = re.compile(r"^\s*ERROR:\s*(.+?)\s*$")


def parse_log(text: str) -> LogSummary:
    warnings: list[str] = []
    errors: list[str] = []
    declared_warnings = 0
    declared_errors = 0
    finished_normally = False

    for line in text.splitlines():
        if match := TOTALS_RE.match(line):
            declared_warnings = int(match.group(1))
            declared_errors = int(match.group(2))
        elif match := ERROR_RE.match(line):
            errors.append(match.group(1))
        elif match := WARNING_RE.match(line):
            warnings.append(match.group(1))
        elif "Program finished normally." in line:
            finished_normally = True

    return LogSummary(
        warnings=tuple(warnings),
        errors=tuple(errors),
        finished_normally=finished_normally,
        declared_warnings=declared_warnings,
        declared_errors=declared_errors,
    )


def build_snapshot(
    report: Mapping[str, Any],
    log_text: str | None,
    *,
    board: str | None = None,
    device: str | None = None,
    report_path: str | None = None,
    log_path: str | None = None,
) -> ReportSnapshot:
    """Convert external data into the renderer's immutable input."""
    clocks = tuple(
        ClockResult(
            name=str(name),
            target_mhz=_number(raw.get("constraint")),
            achieved_mhz=_number(raw.get("achieved")),
        )
        for name, raw in _mapping(report.get("fmax")).items()
        if isinstance(raw, Mapping)
    )
    resources = tuple(
        ResourceUsage(
            name=str(name),
            used=_integer(raw.get("used")),
            available=_integer(raw.get("available")),
        )
        for name, raw in _mapping(report.get("utilization")).items()
        if isinstance(raw, Mapping)
    )
    paths = tuple(
        _parse_path(path) for path in _entries(report.get("critical_paths"))
    )
    return ReportSnapshot(
        clocks=clocks,
        resources=resources,
        critical_paths=paths,
        log=parse_log(log_text) if log_text is not None else None,
        board=board,
        device=device,
        report_path=report_path,
        log_path=log_path,
    )


def select_resources(
    resources: tuple[ResourceUsage, ...], *, include_all: bool
) -> tuple[ResourceUsage, ...]:
    if include_all:
        return resources
    return tuple(
        resource
        for resource in resources
        if resource.name.upper() not in SPECIAL_RESOURCES
        and (
            resource.used
            or resource.name.upper().startswith(KEY_RESOURCE_PREFIXES)
        )
    )


VERDICT_STYLES = {
    "PASS": "bold green",
    "FAIL": "bold red",
    "UNKNOWN": "bold yellow",
    "SUCCESS": "bold green",
    "FAILED": "bold red",
}


def _verdict(value: str) -> Text:
    return Text(value, style=VERDICT_STYLES.get(value, "bold"))


def _count(count: int, style: str) -> Text:
    return Text(str(count), style=f"bold {style}" if count else "dim")


def _mhz(value: float | None) -> str:
    return MISSING if value is None else f"{value:.2f} MHz"


def _ns(value: float | None, *, signed: bool = False) -> str:
    if value is None:
        return MISSING
    return f"{value:+.2f} ns" if signed else f"{value:.3f} ns"


def _grid(*, wrap: bool = False) -> Table:
    grid = Table.grid(padding=(0, 2))
    grid.add_column(
        style="dim", no_wrap=True, min_width=13, overflow="ellipsis"
    )
    if wrap:
        grid.add_column()
    else:
        grid.add_column(no_wrap=True, overflow="ellipsis")
    return grid


def _panel(title: str, body: Any, *, border_style: str = "") -> Panel:
    return Panel(
        body,
        title=Text(title, style="bold"),
        title_align="center",
        box=box.ROUNDED,
        border_style=border_style,
        expand=True,
    )


def _table(title: str, *, expand: bool = False) -> Table:
    return Table(
        box=box.ROUNDED,
        title=title,
        title_justify="center",
        title_style="bold",
        header_style="bold",
        expand=expand,
    )


def _fit(value: str, max_width: int) -> Text:
    text = Text(value, no_wrap=True)
    text.truncate(max_width, overflow="ellipsis")
    return text


def _render_header(console: Console, snapshot: ReportSnapshot) -> None:
    pnr = snapshot.pnr_status
    timing = snapshot.timing_status
    grid = _grid()
    if snapshot.board:
        grid.add_row("Board", Text(snapshot.board, style="bold"))
    if snapshot.device:
        grid.add_row("Device", Text(snapshot.device, style="bold"))
    grid.add_row("P&R status", _verdict(pnr))
    grid.add_row("Timing status", _verdict(timing))
    if snapshot.log:
        grid.add_row("Warnings", _count(snapshot.log.warning_count, "yellow"))
        grid.add_row("Errors", _count(snapshot.log.error_count, "red"))

    failed = pnr == "FAILED" or timing == "FAIL"
    unknown = "UNKNOWN" in (pnr, timing)
    border = "red" if failed else "yellow" if unknown else "green"
    console.print(_panel("FPGA Place & Route", grid, border_style=border))


def _render_timing(console: Console, clocks: tuple[ClockResult, ...]) -> None:
    console.print()
    if not clocks:
        console.print(Text("Timing", style="bold"))
        console.print(
            Text("No timing data reported in report.json.", style="dim")
        )
        return

    table = _table("Timing")
    table.add_column("Clock", no_wrap=True, overflow="ellipsis")
    for name in ("Target", "Achieved", "Worst slack"):
        table.add_column(name, justify="right", no_wrap=True)
    table.add_column("Result", no_wrap=True)
    for clock in clocks:
        table.add_row(
            Text(clock.name),
            _mhz(clock.target_mhz),
            _mhz(clock.achieved_mhz),
            _ns(clock.slack_ns, signed=True),
            _verdict(clock.result),
        )
    console.print(table)


def _render_resources(
    console: Console, snapshot: ReportSnapshot, *, include_all: bool
) -> None:
    console.print()
    resources = select_resources(snapshot.resources, include_all=include_all)
    if not snapshot.resources:
        console.print(Text("Resource Utilization", style="bold"))
        console.print(
            Text("No utilization data reported in report.json.", style="dim")
        )
        return

    table = _table("Resource Utilization")
    table.add_column("Resource", no_wrap=True, overflow="ellipsis")
    for name in ("Used", "Available", "Utilization"):
        table.add_column(name, justify="right", no_wrap=True)
    for resource in resources:
        table.add_row(
            Text(resource.name),
            str(resource.used),
            str(resource.available),
            MISSING
            if resource.percent is None
            else f"{resource.percent:.2f}%",
        )
    console.print(table)

    hidden = len(snapshot.resources) - len(resources)
    if hidden:
        console.print(
            Text(
                f"{hidden} resource(s) hidden (unused or device-fixed) · "
                "use --all-resources to list them",
                style="dim",
            )
        )


def _render_path(
    console: Console, index: int, path: CriticalPath, limit: int
) -> None:
    overview = _grid(wrap=True)
    overview.add_row("Clock", Text(path.event))
    overview.add_row(
        "Total delay", Text(_ns(path.total_delay_ns), style="bold")
    )

    endpoints = _grid()
    width = max(16, console.width - 23)
    endpoints.add_row("From", _fit(path.origin, width))
    endpoints.add_row("To", _fit(path.destination, width))

    details = _grid(wrap=True)
    breakdown = " · ".join(
        f"{kind} {_ns(delay)}" for kind, delay in path.delay_by_type
    )
    details.add_row("Delay by type", Text(breakdown or MISSING))
    details.add_row(
        "HDL sources", Text(", ".join(path.hdl_sources) or MISSING)
    )
    console.print()
    console.print(
        _panel(f"Critical Path {index}", Group(overview, endpoints, details))
    )

    if not path.segments:
        return

    slowest = sorted(
        path.segments, key=lambda item: item.delay_ns, reverse=True
    )[:limit]
    title = f"{len(path.segments)} segments"
    if len(slowest) < len(path.segments):
        title += f" — {len(slowest)} slowest shown"
    table = _table(title, expand=True)
    table.add_column("Type", no_wrap=True, width=12)
    table.add_column("Delay", justify="right", no_wrap=True, width=12)
    table.add_column(
        "Net / destination", ratio=1, no_wrap=True, overflow="ellipsis"
    )
    for segment in slowest:
        table.add_row(segment.kind, _ns(segment.delay_ns), Text(segment.label))
    console.print()
    console.print(table)


def _message_panel(
    title: str, messages: tuple[str, ...], count: int, style: str
) -> Panel:
    if messages:
        body = Text(
            "\n".join(f"• {message}" for message in messages), style=style
        )
    else:
        body = Text(
            f"{count} message(s) reported, but no details were found "
            "in the log."
        )
    return _panel(title, body, border_style=style)


def _render_messages(console: Console, log: LogSummary | None) -> None:
    console.print()
    if log is None:
        console.print(
            _panel(
                "Warnings & errors",
                Text("No log file given, cannot report warnings or errors."),
            )
        )
        return
    if not log.warning_count and not log.error_count:
        console.print(
            _panel(
                "Warnings & errors",
                Text("No warnings or errors.", style="dim"),
            )
        )
        return
    if log.warning_count:
        console.print(
            _message_panel(
                f"Warnings ({log.warning_count})",
                log.warnings,
                log.warning_count,
                "yellow",
            )
        )
    if log.error_count:
        console.print(
            _message_panel(
                f"Errors ({log.error_count})",
                log.errors,
                log.error_count,
                "bold red",
            )
        )


def render_report(
    console: Console,
    snapshot: ReportSnapshot,
    *,
    include_all_resources: bool = False,
    max_segments: int = 5,
) -> None:
    """Render without file I/O or knowledge of the external data schema."""
    _render_header(console, snapshot)
    _render_timing(console, snapshot.clocks)
    _render_resources(console, snapshot, include_all=include_all_resources)
    if snapshot.critical_paths:
        for index, path in enumerate(snapshot.critical_paths, start=1):
            _render_path(console, index, path, max_segments)
    else:
        console.print()
        console.print(Text("Critical paths", style="bold"))
        console.print(
            Text("No critical paths reported in report.json.", style="dim")
        )
    _render_messages(console, snapshot.log)
    console.print()
    footer = _grid()
    footer.add_row(
        "report.json", Text(snapshot.report_path or MISSING, style="dim")
    )
    footer.add_row("pnr.log", Text(snapshot.log_path or MISSING, style="dim"))
    console.print(footer)


def _non_negative(value: str) -> int:
    number = int(value)
    if number < 0:
        raise argparse.ArgumentTypeError("must be non-negative")
    return number


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="pnr_report.py",
        description=(
            "Render nextpnr report.json + pnr.log as a Rich terminal report."
        ),
    )
    parser.add_argument(
        "--report", required=True, help="path to nextpnr report.json"
    )
    parser.add_argument("--log", help="path to the nextpnr log file")
    parser.add_argument(
        "--board", help="board name, shown in the report header"
    )
    parser.add_argument(
        "--device", help="FPGA part, shown in the report header"
    )
    parser.add_argument(
        "--all-resources", action="store_true", help="list every resource"
    )
    parser.add_argument(
        "--max-segments",
        type=_non_negative,
        default=5,
        metavar="N",
        help="segments per path",
    )
    parser.add_argument("--width", type=int, help="render width in columns")
    return parser


def _read_report(path: Path) -> Mapping[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise ValueError(f"cannot read {path}: {error}") from error
    except json.JSONDecodeError as error:
        raise ValueError(f"{path} is not valid JSON: {error}") from error
    if not isinstance(data, Mapping):
        raise ValueError(f"{path} must contain a JSON object")
    return data


def main(argv: Sequence[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    report_path = Path(args.report)
    log_path = Path(args.log) if args.log else None
    try:
        report = _read_report(report_path)
        log_text = log_path.read_text(encoding="utf-8") if log_path else None
    except OSError as error:
        print(f"error: cannot read {log_path}: {error}", file=sys.stderr)
        return 2
    except ValueError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2

    snapshot = build_snapshot(
        report,
        log_text,
        board=args.board,
        device=args.device,
        report_path=str(report_path),
        log_path=str(log_path) if log_path else None,
    )
    console = Console(width=args.width) if args.width else Console()
    render_report(
        console,
        snapshot,
        include_all_resources=args.all_resources,
        max_segments=args.max_segments,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
