#!/usr/bin/env python3
import csv
import sys
from pathlib import Path

# --- Configuration ---
COL_DEPT     = "dept_code"
COL_CODE     = "course_code"
COL_TITLE    = "title"
COL_DESC     = "description"
COL_SEMESTER = "semester"
COL_PREREQ   = "prereq"
COL_CU       = "cu"
COL_ALSO     = "also_offered_as"
COL_MUTEX    = "mutually_exclusive"
COL_COREQ    = "coreq"

CHUNK_SIZE   = 200  # Number of courses per sub-function to prevent compiler strain

def rs_escape(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"')

def rs_string(s: str) -> str:
    return f'"{rs_escape(s)}".to_string()'

def rs_opt_string(s: str) -> str:
    s = s.strip()
    return f'Some("{rs_escape(s)}".to_string())' if s else "None"

def emit_course_push(row: dict) -> str:
    """Generates a single v.push(...) statement."""
    return f"""    v.push(Course {{
        dept_code: {rs_string(row.get(COL_DEPT, "").strip())},
        course_code: {rs_string(row.get(COL_CODE, "").strip())},
        title: {rs_string(row.get(COL_TITLE, "").strip())},
        description: {rs_opt_string(row.get(COL_DESC, "").strip())},
        semester: {rs_opt_string(row.get(COL_SEMESTER, "").strip())},
        prereq: {rs_opt_string(row.get(COL_PREREQ, "").strip())},
        cu: {rs_string(row.get(COL_CU, "").strip() or "1")},
        also_offered_as: {rs_opt_string(row.get(COL_ALSO, "").strip())},
        mutually_exclusive: {rs_opt_string(row.get(COL_MUTEX, "").strip())},
        coreq: {rs_opt_string(row.get(COL_COREQ, "").strip())},
    }});"""

def transpile(input_path: Path, output_path: Path):
    if not input_path.exists():
        print(f"Error: {input_path} not found.")
        return

    with input_path.open(newline="", encoding="utf-8-sig") as f:
        rows = list(csv.DictReader(f))

    chunks = [rows[i:i + CHUNK_SIZE] for i in range(0, len(rows), CHUNK_SIZE)]
    
    with output_path.open("w", encoding="utf-8") as out:
        out.write("// @generated - do not edit\n")
        out.write("use super::Course;\n\n")

        # Generate sub-functions for each chunk
        chunk_names = []
        for i, chunk in enumerate(chunks):
            func_name = f"load_chunk_{i}"
            chunk_names.append(func_name)
            out.write(f"fn {func_name}(v: &mut Vec<Course>) {{\n")
            for row in chunk:
                out.write(emit_course_push(row) + "\n")
            out.write("}\n\n")

        # Main entry point
        out.write("/// Returns all courses. Uses heap allocation to avoid stack overflow.\n")
        out.write("pub fn all_courses() -> Vec<Course> {\n")
        out.write(f"    let mut v = Vec::with_capacity({len(rows)});\n")
        for name in chunk_names:
            out.write(f"    {name}(&mut v);\n")
        out.write("    v\n")
        out.write("}\n")

    print(f"Successfully transpiled {len(rows)} courses to {output_path}")

if __name__ == "__main__":
    # Usage: python transpile.py [input.csv] [output.rs]
    src = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("all_courses.csv")
    dst = Path(sys.argv[2]) if len(sys.argv) > 2 else Path("src/courses_data.rs")
    transpile(src, dst)