import { toJpeg } from "html-to-image";
import { buildScheduleDisplay } from "@/lib/scheduleDisplay";
import { buildDegreeColorMap } from "@/lib/degreeColors";
import { formatDegreeApiLabel, catalogForProgram } from "@/lib/degreeDisplay";

/** Fixed landscape canvas — same aspect ratio for every export. */
export const JPEG_EXPORT_WIDTH = 1680;
export const JPEG_EXPORT_HEIGHT = 1050; // 16:10

const STATUS_COLORS = {
  taken: { bg: "#dcfce7", border: "#86efac", text: "#166534" },
  frozen: { bg: "#ffedd5", border: "#fdba74", text: "#9a3412" },
  suggested: { bg: "#f8fafc", border: "#cbd5e1", text: "#0f172a" },
  slot: { bg: "#f1f5f9", border: "#94a3b8", text: "#334155" },
};

const EXPORT_STYLE_ID = "schedule-jpeg-export-styles";

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function ensureExportStyles() {
  let style = document.getElementById(EXPORT_STYLE_ID);
  if (style) return style;
  style = document.createElement("style");
  style.id = EXPORT_STYLE_ID;
  style.textContent = `
    .schedule-jpeg-root {
      box-sizing: border-box;
      width: ${JPEG_EXPORT_WIDTH}px;
      height: ${JPEG_EXPORT_HEIGHT}px;
      padding: 28px 32px 24px;
      background: #ffffff;
      color: #0f172a;
      font-family: "Segoe UI", "Helvetica Neue", Helvetica, Arial, sans-serif;
      display: flex;
      flex-direction: column;
      gap: 14px;
      overflow: hidden;
    }
    .schedule-jpeg-root *, .schedule-jpeg-root *::before, .schedule-jpeg-root *::after {
      box-sizing: border-box;
    }
    .schedule-jpeg-root .sj-header {
      flex: 0 0 auto;
      display: flex;
      flex-direction: column;
      gap: 6px;
      border-bottom: 2px solid #1f4e79;
      padding-bottom: 12px;
    }
    .schedule-jpeg-root .sj-title-row {
      display: flex;
      align-items: baseline;
      justify-content: space-between;
      gap: 16px;
    }
    .schedule-jpeg-root .sj-title {
      margin: 0;
      font-size: 26px;
      font-weight: 800;
      letter-spacing: -0.02em;
      color: #1f4e79;
    }
    .schedule-jpeg-root .sj-degrees {
      margin: 0;
      font-size: 14px;
      font-weight: 600;
      color: #334155;
      text-align: right;
      max-width: 55%;
    }
    .schedule-jpeg-root .sj-meta {
      display: flex;
      gap: 16px;
      flex-wrap: wrap;
      font-size: 12px;
      color: #64748b;
    }
    .schedule-jpeg-root .sj-meta strong { color: #334155; font-weight: 700; }
    .schedule-jpeg-root .sj-board {
      flex: 1 1 auto;
      min-height: 0;
      display: grid;
      gap: 12px;
    }
    .schedule-jpeg-root .sj-year {
      min-width: 0;
      min-height: 0;
      display: flex;
      flex-direction: column;
      border: 1px solid #cbd5e1;
      border-radius: 10px;
      background: #f8fafc;
      overflow: hidden;
    }
    .schedule-jpeg-root .sj-year-head {
      flex: 0 0 auto;
      padding: 8px 10px;
      font-size: 13px;
      font-weight: 800;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      color: #ffffff;
      background: #1f4e79;
      text-align: center;
    }
    .schedule-jpeg-root .sj-sems {
      flex: 1 1 auto;
      min-height: 0;
      display: grid;
      gap: 1px;
      background: #cbd5e1;
    }
    .schedule-jpeg-root .sj-sem {
      min-width: 0;
      min-height: 0;
      background: #ffffff;
      display: flex;
      flex-direction: column;
    }
    .schedule-jpeg-root .sj-sem-head {
      flex: 0 0 auto;
      padding: 6px 8px;
      font-size: 11px;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.04em;
      color: #475569;
      background: #e2e8f0;
      text-align: center;
      border-bottom: 1px solid #cbd5e1;
    }
    .schedule-jpeg-root .sj-sem-body {
      flex: 1 1 auto;
      min-height: 0;
      overflow: hidden;
      padding: 6px;
      display: flex;
      flex-direction: column;
      gap: var(--sj-chip-gap, 4px);
    }
    .schedule-jpeg-root .sj-chip {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 6px;
      padding: var(--sj-chip-pad, 4px 6px);
      border: 1px solid;
      border-radius: 5px;
      font-size: var(--sj-chip-size, 11px);
      line-height: 1.2;
      font-weight: 600;
      flex: 0 0 auto;
    }
    .schedule-jpeg-root .sj-chip-label {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .schedule-jpeg-root .sj-chip-cu {
      flex: 0 0 auto;
      font-size: 0.9em;
      opacity: 0.85;
      font-variant-numeric: tabular-nums;
    }
    .schedule-jpeg-root .sj-empty {
      color: #94a3b8;
      font-size: 12px;
      text-align: center;
      padding: 8px 0;
    }
    .schedule-jpeg-root .sj-sem-foot {
      flex: 0 0 auto;
      padding: 5px 6px;
      font-size: 10px;
      font-weight: 700;
      color: #9a3412;
      background: #ffedd5;
      text-align: center;
      border-top: 1px solid #fdba74;
    }
    .schedule-jpeg-root .sj-footer {
      flex: 0 0 auto;
      display: flex;
      flex-direction: column;
      gap: 8px;
      border-top: 1px solid #e2e8f0;
      padding-top: 10px;
    }
    .schedule-jpeg-root .sj-legend {
      display: flex;
      flex-wrap: wrap;
      gap: 12px 16px;
      align-items: center;
      font-size: 11px;
      color: #475569;
    }
    .schedule-jpeg-root .sj-legend-item {
      display: inline-flex;
      align-items: center;
      gap: 6px;
    }
    .schedule-jpeg-root .sj-swatch {
      width: 12px;
      height: 12px;
      border-radius: 2px;
      display: inline-block;
    }
    .schedule-jpeg-root .sj-chip-sample {
      width: 14px;
      height: 10px;
      border-radius: 2px;
      border: 1px solid;
      display: inline-block;
    }
    .schedule-jpeg-root .sj-chip-sample.taken { background: #dcfce7; border-color: #86efac; }
    .schedule-jpeg-root .sj-chip-sample.frozen { background: #ffedd5; border-color: #fdba74; }
    .schedule-jpeg-root .sj-chip-sample.suggested { background: #f8fafc; border-color: #cbd5e1; }
    .schedule-jpeg-root .sj-conc-row {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      font-size: 11px;
      color: #334155;
    }
    .schedule-jpeg-root .sj-conc {
      padding: 3px 8px;
      border-radius: 999px;
      background: #eff6ff;
      border: 1px solid #bfdbfe;
      font-weight: 600;
    }
  `;
  document.head.appendChild(style);
  return style;
}

