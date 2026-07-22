/**
 * Excel export: one sheet with schedule in the center and major requirements on the side(s).
 *
 * Future import (Phase 2 — not implemented):
 * - Prefer JSON round-trip of planner localStorage state (exact restore).
 * - Optional later: parse only the center schedule columns if schema is frozen.
 * - Do not import from JPEG/screenshots (OCR is unreliable).
 */

import ExcelJS from "exceljs";
import { buildScheduleDisplay } from "@/lib/scheduleDisplay";
import { flattenAllDegreeRequirements } from "@/lib/exportRequirementsFlat";
import { formatDegreeApiLabel, catalogForProgram } from "@/lib/degreeDisplay";

const FILL_TAKEN = "C6EFCE";
const FILL_FROZEN = "FCE4D6";
const FILL_HEADER = "1F4E79";
const FILL_SECTION = "D6DCE4";
const FILL_FULFILLED = "E2EFDA";
const FILL_OPEN = "FFF2CC";
const FILL_TOTAL = "F4B183";

const REQ_COLS = 4; // Requirement | Status | CU | Courses
const GAP = 1;

function solidFill(argb) {
  return { type: "pattern", pattern: "solid", fgColor: { argb: `FF${argb}` } };
}

function thinBorder() {
  const edge = { style: "thin", color: { argb: "FFB0B0B0" } };
  return { top: edge, left: edge, bottom: edge, right: edge };
}

/**
 * Place requirements relative to schedule:
 * 1 degree → right only
 * 2 degrees → left + right
 * 3+ → left, right, then further right blocks
 */
function placeRequirementBlocks(degreeBlocks) {
  const n = degreeBlocks.length;
  if (n === 0) return { left: [], right: [] };
  if (n === 1) return { left: [], right: [degreeBlocks[0]] };
  return {
    left: [degreeBlocks[0]],
    right: degreeBlocks.slice(1),
  };
}

function writeRequirementBlock(sheet, startCol, startRow, block) {
  const { title, rows } = block;
  let r = startRow;

  sheet.mergeCells(r, startCol, r, startCol + REQ_COLS - 1);
  const titleCell = sheet.getCell(r, startCol);
  titleCell.value = title;
  titleCell.font = { bold: true, color: { argb: "FFFFFFFF" }, size: 11 };
  titleCell.fill = solidFill(FILL_HEADER);
  titleCell.alignment = { horizontal: "left", vertical: "middle" };
  r += 1;

  const headers = ["Requirement", "Status", "CU", "Courses / suggested"];
  headers.forEach((h, i) => {
    const cell = sheet.getCell(r, startCol + i);
    cell.value = h;
    cell.font = { bold: true, size: 9 };
    cell.fill = solidFill(FILL_SECTION);
    cell.border = thinBorder();
  });
  r += 1;

  for (const row of rows) {
    if (row.kind === "header") {
      sheet.mergeCells(r, startCol, r, startCol + REQ_COLS - 1);
      const cell = sheet.getCell(r, startCol);
      cell.value = row.requirement;
      cell.font = { bold: true, size: 9 };
      cell.fill = solidFill(FILL_SECTION);
      cell.border = thinBorder();
    } else {
      const values = [row.requirement, row.status, row.cu, row.courses];
      values.forEach((v, i) => {
        const cell = sheet.getCell(r, startCol + i);
        cell.value = v;
        cell.font = { size: 9 };
        cell.border = thinBorder();
        cell.alignment = { wrapText: true, vertical: "top" };
        if (i === 1) {
          if (row.status === "Fulfilled") cell.fill = solidFill(FILL_FULFILLED);
          else if (row.status === "Open" || row.status === "Partial") cell.fill = solidFill(FILL_OPEN);
        }
      });
    }
    r += 1;
  }

  return r - startRow;
}

