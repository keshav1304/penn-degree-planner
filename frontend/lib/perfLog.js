/** Dev-only performance logging for bootstrap and popover flows. */
const ENABLED =
    typeof process !== "undefined"
    && process.env.NODE_ENV === "development";

/**
 * @param {string} step
 * @param {number} ms
 * @param {Record<string, unknown>} [detail]
 */
export function perfLog(step, ms, detail) {
    if (!ENABLED) return;
    if (detail) {
        console.info(`[perf] ${step}: ${ms.toFixed(1)}ms`, detail);
    } else {
        console.info(`[perf] ${step}: ${ms.toFixed(1)}ms`);
    }
}

/** @returns {(detail?: Record<string, unknown>) => number} Elapsed ms since mark was created. */
export function perfMark(label) {
    const start = typeof performance !== "undefined" ? performance.now() : Date.now();
    return (detail) => {
        const ms = (typeof performance !== "undefined" ? performance.now() : Date.now()) - start;
        perfLog(label, ms, detail);
        return ms;
    };
}
