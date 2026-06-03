export const UNDERGRAD_GRAD_CU_LIMIT = 3;
export const GRADUATE_LEVEL_MIN = 5000;
export const GRADUATE_SCHOOL_CODE = "SEAS_MS";

export function isGraduateDegree(schoolCode) {
    return schoolCode === GRADUATE_SCHOOL_CODE;
}

export function courseNumber(courseCode) {
    if (!courseCode || typeof courseCode !== "string") return null;
    const parts = courseCode.trim().split(/\s+/);
    if (parts.length < 2) return null;
    const num = parseInt(parts[1], 10);
    return Number.isFinite(num) ? num : null;
}

export function isGraduateLevelCourse(courseCode) {
    const num = courseNumber(courseCode);
    return num != null && num >= GRADUATE_LEVEL_MIN;
}

export function formatUndergradGradBudget(used, limit = UNDERGRAD_GRAD_CU_LIMIT) {
    const u = Number(used) || 0;
    const l = Number(limit) || UNDERGRAD_GRAD_CU_LIMIT;
    const rounded = Math.abs(u - Math.round(u)) < 0.001 ? Math.round(u) : u.toFixed(1);
    return `${rounded} / ${l} CU undergrad→grad double-count`;
}

export function buildCourseDegreesMapFromAllocations(summary, degreeResults = []) {
    const degMap = {};
    if (!summary?.course_allocations) return degMap;

    const labelForIndex = (degreeIndex) => {
        const result = degreeResults[degreeIndex];
        if (!result) return null;
        return `${result.school}-${result.major}`;
    };

    Object.entries(summary.course_allocations).forEach(([courseId, allocs]) => {
        allocs.forEach((alloc) => {
            const label =
                alloc.school && alloc.major
                    ? `${alloc.school}-${alloc.major}`
                    : labelForIndex(alloc.degree_index);
            if (!label) return;
            if (!degMap[courseId]) degMap[courseId] = [];
            if (!degMap[courseId].includes(label)) degMap[courseId].push(label);
        });
    });
    return degMap;
}

export function courseViolationMap(summary) {
    const map = {};
    (summary?.violations || []).forEach((v) => {
        if (v.course_id) {
            map[v.course_id] = v.message;
        }
    });
    return map;
}

export function coursesUsingUndergradGradBudget(summary) {
    const set = new Set();
    if (!summary?.course_allocations) return set;
    Object.entries(summary.course_allocations).forEach(([courseId, allocs]) => {
        if (allocs.some((a) => a.uses_undergrad_grad_budget)) {
            set.add(courseId);
        }
    });
    return set;
}
