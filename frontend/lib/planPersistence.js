/**
 * Browser persistence for named plans and the last matching `/generate_schedule`
 * response for each plan. The cached schedule is keyed to the inputs that
 * produced it so a mid-edit reload does not paint a stale grid.
 *
 * Storage is one library (`activeId` + `schedules[]`). An older single-plan
 * blob is wrapped as "Schedule 1" on load. The separate schedule cache key is
 * only read during that migration.
 */
import {
  filterFrozenPlacements,
  filterValidCourseCodes,
  filterValidPlacements,
} from "@/lib/courseUtils";
import { gapSemesterKeys } from "@/lib/semesterOptions";

export const STORAGE_KEY = "penn_degree_planner_state";
export const SCHEDULE_CACHE_KEY = "penn_degree_planner_schedule";
export const CATALOG_CACHE_KEY = "penn_degree_planner_catalogs";
export const MAX_SCHEDULE_NAME_LENGTH = 40;

export function newScheduleId() {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `plan-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
}

export function clampScheduleName(name, fallback = "Schedule") {
  const trimmed = String(name ?? "").trim().replace(/\s+/g, " ").slice(0, MAX_SCHEDULE_NAME_LENGTH);
  return trimmed || fallback;
}

export function emptyPlanInputs() {
  return {
    degrees: [],
    takenCourses: [],
    frozenCourses: [],
    assignedCourses: [],
    allowSummer: false,
    semesterCuLimits: {},
    gapSemesters: {},
  };
}

function cloneValue(value) {
  if (typeof structuredClone === "function") return structuredClone(value);
  return JSON.parse(JSON.stringify(value));
}

function pickInputs(source) {
  const base = emptyPlanInputs();
  if (!source || typeof source !== "object") return base;
  return {
    degrees: Array.isArray(source.degrees) ? source.degrees : base.degrees,
    takenCourses: Array.isArray(source.takenCourses) ? source.takenCourses : base.takenCourses,
    frozenCourses: Array.isArray(source.frozenCourses) ? source.frozenCourses : base.frozenCourses,
    assignedCourses: Array.isArray(source.assignedCourses) ? source.assignedCourses : base.assignedCourses,
    allowSummer: source.allowSummer !== undefined ? Boolean(source.allowSummer) : base.allowSummer,
    semesterCuLimits: source.semesterCuLimits && typeof source.semesterCuLimits === "object"
      ? source.semesterCuLimits
      : base.semesterCuLimits,
    gapSemesters: source.gapSemesters && typeof source.gapSemesters === "object"
      ? source.gapSemesters
      : base.gapSemesters,
  };
}

/** Drop overlap_plan so several named plans fit in localStorage. */
export function slimScheduleData(scheduleData) {
  if (!scheduleData || !Array.isArray(scheduleData.schedule)) return null;
  const rest = { ...scheduleData };
  delete rest.overlap_plan;
  return rest;
}

export function scheduleCacheFrom(inputKey, scheduleData) {
  const slim = slimScheduleData(scheduleData);
  if (!inputKey || !slim) return null;
  return { inputKey, scheduleData: slim };
}

export function cachedScheduleFor(plan, inputKey) {
  const cache = plan?.scheduleCache;
  if (!cache || cache.inputKey !== inputKey) return null;
  if (!Array.isArray(cache.scheduleData?.schedule)) return null;
  return cache.scheduleData;
}

function normalizeCache(cache) {
  if (!cache || typeof cache.inputKey !== "string") return null;
  const slim = slimScheduleData(cache.scheduleData);
  if (!slim) return null;
  return { inputKey: cache.inputKey, scheduleData: slim };
}

export function createPlan(name, inputs = emptyPlanInputs(), scheduleCache = null) {
  return {
    id: newScheduleId(),
    name: clampScheduleName(name, "Schedule 1"),
    updatedAt: Date.now(),
    ...cloneValue(pickInputs(inputs)),
    scheduleCache: normalizeCache(scheduleCache),
  };
}

export function nextScheduleName(schedules) {
  const taken = new Set((schedules || []).map((plan) => plan.name));
  let n = Math.max(1, (schedules || []).length + 1);
  let name = `Schedule ${n}`;
  while (taken.has(name)) {
    n += 1;
    name = `Schedule ${n}`;
  }
  return name;
}

export function duplicateScheduleName(name) {
  const suffix = " copy";
  const stem = clampScheduleName(name, "Schedule").slice(0, MAX_SCHEDULE_NAME_LENGTH - suffix.length);
  return clampScheduleName(`${stem}${suffix}`, "Schedule copy");
}

export function freshPlanLibrary() {
  const plan = createPlan("Schedule 1");
  return { activeId: plan.id, schedules: [plan] };
}

function isLegacyPlan(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  if (Array.isArray(value.schedules)) return false;
  return (
    "degrees" in value
    || "takenCourses" in value
    || "frozenCourses" in value
    || "assignedCourses" in value
  );
}

export function libraryFromLegacy(legacy, legacyCache) {
  const plan = createPlan("Schedule 1", legacy, normalizeCache(legacyCache));
  return { activeId: plan.id, schedules: [plan] };
}

export function normalizeLibrary(raw) {
  if (!raw || !Array.isArray(raw.schedules) || raw.schedules.length === 0) return null;
  const schedules = raw.schedules.map((plan, index) => ({
    id: typeof plan?.id === "string" && plan.id ? plan.id : newScheduleId(),
    name: clampScheduleName(plan?.name, `Schedule ${index + 1}`),
    updatedAt: typeof plan?.updatedAt === "number" ? plan.updatedAt : Date.now(),
    ...cloneValue(pickInputs(plan)),
    scheduleCache: normalizeCache(plan?.scheduleCache),
  }));
  const activeId = schedules.some((plan) => plan.id === raw.activeId)
    ? raw.activeId
    : schedules[0].id;
  return { activeId, schedules };
}

function readLegacyScheduleCache() {
  if (typeof window === "undefined") return null;
  try {
    const raw = localStorage.getItem(SCHEDULE_CACHE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    return normalizeCache(parsed);
  } catch {
    return null;
  }
}

export function loadPlanLibrary() {
  if (typeof window === "undefined") return null;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (Array.isArray(parsed?.schedules)) return normalizeLibrary(parsed);
    if (isLegacyPlan(parsed)) {
      const library = libraryFromLegacy(parsed, readLegacyScheduleCache());
      savePlanLibrary(library);
      return library;
    }
    return null;
  } catch {
    return null;
  }
}

export function savePlanLibrary(library) {
  if (typeof window === "undefined" || !library) return;
  const write = (value) => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
  };
  try {
    write(library);
  } catch {
    try {
      write({
        ...library,
        schedules: (library.schedules || []).map((plan) => ({ ...plan, scheduleCache: null })),
      });
    } catch {
      return;
    }
  }
  try {
    localStorage.removeItem(SCHEDULE_CACHE_KEY);
  } catch { }
}

export function activePlan(library) {
  if (!library) return null;
  return library.schedules.find((plan) => plan.id === library.activeId) || library.schedules[0] || null;
}

/**
 * Update the active plan. `inputs` omitted keeps the stored inputs.
 * `scheduleCache` omitted keeps the stored cache; `null` clears it.
 */
export function upsertActivePlan(library, activeId, inputs, scheduleCache) {
  if (!library) return library;
  const schedules = library.schedules.map((plan) => {
    if (plan.id !== activeId) return plan;
    const next = { ...plan, updatedAt: Date.now() };
    if (inputs) Object.assign(next, cloneValue(pickInputs(inputs)));
    if (scheduleCache !== undefined) next.scheduleCache = normalizeCache(scheduleCache);
    return next;
  });
  return { ...library, schedules };
}

export function addEmptyPlan(library, name) {
  const plan = createPlan(name || nextScheduleName(library?.schedules));
  const schedules = [...(library?.schedules || []), plan];
  return { library: { activeId: plan.id, schedules }, plan };
}

export function duplicatePlan(library, sourceId, name) {
  const source = library?.schedules?.find((plan) => plan.id === sourceId);
  if (!source) return { library, plan: null };
  const plan = createPlan(
    name || duplicateScheduleName(source.name),
    source,
    source.scheduleCache,
  );
  return {
    library: { activeId: plan.id, schedules: [...library.schedules, plan] },
    plan,
  };
}

export function renamePlan(library, id, name) {
  if (!library) return library;
  return {
    ...library,
    schedules: library.schedules.map((plan) => (
      plan.id === id
        ? { ...plan, name: clampScheduleName(name, plan.name), updatedAt: Date.now() }
        : plan
    )),
  };
}

export function deletePlan(library, id) {
  if (!library) return freshPlanLibrary();
  const index = library.schedules.findIndex((plan) => plan.id === id);
  const remaining = library.schedules.filter((plan) => plan.id !== id);
  if (remaining.length === 0) return freshPlanLibrary();
  let activeId = library.activeId;
  if (activeId === id || !remaining.some((plan) => plan.id === activeId)) {
    const neighbor = remaining[Math.min(Math.max(index, 0), remaining.length - 1)];
    activeId = neighbor.id;
  }
  return { activeId, schedules: remaining };
}

export function resetActivePlan(library, activeId) {
  return upsertActivePlan(library, activeId, emptyPlanInputs(), null);
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

export function loadCachedCatalogs() {
  if (typeof window === "undefined") return null;
  try {
    const raw = localStorage.getItem(CATALOG_CACHE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    return {
      degreeCatalog: Array.isArray(parsed?.degreeCatalog) ? parsed.degreeCatalog : [],
      minorCatalog: Array.isArray(parsed?.minorCatalog) ? parsed.minorCatalog : [],
      concentrationCatalog:
        parsed?.concentrationCatalog && typeof parsed.concentrationCatalog === "object"
          ? parsed.concentrationCatalog
          : {},
    };
  } catch {
    return null;
  }
}

export function saveCachedCatalogs(partial) {
  if (typeof window === "undefined" || !partial) return;
  const prev = loadCachedCatalogs() || {
    degreeCatalog: [],
    minorCatalog: [],
    concentrationCatalog: {},
  };
  const next = {
    degreeCatalog: Array.isArray(partial.degreeCatalog) ? partial.degreeCatalog : prev.degreeCatalog,
    minorCatalog: Array.isArray(partial.minorCatalog) ? partial.minorCatalog : prev.minorCatalog,
    concentrationCatalog:
      partial.concentrationCatalog && typeof partial.concentrationCatalog === "object"
        ? partial.concentrationCatalog
        : prev.concentrationCatalog,
  };
  try {
    localStorage.setItem(CATALOG_CACHE_KEY, JSON.stringify(next));
  } catch { }
}
