import { isRequirementSlotId, isValidCourseCode } from "@/lib/courseUtils";

/** True when a course or slot counts toward this degree label in the schedule grid. */
export function courseCountsForDegree(courseId, degreeLabel, courseDegreesMap) {
    if (isRequirementSlotId(courseId)) return true;
    if (!isValidCourseCode(courseId)) return false;
    const degs = courseDegreesMap[courseId];
    if (!degs?.length) return false;
    return degs.includes(degreeLabel);
}

export function filterCoursesForDegree(courseIds, degreeLabel, courseDegreesMap) {
    return (courseIds || []).filter((id) => courseCountsForDegree(id, degreeLabel, courseDegreesMap));
}

/** Filter attribute fulfillment rows to courses allocated to this degree. */
export function filterAttributeFulfillmentForDegree(attrMap, degreeLabel, courseDegreesMap) {
    if (!attrMap || !(attrMap instanceof Map)) return attrMap;
    const filtered = new Map();
    attrMap.forEach((ids, code) => {
        const kept = filterCoursesForDegree(ids, degreeLabel, courseDegreesMap);
        if (kept.length) filtered.set(code, kept);
    });
    return filtered.size ? filtered : undefined;
}

/** Align overlay concentration progress with course_allocations / RequirementsPanel. */
export function filterConcentrationInfoForDegree(ci, degreeLabel, courseDegreesMap) {
    const slotCount = Math.max(
        ci.requirement_descriptions?.length ?? 0,
        ci.matched_courses?.length ?? 0,
        ci.requirement_fulfilled?.length ?? 0,
    );

    const matchedCourses = [];
    const requirementFulfilled = [];

    for (let j = 0; j < slotCount; j++) {
        const filtered = filterCoursesForDegree(
            ci.matched_courses?.[j] || [],
            degreeLabel,
            courseDegreesMap,
        );
        matchedCourses.push(filtered);
        requirementFulfilled.push(filtered.length > 0);
    }

    return {
        ...ci,
        matched_courses: matchedCourses,
        requirement_fulfilled: requirementFulfilled,
        requirements_fulfilled: requirementFulfilled.filter(Boolean).length,
    };
}

function addCourseToDegreeMap(degMap, courseId, degreeLabel) {
    if (!degMap[courseId]) degMap[courseId] = [];
    if (!degMap[courseId].includes(degreeLabel)) degMap[courseId].push(degreeLabel);
}

export function buildCourseDegreesMapFromAllocations(summary, degreeResults = []) {
    const degMap = {};
    if (summary?.course_allocations) {
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
                addCourseToDegreeMap(degMap, courseId, label);
            });
        });
    } else {
        // Single-degree (or pre-allocation): every course in degree_results belongs to that degree.
        degreeResults.forEach((result) => {
            const degreeLabel = `${result.school}-${result.major}`;
            const addCourses = (mapped) => {
                mapped.course_ids?.forEach((id) => {
                    if (isValidCourseCode(id)) addCourseToDegreeMap(degMap, id, degreeLabel);
                });
            };
            result.fulfilled_requirements?.forEach(addCourses);
            result.suggested_for_unfulfilled?.forEach(addCourses);
            result.unfulfilled_requirements?.forEach((mapped) => {
                if (mapped.partial) addCourses(mapped);
            });
            result.concentration_info?.forEach((ci) => {
                if (ci.is_core) return;
                (ci.matched_courses || []).flat().forEach((id) => {
                    if (isValidCourseCode(id)) addCourseToDegreeMap(degMap, id, degreeLabel);
                });
            });
        });
    }

    // Requirement slots are placeholders, not in cross_degree allocations.
    degreeResults.forEach((result) => {
        const degreeLabel = `${result.school}-${result.major}`;
        const addSlots = (mapped) => {
            mapped.course_ids?.forEach((id) => {
                if (!isRequirementSlotId(id)) return;
                addCourseToDegreeMap(degMap, id, degreeLabel);
            });
        };
        result.fulfilled_requirements?.forEach(addSlots);
        result.suggested_for_unfulfilled?.forEach(addSlots);
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
