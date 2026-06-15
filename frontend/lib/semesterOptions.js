const YEAR_LABELS = ["Freshman", "Sophomore", "Junior", "Senior", "Fifth Year", "Sixth Year"];

export const DEFAULT_SEMESTER_CU_LIMIT = 5.5;
export const DUAL_UG_SEMESTER_CU_LIMIT = 6.5;
export const DEFAULT_SUMMER_CU_LIMIT = 2.0;

export function isGraduateSchool(schoolCode) {
    return schoolCode === "SEAS_MS";
}

/** Two or more undergrad degrees with no MS programs. */
export function isDualUndergradOnly(degrees) {
    return (
        degrees?.length >= 2
        && degrees.every((d) => !isGraduateSchool(d.schoolCode))
    );
}

/** Every selected degree is in the College (CAS). */
export function isAllCasCollege(degrees) {
    return degrees?.length > 0 && degrees.every((d) => d.schoolCode === "CAS");
}

/**
 * Default max CU for a semester (before user overrides).
 *
 * - Year 1 Fall: always 5.5
 * - Dual undergrad (not MS), not all-CAS: 6.5 for other fall/spring terms
 * - Dual CAS college majors: 5.5 everywhere
 * - Single degree or any other case: 5.5
 */
export function defaultSemesterCuLimit(semester, year = null, degrees = []) {
    if (semester === "Summer") {
        return DEFAULT_SUMMER_CU_LIMIT;
    }
    if (year === 1 && semester === "Fall") {
        return DEFAULT_SEMESTER_CU_LIMIT;
    }
    if (isDualUndergradOnly(degrees) && !isAllCasCollege(degrees)) {
        return DUAL_UG_SEMESTER_CU_LIMIT;
    }
    return DEFAULT_SEMESTER_CU_LIMIT;
}

/**
 * Effective CU limit for a semester: user override if set, else degree-composition default.
 */
export function resolveSemesterCuLimit(semester, year, degrees = [], userOverrides = {}) {
    const key = `${year}-${semester}`;
    if (userOverrides[key] != null) {
        return userOverrides[key];
    }
    return defaultSemesterCuLimit(semester, year, degrees);
}

/** Full limits map for schedule generation (defaults + overrides for every term). */
export function buildSemesterCuLimitsMap(degrees, maxYear, allowSummer = true, userOverrides = {}) {
    const limits = {};
    const years = Math.max(4, maxYear || 4);
    const semesters = allowSummer ? ["Fall", "Spring", "Summer"] : ["Fall", "Spring"];
    for (let y = 1; y <= years; y++) {
        for (const sem of semesters) {
            limits[`${y}-${sem}`] = resolveSemesterCuLimit(sem, y, degrees, userOverrides);
        }
    }
    return limits;
}

/** Stable key for when CU policy should reset (school mix / degree count). */
export function degreeCuPolicyKey(degrees = []) {
    return degrees
        .map((d) => d.schoolCode || "")
        .sort()
        .join("|");
}

export function undergradScheduleYears(degrees) {
    const schools = (degrees || []).map((d) => d.schoolCode);
    if (schools.length < 2) return 4;
    if (schools.every((s) => s === "CAS")) return 4;
    if (isDualUndergradOnly(degrees)) return 5;
    return 4;
}

export function buildSemesterOptions(maxYear = 4, allowSummer = true) {
    const options = [
        { label: "—", value: "" },
        { label: "Credits Received", value: "Credits-0" },
    ];

    const years = Math.max(4, maxYear);
    for (let y = 1; y <= years; y++) {
        const name = YEAR_LABELS[y - 1] || `Year ${y}`;
        options.push({ label: `${name} Fall`, value: `Fall-${y}` });
        options.push({ label: `${name} Spring`, value: `Spring-${y}` });
        if (allowSummer) {
            options.push({ label: `${name} Summer`, value: `Summer-${y}` });
        }
    }

    return options;
}

export function maxYearFromSchedule(schedule) {
    if (!schedule?.length) return 4;
    const years = schedule.map((s) => s.year).filter((y) => y > 0);
    if (years.length === 0) return 4;
    return Math.max(4, ...years);
}
