/** Placement for a course from assigned (green) or frozen (orange) state. */
export function getCoursePlacement(courseId, assignedCourses = [], frozenCourses = []) {
    const assigned = assignedCourses.find((a) => a.courseId === courseId);
    if (assigned) return { year: assigned.year, semester: assigned.semester };
    const frozen = frozenCourses.find((f) => f.courseId === courseId);
    if (frozen) return { year: frozen.year, semester: frozen.semester };
    return null;
}

/** Index into semesterOptions order; unassigned sorts last. */
export function placementSortIndex(placement, semesterOptions) {
    if (!placement) return semesterOptions.length;
    if (placement.year === 0) {
        const idx = semesterOptions.findIndex((o) => o.value === "Credits-0");
        return idx >= 0 ? idx : 1;
    }
    const value = `${placement.semester}-${placement.year}`;
    const idx = semesterOptions.findIndex((o) => o.value === value);
    return idx >= 0 ? idx : semesterOptions.length - 1;
}

export function sortCourseCodesBySemester(
    courseCodes,
    { assignedCourses = [], frozenCourses = [], semesterOptions = [] } = {}
) {
    return [...courseCodes].sort((a, b) => {
        const ia = placementSortIndex(
            getCoursePlacement(a, assignedCourses, frozenCourses),
            semesterOptions
        );
        const ib = placementSortIndex(
            getCoursePlacement(b, assignedCourses, frozenCourses),
            semesterOptions
        );
        if (ia !== ib) return ia - ib;
        return a.localeCompare(b);
    });
}

/** Degree labels in API / courseDegreesMap order. */
export function buildDegreeOrder(scheduleData) {
    if (!scheduleData?.degree_results?.length) return [];
    return scheduleData.degree_results.map((r) => `${r.school}-${r.major}`);
}

/**
 * Sort: first-degree-only → shared (2+ degrees) → last-degree-only → unmapped.
 * With one degree, alphabetical only.
 */
export function sortCourseCodesByDegree(courseCodes, degreeOrder, courseDegreesMap = {}) {
    if (!degreeOrder?.length) return [...courseCodes].sort((a, b) => a.localeCompare(b));

    const bucket = (courseId) => {
        const set = new Set(courseDegreesMap[courseId] || []);
        const matched = degreeOrder.filter((d) => set.has(d));
        const n = degreeOrder.length;

        if (matched.length === 0) return 3;
        if (n >= 2 && matched.length >= 2) return 1;
        if (matched.length === 1) {
            const idx = degreeOrder.indexOf(matched[0]);
            if (idx === 0) return 0;
            if (idx === n - 1) return 2;
            return 1;
        }
        return 1;
    };

    return [...courseCodes].sort((a, b) => {
        const ba = bucket(a);
        const bb = bucket(b);
        if (ba !== bb) return ba - bb;
        return a.localeCompare(b);
    });
}