function writeScheduleCenter(sheet, startCol, startRow, ctx) {
  const {
    display,
    degreeLabels,
  } = ctx;
  let r = startRow;
  const sems = display.visibleSemesters;
  // Per semester: label col + CU col
  const scheduleWidth = sems.length * 2;

  sheet.mergeCells(r, startCol, r, startCol + scheduleWidth - 1);
  const title = sheet.getCell(r, startCol);
  title.value = "Schedule";
  title.font = { bold: true, color: { argb: "FFFFFFFF" }, size: 11 };
  title.fill = solidFill(FILL_HEADER);
  title.alignment = { horizontal: "center" };
  r += 1;

  sheet.mergeCells(r, startCol, r, startCol + scheduleWidth - 1);
  sheet.getCell(r, startCol).value = degreeLabels.join(" · ") || "Degree plan";
  sheet.getCell(r, startCol).font = { size: 9, italic: true };
  r += 1;

  // Credits received
  if (display.creditsCourses.length) {
    sheet.mergeCells(r, startCol, r, startCol + scheduleWidth - 1);
    sheet.getCell(r, startCol).value = `Credits Received: ${display.creditsCourses.map((a) => a.courseId).join(", ")}`;
    sheet.getCell(r, startCol).font = { size: 9 };
    sheet.getCell(r, startCol).fill = solidFill(FILL_TAKEN);
    r += 1;
  }

  r += 1; // spacer

  for (const year of display.visibleYears) {
    sheet.mergeCells(r, startCol, r, startCol + scheduleWidth - 1);
    const yearCell = sheet.getCell(r, startCol);
    yearCell.value = `Year ${year}`;
    yearCell.font = { bold: true, size: 10 };
    yearCell.fill = solidFill(FILL_SECTION);
    r += 1;

    // Semester headers
    sems.forEach((sem, i) => {
      const c = startCol + i * 2;
      const h = sheet.getCell(r, c);
      h.value = sem;
      h.font = { bold: true, size: 9 };
      h.fill = solidFill(FILL_HEADER);
      h.font = { bold: true, color: { argb: "FFFFFFFF" }, size: 9 };
      h.border = thinBorder();
      const cuH = sheet.getCell(r, c + 1);
      cuH.value = "CU";
      cuH.font = { bold: true, color: { argb: "FFFFFFFF" }, size: 9 };
      cuH.fill = solidFill(FILL_HEADER);
      cuH.border = thinBorder();
      cuH.alignment = { horizontal: "center" };
    });
    r += 1;

    const termData = sems.map((sem) => display.getSemesterItems(year, sem));
    const maxRows = Math.max(1, ...termData.map((t) => t.items.length));

    for (let i = 0; i < maxRows; i += 1) {
      sems.forEach((_, si) => {
        const c = startCol + si * 2;
        const item = termData[si].items[i];
        const labelCell = sheet.getCell(r, c);
        const cuCell = sheet.getCell(r, c + 1);
        labelCell.border = thinBorder();
        cuCell.border = thinBorder();
        cuCell.alignment = { horizontal: "center" };
        if (item) {
          labelCell.value = item.label;
          labelCell.font = { size: 9 };
          cuCell.value = Number(item.cu.toFixed(1));
          cuCell.font = { size: 9 };
          if (item.status === "taken") {
            labelCell.fill = solidFill(FILL_TAKEN);
            cuCell.fill = solidFill(FILL_TAKEN);
          } else if (item.status === "frozen") {
            labelCell.fill = solidFill(FILL_FROZEN);
            cuCell.fill = solidFill(FILL_FROZEN);
          }
        }
      });
      r += 1;
    }

    // Totals row
    sems.forEach((_, si) => {
      const c = startCol + si * 2;
      const { actualCu, limitCu } = termData[si];
      const labelCell = sheet.getCell(r, c);
      const cuCell = sheet.getCell(r, c + 1);
      labelCell.value = "Total CUs";
      labelCell.font = { bold: true, size: 9 };
      labelCell.fill = solidFill(FILL_TOTAL);
      labelCell.border = thinBorder();
      cuCell.value = `${actualCu.toFixed(1)} / ${limitCu}`;
      cuCell.font = { bold: true, size: 9 };
      cuCell.fill = solidFill(FILL_TOTAL);
      cuCell.border = thinBorder();
      cuCell.alignment = { horizontal: "center" };
    });
    r += 2; // blank between years
  }

  return { height: r - startRow, width: scheduleWidth };
}

