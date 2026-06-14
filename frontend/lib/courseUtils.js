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

/** Flexible pool slot placeholders (`1:p0`) — shown on schedule, not in requirements panel. */
export function isPoolFlexibleSlotInstanceId(instanceId) {
  if (!instanceId || typeof instanceId !== "string") return false;
  return instanceId.split(":").some((seg) => /^p\d+$/.test(seg));
}

/** Pool coverage constraints (`1:c0`) — not fixed pool slots (`1:f0:c0`) or flex slots (`1:p0`). */
export function isPoolConstraintInstanceId(instanceId) {
  if (!instanceId || typeof instanceId !== "string") return false;
  const segments = instanceId.split(":");
  if (segments.some((seg) => /^[fp]\d+$/.test(seg))) return false;
  return segments.some((seg) => /^c\d+$/.test(seg));
}

/** Schedule slots scoped to a pool coverage constraint. */
export function isPoolConstraintSlotId(id) {
  if (!isRequirementSlotId(id)) return false;
  const rest = id.slice(4);
  const scope = rest.split(":R:")[0];
  return scope.split(":").some((seg) => /^c\d+$/.test(seg));
}

/** Combined cross-degree requirement block on the schedule (one CU, two requirements). */
export function isOverlapScheduleGroupId(id) {
  return typeof id === "string" && id.startsWith("req:overlap:");
}

/** Requirement slots that represent real schedule CU (pool fixed/flexible / overlap groups). */
export function isSchedulableRequirementSlotId(id) {
  return (
    isOverlapScheduleGroupId(id)
    || (isRequirementSlotId(id) && !isPoolConstraintSlotId(id))
  );
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
