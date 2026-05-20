/** Penn course code: DEPT + space + number (e.g. "STAT 4300"). */
export function isValidCourseCode(code) {
  if (!code || typeof code !== "string") return false;
  const trimmed = code.trim();
  const space = trimmed.indexOf(" ");
  if (space <= 0 || space === trimmed.length - 1) return false;
  const prefix = trimmed.slice(0, space);
  const suffix = trimmed.slice(space + 1);
  return /^[A-Za-z]+$/.test(prefix) && /^\d+$/.test(suffix);
}

/** Stable schedule placeholder for an open requirement (not a course). */
export function isRequirementSlotId(id) {
  return typeof id === "string" && id.startsWith("req:");
}

/** Item that may appear on the schedule grid (course or requirement slot). */
export function isSchedulePlacementId(id) {
  return isValidCourseCode(id) || isRequirementSlotId(id);
}

/** Schedule grid status: suggested (white), frozen (orange), taken (green). */
export const SCHEDULE_STATUS = {
  SUGGESTED: "suggested",
  FROZEN: "frozen",
  TAKEN: "taken",
};

export function getScheduleCourseStatus(courseId, { assignedCourses = [], frozenCourses = [] }) {
  if (assignedCourses.some((a) => a.courseId === courseId)) {
    return SCHEDULE_STATUS.TAKEN;
  }
  if (frozenCourses.some((f) => f.courseId === courseId)) {
    return SCHEDULE_STATUS.FROZEN;
  }
  return SCHEDULE_STATUS.SUGGESTED;
}

export function filterValidCourseCodes(codes) {
  return (codes || []).filter(isValidCourseCode);
}

/** Green / assigned placements — courses only. */
export function filterValidPlacements(placements) {
  return (placements || []).filter((p) => isValidCourseCode(p.courseId));
}

/** Orange / frozen — real courses and requirement slots. */
export function filterFrozenPlacements(placements) {
  return (placements || []).filter((p) => isSchedulePlacementId(p.courseId));
}