/**
 * Build and download the Excel workbook.
 */
export async function exportScheduleExcel({
  scheduleData,
  frozenCourses,
  assignedCourses,
  allowSummer,
  degrees,
  semesterCuLimits,
  courseCuMap,
  requirementSlotLabels,
  degreeCatalog,
  minorCatalog,
  courseDegreesMap,
  concentrationData,
  filename = "penn-degree-plan.xlsx",
}) {
  const display = buildScheduleDisplay({
    scheduleData,
    frozenCourses,
    assignedCourses,
    allowSummer,
    degrees,
    semesterCuLimits,
    courseCuMap,
    requirementSlotLabels,
  });

  const degreeBlocks = flattenAllDegreeRequirements({
    scheduleData,
    degrees,
    degreeCatalog,
    minorCatalog,
    courseDegreesMap,
    concentrationData,
  });

  const degreeLabels = (scheduleData?.degree_results || []).map((result, i) =>
    formatDegreeApiLabel(
      result.school,
      result.major,
      catalogForProgram(degrees[i], result, degreeCatalog, minorCatalog),
    ),
  );

  const { left, right } = placeRequirementBlocks(degreeBlocks);

  const workbook = new ExcelJS.Workbook();
  workbook.creator = "Penn Degree Planner";
  workbook.created = new Date();
  const sheet = workbook.addWorksheet("Degree Plan", {
    views: [{ showGridLines: false }],
  });

  const startRow = 1;
  let scheduleStartCol = 1;

  if (left.length) {
    scheduleStartCol = 1 + left.length * (REQ_COLS + GAP);
  }

  // Write left requirement blocks
  left.forEach((block, i) => {
    const col = 1 + i * (REQ_COLS + GAP);
    writeRequirementBlock(sheet, col, startRow, block);
  });

  const { width: scheduleWidth } = writeScheduleCenter(
    sheet,
    scheduleStartCol,
    startRow,
    { display, degreeLabels },
  );

  // Write right requirement blocks (and any 3+ degrees)
  const rightStartCol = scheduleStartCol + scheduleWidth + GAP;
  right.forEach((block, i) => {
    const col = rightStartCol + i * (REQ_COLS + GAP);
    writeRequirementBlock(sheet, col, startRow, block);
  });

  // Column widths
  const lastCol = Math.max(
    scheduleStartCol + scheduleWidth - 1,
    rightStartCol + Math.max(right.length, 1) * (REQ_COLS + GAP) - GAP - 1,
    left.length ? left.length * (REQ_COLS + GAP) - GAP : 1,
  );
  for (let c = 1; c <= lastCol; c += 1) {
    const inSchedule =
      c >= scheduleStartCol && c < scheduleStartCol + scheduleWidth;
    if (inSchedule) {
      const offset = c - scheduleStartCol;
      sheet.getColumn(c).width = offset % 2 === 0 ? 28 : 8;
    } else {
      const relative = (() => {
        if (left.length && c < scheduleStartCol) {
          const idx = c - 1;
          return idx % (REQ_COLS + GAP);
        }
        const idx = c - rightStartCol;
        return idx % (REQ_COLS + GAP);
      })();
      if (relative === REQ_COLS) {
        sheet.getColumn(c).width = 2;
      } else if (relative === 0) sheet.getColumn(c).width = 36;
      else if (relative === 1) sheet.getColumn(c).width = 11;
      else if (relative === 2) sheet.getColumn(c).width = 6;
      else sheet.getColumn(c).width = 28;
    }
  }

  const buffer = await workbook.xlsx.writeBuffer();
  const blob = new Blob([buffer], {
    type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}
