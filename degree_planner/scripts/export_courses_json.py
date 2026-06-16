#!/usr/bin/env python3
"""One-off converter: courses_data.rs (generated Rust) -> courses.json."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


def parse_rust_string_literal(raw: str) -> str:
    """Decode a Rust double-quoted string body (handles \\ and \")."""
    out: list[str] = []
    i = 0
    while i < len(raw):
        ch = raw[i]
        if ch == "\\" and i + 1 < len(raw):
            nxt = raw[i + 1]
            if nxt == "n":
                out.append("\n")
            elif nxt == "t":
                out.append("\t")
            elif nxt == "r":
                out.append("\r")
            elif nxt == "\\":
                out.append("\\")
            elif nxt == '"':
                out.append('"')
            else:
                out.append(nxt)
            i += 2
            continue
        out.append(ch)
        i += 1
    return "".join(out)


def parse_required_string(block: str, field: str) -> str:
    m = re.search(
        rf'{field}:\s*"((?:\\.|[^"\\])*)"\s*\.to_string\(\)',
        block,
        re.DOTALL,
    )
    if not m:
        raise ValueError(f"missing required field {field}")
    return parse_rust_string_literal(m.group(1))


def parse_optional_string(block: str, field: str) -> str | None:
    if re.search(rf"{field}:\s*None\b", block):
        return None
    m = re.search(rf"{field}:\s*Some\(\"", block)
    if not m:
        raise ValueError(f"missing optional field {field}")
    start = m.end()
    end_marker = '".to_string())'
    end = block.find(end_marker, start)
    if end == -1:
        raise ValueError(f"unterminated Some string for {field}")
    return parse_rust_string_literal(block[start:end])


def parse_cu(block: str) -> float:
    m = re.search(r"cu:\s*([0-9]+(?:\.[0-9]+)?)_f64", block)
    if not m:
        raise ValueError("missing cu field")
    return float(m.group(1))


def parse_course_block(block: str) -> dict:
    return {
        "dept_code": parse_required_string(block, "dept_code"),
        "course_code": parse_required_string(block, "course_code"),
        "title": parse_required_string(block, "title"),
        "description": parse_optional_string(block, "description"),
        "semester": parse_optional_string(block, "semester"),
        "prereq": parse_optional_string(block, "prereq"),
        "cu": parse_cu(block),
        "also_offered_as": parse_optional_string(block, "also_offered_as"),
        "mutually_exclusive": parse_optional_string(block, "mutually_exclusive"),
        "coreq": parse_optional_string(block, "coreq"),
    }


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    src = root / "src" / "penn_data" / "courses_data.rs"
    out = root / "src" / "penn_data" / "courses.json"

    text = src.read_text(encoding="utf-8")
    blocks = re.findall(r"v\.push\(Course \{([\s\S]*?)\}\);", text)
    courses = [parse_course_block(b) for b in blocks]

    out.write_text(json.dumps(courses, ensure_ascii=False), encoding="utf-8")
    print(f"wrote {len(courses)} courses to {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