function degreeLabelsFromContext({
  scheduleData,
  degrees,
  degreeCatalog,
  minorCatalog,
}) {
  return (scheduleData?.degree_results || []).map((result, i) =>
    formatDegreeApiLabel(
      result.school,
      result.major,
      catalogForProgram(degrees?.[i], result, degreeCatalog, minorCatalog),
    ),
  );
}

function densityVars(maxItems) {
  if (maxItems <= 6) {
    return { chipSize: "11px", chipPad: "4px 6px", chipGap: "4px" };
  }
  if (maxItems <= 8) {
    return { chipSize: "10px", chipPad: "3px 5px", chipGap: "3px" };
  }
  if (maxItems <= 10) {
    return { chipSize: "9px", chipPad: "2px 4px", chipGap: "2px" };
  }
  return { chipSize: "8px", chipPad: "2px 3px", chipGap: "2px" };
}

/**
 * Build a fixed-size offscreen landscape board for JPEG capture.
 * Years sit side-by-side so the image stays wide, never page-tall.
 */
export function buildScheduleJpegRoot(ctx) {
  const display = buildScheduleDisplay(ctx);
  const degreeLabels = degreeLabelsFromContext(ctx);
  const colorMap = buildDegreeColorMap(ctx.scheduleData);
  const years = display.visibleYears;
  const sems = display.visibleSemesters;
  const yearCount = Math.max(years.length, 1);

  let maxItems = 0;
  for (const year of years) {
    for (const sem of sems) {
      maxItems = Math.max(maxItems, display.getSemesterItems(year, sem).items.length);
    }
  }
  const density = densityVars(maxItems);

  const creditsLine = display.creditsCourses.length
    ? display.creditsCourses.map((a) => a.courseId).join(" · ")
    : "";

  const legendItems = Object.entries(colorMap).map(([key, color]) => {
    const idx = (ctx.scheduleData?.degree_results || []).findIndex(
      (r) => `${r.school}-${r.major}` === key,
    );
    const label = (idx >= 0 && degreeLabels[idx]) || key;
    return `<span class="sj-legend-item"><span class="sj-swatch" style="background:${escapeHtml(color)}"></span>${escapeHtml(label)}</span>`;
  });

  legendItems.push(
    `<span class="sj-legend-item"><span class="sj-chip-sample taken"></span>Taken</span>`,
    `<span class="sj-legend-item"><span class="sj-chip-sample frozen"></span>Frozen</span>`,
    `<span class="sj-legend-item"><span class="sj-chip-sample suggested"></span>Suggested</span>`,
  );

  const conc = (ctx.concentrationData || []).slice(0, 4).map((ci) => {
    const done = ci.requirements_fulfilled || 0;
    const total = ci.requirements_total || 0;
    return `<span class="sj-conc">${escapeHtml(ci.name)} ${done}/${total}</span>`;
  }).join("");

  const yearColumns = years.map((year) => {
    const semCols = sems.map((sem) => {
      const { items, actualCu, limitCu } = display.getSemesterItems(year, sem);
      const chips = items.map((item) => {
        const status = item.kind === "slot" || item.kind === "overlap"
          ? (item.status === "frozen" ? "frozen" : "slot")
          : item.status;
        const colors = STATUS_COLORS[status] || STATUS_COLORS.suggested;
        const label = item.label.length > 36 ? `${item.label.slice(0, 34)}…` : item.label;
        return `<div class="sj-chip" style="background:${colors.bg};border-color:${colors.border};color:${colors.text}"><span class="sj-chip-label">${escapeHtml(label)}</span><span class="sj-chip-cu">${item.cu.toFixed(1)}</span></div>`;
      }).join("");

      return `<div class="sj-sem">
        <div class="sj-sem-head">${escapeHtml(sem)}</div>
        <div class="sj-sem-body">${chips || '<div class="sj-empty">—</div>'}</div>
        <div class="sj-sem-foot">${actualCu.toFixed(1)} / ${limitCu} CU</div>
      </div>`;
    }).join("");

    return `<section class="sj-year">
      <header class="sj-year-head">Year ${year}</header>
      <div class="sj-sems" style="grid-template-columns:repeat(${sems.length},minmax(0,1fr))">${semCols}</div>
    </section>`;
  }).join("");

  const root = document.createElement("div");
  root.className = "schedule-jpeg-root";
  root.setAttribute("aria-hidden", "true");
  root.style.setProperty("--sj-chip-size", density.chipSize);
  root.style.setProperty("--sj-chip-pad", density.chipPad);
  root.style.setProperty("--sj-chip-gap", density.chipGap);
  root.innerHTML = `
    <header class="sj-header">
      <div class="sj-title-row">
        <h1 class="sj-title">Penn Degree Planner</h1>
        <p class="sj-degrees">${escapeHtml(degreeLabels.join(" · ") || "Degree plan")}</p>
      </div>
      <div class="sj-meta">
        ${creditsLine ? `<span><strong>Credits received</strong> ${escapeHtml(creditsLine)}</span>` : ""}
        <span><strong>Terms</strong> ${escapeHtml(sems.join(", "))}</span>
      </div>
    </header>
    <div class="sj-board" style="grid-template-columns:repeat(${yearCount},minmax(0,1fr))">${yearColumns || '<div class="sj-empty">No schedule years</div>'}</div>
    <footer class="sj-footer">
      <div class="sj-legend">${legendItems.join("")}</div>
      ${conc ? `<div class="sj-conc-row">${conc}</div>` : ""}
    </footer>
  `;

  return root;
}

/**
 * Build a standardized landscape JPEG of the schedule and download it.
 * Does not screenshot the live UI — uses a dedicated fixed-aspect export board.
 */
export async function exportScheduleJpeg(ctx, options = {}) {
  const filename = options.filename || "degree-plan.jpg";
  const quality = options.quality ?? 0.92;

  ensureExportStyles();
  const root = buildScheduleJpegRoot(ctx);
  Object.assign(root.style, {
    position: "fixed",
    left: "-10000px",
    top: "0",
    zIndex: "-1",
    pointerEvents: "none",
  });
  document.body.appendChild(root);

  try {
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));

    const dataUrl = await toJpeg(root, {
      quality,
      pixelRatio: 2,
      width: JPEG_EXPORT_WIDTH,
      height: JPEG_EXPORT_HEIGHT,
      canvasWidth: JPEG_EXPORT_WIDTH * 2,
      canvasHeight: JPEG_EXPORT_HEIGHT * 2,
      backgroundColor: "#ffffff",
      cacheBust: true,
    });

    const link = document.createElement("a");
    link.download = filename;
    link.href = dataUrl;
    link.click();
  } finally {
    root.remove();
  }
}
