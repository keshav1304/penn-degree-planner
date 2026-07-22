import { toJpeg } from "html-to-image";

/**
 * Capture the schedule grid as a JPEG and trigger download.
 * @param {HTMLElement} element - .schedule-container root
 * @param {{ filename?: string, quality?: number }} [options]
 */
export async function exportScheduleJpeg(element, options = {}) {
  if (!element) throw new Error("Schedule element not found");

  const filename = options.filename || "degree-plan.jpg";
  const quality = options.quality ?? 0.92;
  const wasCollapsed = element.classList.contains("credits-collapsed");

  element.classList.add("exporting");
  element.classList.remove("credits-collapsed");

  try {
    // Allow layout to settle after expanding credits / hiding chrome
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));

    const dataUrl = await toJpeg(element, {
      quality,
      pixelRatio: 2,
      backgroundColor: "#ffffff",
      cacheBust: true,
      filter: (node) => {
        if (!(node instanceof Element)) return true;
        if (node.classList?.contains("export-hide")) return false;
        return true;
      },
    });

    const link = document.createElement("a");
    link.download = filename;
    link.href = dataUrl;
    link.click();
  } finally {
    element.classList.remove("exporting");
    if (wasCollapsed) element.classList.add("credits-collapsed");
  }
}
