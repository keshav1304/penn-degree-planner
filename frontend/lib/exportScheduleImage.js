import { buildScheduleDisplay } from "@/lib/scheduleDisplay";
import { buildDegreeColorMap } from "@/lib/degreeColors";
import { formatDegreeApiLabel, catalogForProgram } from "@/lib/degreeDisplay";

/** Fixed landscape canvas — same aspect ratio for every export. */
export const JPEG_EXPORT_WIDTH = 1680;
export const JPEG_EXPORT_HEIGHT = 1050; // 16:10

const COLORS = {
  bg: "#ffffff",
  ink: "#0f172a",
  muted: "#64748b",
  soft: "#475569",
  line: "#cbd5e1",
  panel: "#f8fafc",
  head: "#1f4e79",
  semHead: "#e2e8f0",
  footBg: "#ffedd5",
  footBorder: "#fdba74",
  footText: "#9a3412",
  taken: { bg: "#dcfce7", border: "#86efac", text: "#166534" },
  frozen: { bg: "#ffedd5", border: "#fdba74", text: "#9a3412" },
  suggested: { bg: "#f8fafc", border: "#cbd5e1", text: "#0f172a" },
  slot: { bg: "#f1f5f9", border: "#94a3b8", text: "#334155" },
};

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

function roundRect(ctx, x, y, w, h, r) {
  const radius = Math.min(r, w / 2, h / 2);
  ctx.beginPath();
  ctx.moveTo(x + radius, y);
  ctx.arcTo(x + w, y, x + w, y + h, radius);
  ctx.arcTo(x + w, y + h, x, y + h, radius);
  ctx.arcTo(x, y + h, x, y, radius);
  ctx.arcTo(x, y, x + w, y, radius);
  ctx.closePath();
}

function fillRoundRect(ctx, x, y, w, h, r, fill) {
  roundRect(ctx, x, y, w, h, r);
  ctx.fillStyle = fill;
  ctx.fill();
}

function strokeRoundRect(ctx, x, y, w, h, r, stroke, lineWidth = 1) {
  roundRect(ctx, x, y, w, h, r);
  ctx.strokeStyle = stroke;
  ctx.lineWidth = lineWidth;
  ctx.stroke();
}

function truncateToWidth(ctx, text, maxWidth) {
  if (ctx.measureText(text).width <= maxWidth) return text;
  let truncated = text;
  while (truncated.length > 1 && ctx.measureText(`${truncated}…`).width > maxWidth) {
    truncated = truncated.slice(0, -1);
  }
  return `${truncated}…`;
}

function chipStyle(item) {
  if (item.kind === "slot" || item.kind === "overlap") {
    return item.status === "frozen" ? COLORS.frozen : COLORS.slot;
  }
  return COLORS[item.status] || COLORS.suggested;
}

function densityForMaxItems(maxItems) {
  if (maxItems <= 6) return { font: 11, padY: 4, padX: 6, gap: 4, h: 22 };
  if (maxItems <= 8) return { font: 10, padY: 3, padX: 5, gap: 3, h: 19 };
  if (maxItems <= 10) return { font: 9, padY: 2, padX: 4, gap: 2, h: 17 };
  return { font: 8, padY: 2, padX: 3, gap: 2, h: 15 };
}

/**
 * Paint the schedule onto a fixed-size landscape canvas (no DOM screenshot).
 */
