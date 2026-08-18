/**
 * Browser persistence for plan inputs and the last matching `/generate_schedule`
 * response. The cached schedule is keyed to the inputs that produced it so a
 * mid-edit reload does not paint a stale grid.
 */
import {
  filterFrozenPlacements,
  filterValidCourseCodes,
  filterValidPlacements,
} from "@/lib/courseUtils";
import { gapSemesterKeys } from "@/lib/semesterOptions";

export const STORAGE_KEY = "penn_degree_planner_state";
export const SCHEDULE_CACHE_KEY = "penn_degree_planner_schedule";

export function loadSavedState() {
  if (typeof window === "undefined") return null;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : null;
  } catch {
    return null;
  }
}

export function savePlanState(state) {
  if (typeof window === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      degrees: state.degrees,
      takenCourses: state.takenCourses,
      frozenCourses: state.frozenCourses,
      assignedCourses: state.assignedCourses,
      allowSummer: state.allowSummer,
      semesterCuLimits: state.semesterCuLimits,
      gapSemesters: state.gapSemesters,
    }));
  } catch { }
}

function degreePayload(degree) {
  const concentrations = (degree.concentrations?.length
    ? degree.concentrations
    : degree.concentration
      ? [degree.concentration]
      : []
  ).filter(Boolean);
  concentrations.sort();
  return {
    kind: degree.kind || "major",
    major: degree.majorCode,
    school: degree.schoolCode,
    concentrations,
  };
}

function pinnedPlacements(frozenCourses, assignedCourses) {
  return [
    ...filterFrozenPlacements(frozenCourses),
    ...filterValidPlacements((assignedCourses || []).filter((a) => a.year > 0)),
  ]
    .map((p) => ({
      course_id: p.courseId,
      year: p.year,
      semester: p.semester,
    }))
    .sort((a, b) => {
      const id = String(a.course_id).localeCompare(String(b.course_id));
      if (id !== 0) return id;
      if (a.year !== b.year) return a.year - b.year;
      return String(a.semester).localeCompare(String(b.semester));
    });
}

function sortedRecord(obj) {
  if (!obj || typeof obj !== "object") return {};
  return Object.keys(obj)
    .sort()
    .reduce((acc, key) => {
      acc[key] = obj[key];
      return acc;
    }, {});
}

/** Stable key for the generate inputs that determine the packed schedule. */
export function scheduleInputKey({
  degrees,
  takenCourses,
  frozenCourses,
  assignedCourses,
  allowSummer,
  semesterCuLimits,
  gapSemesters,
}) {
  return JSON.stringify({
    taken: [...filterValidCourseCodes(takenCourses || [])].sort(),
    degrees: (degrees || []).map(degreePayload),
    frozen: pinnedPlacements(frozenCourses, assignedCourses),
    allowSummer: Boolean(allowSummer),
    semesterCuLimits: sortedRecord(semesterCuLimits),
    gapSemesters: [...gapSemesterKeys(gapSemesters)].sort(),
  });
}

export function loadCachedSchedule(inputKey) {
  if (typeof window === "undefined" || !inputKey) return null;
  try {
    const raw = localStorage.getItem(SCHEDULE_CACHE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (parsed?.inputKey !== inputKey || !parsed?.scheduleData) return null;
    if (!Array.isArray(parsed.scheduleData.schedule)) return null;
    return parsed.scheduleData;
  } catch {
    return null;
  }
}

export function saveCachedSchedule(inputKey, scheduleData) {
  if (typeof window === "undefined" || !inputKey || !scheduleData) return;
  const write = (data) => {
    localStorage.setItem(SCHEDULE_CACHE_KEY, JSON.stringify({
      inputKey,
      scheduleData: data,
    }));
  };
  try {
    write(scheduleData);
  } catch {
    try {
      write({ ...scheduleData, overlap_plan: null });
    } catch {
      try {
        localStorage.removeItem(SCHEDULE_CACHE_KEY);
      } catch { }
    }
  }
}

export function clearCachedSchedule() {
  if (typeof window === "undefined") return;
  try {
    localStorage.removeItem(SCHEDULE_CACHE_KEY);
  } catch { }
}

export function clearPlanPersistence() {
  if (typeof window === "undefined") return;
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch { }
  clearCachedSchedule();
}
