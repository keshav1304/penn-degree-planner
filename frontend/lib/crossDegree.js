import { isValidCourseCode, isRequirementSlotId } from "@/lib/courseUtils";

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

/** Authoritative allocations when multi-degree; single-degree fallback for stripe display. */
export function buildCourseDegreesMap(summary, degreeResults = []) {
    const fromAllocations = buildCourseDegreesMapFromAllocations(summary, degreeResults);
    if (Object.keys(fromAllocations).length > 0) return fromAllocations;

    const activeResults = (degreeResults || []).filter((r) => !r?.error);
    if (activeResults.length !== 1) return fromAllocations;

    const label = `${activeResults[0].school}-${activeResults[0].major}`;
    const degMap = {};
    const addDegree = (itemId) => {
        if (!isValidCourseCode(itemId) && !isRequirementSlotId(itemId)) return;
        if (!degMap[itemId]) degMap[itemId] = [];
        if (!degMap[itemId].includes(label)) degMap[itemId].push(label);
    };

    activeResults.forEach((result) => {
        result.fulfilled_requirements?.forEach((mapped) => {
            mapped.course_ids?.forEach(addDegree);
        });
        result.suggested_for_unfulfilled?.forEach((mapped) => {
            mapped.course_ids?.forEach(addDegree);
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