export function renderScheduleJpegCanvas(ctxInput, pixelRatio = 2) {
  const display = buildScheduleDisplay(ctxInput);
  const degreeLabels = degreeLabelsFromContext(ctxInput);
  const colorMap = buildDegreeColorMap(ctxInput.scheduleData);
  const years = display.visibleYears;
  const sems = display.visibleSemesters;
  const yearCount = Math.max(years.length, 1);

  let maxItems = 0;
  for (const year of years) {
    for (const sem of sems) {
      maxItems = Math.max(maxItems, display.getSemesterItems(year, sem).items.length);
    }
  }
  const density = densityForMaxItems(maxItems);

  const canvas = document.createElement("canvas");
  canvas.width = JPEG_EXPORT_WIDTH * pixelRatio;
  canvas.height = JPEG_EXPORT_HEIGHT * pixelRatio;
  const ctx = canvas.getContext("2d");
  ctx.scale(pixelRatio, pixelRatio);
  ctx.textBaseline = "middle";

  // Background
  ctx.fillStyle = COLORS.bg;
  ctx.fillRect(0, 0, JPEG_EXPORT_WIDTH, JPEG_EXPORT_HEIGHT);

  const padX = 32;
  const padTop = 28;
  let y = padTop;

  // Header
  ctx.fillStyle = COLORS.head;
  ctx.font = "800 26px 'Segoe UI', 'Helvetica Neue', Helvetica, Arial, sans-serif";
  ctx.textAlign = "left";
  ctx.fillText("Penn Degree Planner", padX, y + 14);

  const degreesText = degreeLabels.join(" · ") || "Degree plan";
  ctx.fillStyle = COLORS.soft;
  ctx.font = "600 14px 'Segoe UI', 'Helvetica Neue', Helvetica, Arial, sans-serif";
  ctx.textAlign = "right";
  ctx.fillText(
    truncateToWidth(ctx, degreesText, JPEG_EXPORT_WIDTH - padX * 2 - 320),
    JPEG_EXPORT_WIDTH - padX,
    y + 16,
  );
  y += 36;

  ctx.textAlign = "left";
  ctx.fillStyle = COLORS.muted;
  ctx.font = "12px 'Segoe UI', 'Helvetica Neue', Helvetica, Arial, sans-serif";
  const metaParts = [];
  if (display.creditsCourses.length) {
    metaParts.push(`Credits received  ${display.creditsCourses.map((a) => a.courseId).join(" · ")}`);
  }
  metaParts.push(`Terms  ${sems.join(", ")}`);
  ctx.fillText(truncateToWidth(ctx, metaParts.join("    "), JPEG_EXPORT_WIDTH - padX * 2), padX, y + 8);
  y += 22;

  ctx.strokeStyle = COLORS.head;
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(padX, y);
  ctx.lineTo(JPEG_EXPORT_WIDTH - padX, y);
  ctx.stroke();
  y += 14;

  // Footer reserved height
  const footerH = 70;
  const boardTop = y;
  const boardBottom = JPEG_EXPORT_HEIGHT - 24 - footerH;
  const boardH = boardBottom - boardTop;
  const boardW = JPEG_EXPORT_WIDTH - padX * 2;
  const yearGap = 12;
  const yearW = (boardW - yearGap * (yearCount - 1)) / yearCount;

  years.forEach((year, yi) => {
    const x0 = padX + yi * (yearW + yearGap);
    fillRoundRect(ctx, x0, boardTop, yearW, boardH, 10, COLORS.panel);
    strokeRoundRect(ctx, x0, boardTop, yearW, boardH, 10, COLORS.line);

    // Year header
    ctx.save();
    roundRect(ctx, x0, boardTop, yearW, 32, 10);
    ctx.clip();
    ctx.fillStyle = COLORS.head;
    ctx.fillRect(x0, boardTop, yearW, 32);
    ctx.restore();
    // square off bottom of header
    ctx.fillStyle = COLORS.head;
    ctx.fillRect(x0, boardTop + 16, yearW, 16);

    ctx.fillStyle = "#ffffff";
    ctx.font = "800 13px 'Segoe UI', 'Helvetica Neue', Helvetica, Arial, sans-serif";
    ctx.textAlign = "center";
    ctx.fillText(`YEAR ${year}`, x0 + yearW / 2, boardTop + 16);

    const semGap = 1;
    const semTop = boardTop + 32;
    const semH = boardH - 32;
    const semW = (yearW - semGap * (sems.length - 1)) / sems.length;
    const headH = 28;
    const footH = 24;
    const bodyTop = semTop + headH;
    const bodyH = semH - headH - footH;

    sems.forEach((sem, si) => {
      const sx = x0 + si * (semW + semGap);
      ctx.fillStyle = "#ffffff";
      ctx.fillRect(sx, semTop, semW, semH);

      ctx.fillStyle = COLORS.semHead;
      ctx.fillRect(sx, semTop, semW, headH);
      ctx.strokeStyle = COLORS.line;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(sx, semTop + headH);
      ctx.lineTo(sx + semW, semTop + headH);
      ctx.stroke();

      ctx.fillStyle = COLORS.soft;
      ctx.font = "700 11px 'Segoe UI', 'Helvetica Neue', Helvetica, Arial, sans-serif";
      ctx.textAlign = "center";
      const gap = display.isGap?.(year, sem);
      ctx.fillText(gap ? `${sem.toUpperCase()} · GAP` : sem.toUpperCase(), sx + semW / 2, semTop + headH / 2);

      const { items, actualCu, limitCu } = display.getSemesterItems(year, sem);
      let cy = bodyTop + 6;
      const chipMaxW = semW - 12;

      items.forEach((item) => {
        if (cy + density.h > bodyTop + bodyH - 4) return;
        const style = chipStyle(item);
        const cx = sx + 6;
        fillRoundRect(ctx, cx, cy, chipMaxW, density.h, 5, style.bg);
        strokeRoundRect(ctx, cx, cy, chipMaxW, density.h, 5, style.border);

        ctx.fillStyle = style.text;
        ctx.font = `600 ${density.font}px 'Segoe UI', 'Helvetica Neue', Helvetica, Arial, sans-serif`;
        ctx.textAlign = "left";
        const cuText = item.cu.toFixed(1);
        const cuW = ctx.measureText(cuText).width + 4;
        const label = truncateToWidth(ctx, item.label, chipMaxW - density.padX * 2 - cuW - 8);
        ctx.fillText(label, cx + density.padX, cy + density.h / 2);
        ctx.textAlign = "right";
        ctx.fillText(cuText, cx + chipMaxW - density.padX, cy + density.h / 2);

        cy += density.h + density.gap;
      });

      if (items.length === 0) {
        ctx.fillStyle = "#94a3b8";
        ctx.font = "12px 'Segoe UI', 'Helvetica Neue', Helvetica, Arial, sans-serif";
        ctx.textAlign = "center";
        ctx.fillText(gap ? "Gap" : "—", sx + semW / 2, bodyTop + bodyH / 2);
      }

      // Footer CU
      ctx.fillStyle = COLORS.footBg;
      ctx.fillRect(sx, semTop + semH - footH, semW, footH);
      ctx.strokeStyle = COLORS.footBorder;
      ctx.beginPath();
      ctx.moveTo(sx, semTop + semH - footH);
      ctx.lineTo(sx + semW, semTop + semH - footH);
      ctx.stroke();
      ctx.fillStyle = COLORS.footText;
      ctx.font = "700 10px 'Segoe UI', 'Helvetica Neue', Helvetica, Arial, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(
        gap ? `Gap · ${actualCu.toFixed(1)} / ${limitCu} CU` : `${actualCu.toFixed(1)} / ${limitCu} CU`,
        sx + semW / 2,
        semTop + semH - footH / 2,
      );
    });
  });

  // Footer legend
  let fy = boardBottom + 16;
  ctx.strokeStyle = "#e2e8f0";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(padX, boardBottom + 6);
  ctx.lineTo(JPEG_EXPORT_WIDTH - padX, boardBottom + 6);
  ctx.stroke();

  ctx.textAlign = "left";
  ctx.font = "11px 'Segoe UI', 'Helvetica Neue', Helvetica, Arial, sans-serif";
  let lx = padX;

  const drawLegendSwatch = (color, label) => {
    ctx.fillStyle = color;
    ctx.fillRect(lx, fy - 5, 12, 12);
    lx += 18;
    ctx.fillStyle = COLORS.soft;
    ctx.fillText(label, lx, fy);
    lx += ctx.measureText(label).width + 16;
  };

  Object.entries(colorMap).forEach(([key, color]) => {
    const idx = (ctxInput.scheduleData?.degree_results || []).findIndex(
      (r) => `${r.school}-${r.major}` === key,
    );
    const label = (idx >= 0 && degreeLabels[idx]) || key;
    drawLegendSwatch(color, label);
  });

  const statusSamples = [
    [COLORS.taken.bg, COLORS.taken.border, "Taken"],
    [COLORS.frozen.bg, COLORS.frozen.border, "Frozen"],
    [COLORS.suggested.bg, COLORS.suggested.border, "Suggested"],
  ];
  statusSamples.forEach(([bg, border, label]) => {
    fillRoundRect(ctx, lx, fy - 5, 14, 10, 2, bg);
    strokeRoundRect(ctx, lx, fy - 5, 14, 10, 2, border);
    lx += 20;
    ctx.fillStyle = COLORS.soft;
    ctx.fillText(label, lx, fy);
    lx += ctx.measureText(label).width + 16;
  });

  const conc = (ctxInput.concentrationData || []).slice(0, 4);
  if (conc.length) {
    fy += 22;
    lx = padX;
    conc.forEach((ci) => {
      const done = ci.requirements_fulfilled || 0;
      const total = ci.requirements_total || 0;
      const text = `${ci.name} ${done}/${total}`;
      const tw = ctx.measureText(text).width + 16;
      fillRoundRect(ctx, lx, fy - 9, tw, 18, 9, "#eff6ff");
      strokeRoundRect(ctx, lx, fy - 9, tw, 18, 9, "#bfdbfe");
      ctx.fillStyle = COLORS.soft;
      ctx.fillText(text, lx + 8, fy);
      lx += tw + 8;
    });
  }

  return canvas;
}

/**
 * Build a standardized landscape JPEG of the schedule and download it.
 * Drawn on canvas (not a DOM screenshot) so capture cannot come out blank.
 */
export async function exportScheduleJpeg(ctxInput, options = {}) {
  const filename = options.filename || "degree-plan.jpg";
  const quality = options.quality ?? 0.92;

  const canvas = renderScheduleJpegCanvas(ctxInput, 2);
  const dataUrl = canvas.toDataURL("image/jpeg", quality);

  if (!dataUrl || dataUrl.length < 1000) {
    throw new Error("Export produced an empty image");
  }

  const link = document.createElement("a");
  link.download = filename;
  link.href = dataUrl;
  link.click();
}
