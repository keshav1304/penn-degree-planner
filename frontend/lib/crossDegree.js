import { isRequirementSlotId, isValidCourseCode } from "@/lib/courseUtils";

/** Instance scope for college-wide CAS requirements (writing / gen-ed pool). */
export function isCasCollegeSharedInstanceScope(scope) {
    if (!scope || typeof scope !== "string") return false;
    if (scope === "0" || scope === "1") return true;
    if (scope.startsWith("1:")) {
        return !scope.slice(2).startsWith("f");
    }
    return false;
}

/** Schedule slot id for a college-wide CAS requirement (writing / gen-ed). */
export function isCasCollegeSharedScheduleSlot(slotId) {
    if (!isRequirementSlotId(slotId)) return false;
    const scope = slotId.slice(4).split(":R:")[0];
    return isCasCollegeSharedInstanceScope(scope);
}

export function casDegreeLabelsFromResults(degreeResults = []) {
    return degreeResults
        .filter((r) => r?.school === "CAS")
        .map((r) => `${r.school}-${r.major}`);
}

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

function labelForDegreeResult(result) {
    return `${result.school}-${result.major}`;
}

function labelForAllocation(alloc, degreeResults) {
    if (alloc.school && alloc.major) {
        return `${alloc.school}-${alloc.major}`;
    }
    const result = degreeResults[alloc.degree_index];
    return result ? labelForDegreeResult(result) : null;
}

function buildCourseMapFromDegreeResults(degreeResults = []) {
    const degMap = {};
    degreeResults.forEach((result) => {
        const degreeLabel = labelForDegreeResult(result);
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
    return degMap;
}

export function buildCourseDegreesMapFromAllocations(summary, degreeResults = []) {
    // Always seed from degree_results (fulfilled + suggested). Required for multi-degree
    // plans without the cross-degree overlap optimizer (e.g. undergrad + SEAS_MS), where
    // course_allocations only lists fulfilled/taken courses—not scheduler suggestions.
    const degMap = buildCourseMapFromDegreeResults(degreeResults);

    // Authoritative for shared / resolved courses when the backend tracked allocations.
    if (summary?.course_allocations) {
        Object.entries(summary.course_allocations).forEach(([courseId, allocs]) => {
            if (!isValidCourseCode(courseId)) return;
            const labels = [];
            allocs.forEach((alloc) => {
                const label = labelForAllocation(alloc, degreeResults);
                if (label && !labels.includes(label)) labels.push(label);
            });
            if (labels.length) degMap[courseId] = labels;
        });
    }

    // Requirement slots are placeholders, not in cross_degree allocations.
    const casDegreeLabels = casDegreeLabelsFromResults(degreeResults);
    const isDualCasCollege = casDegreeLabels.length >= 2;
    degreeResults.forEach((result) => {
        const degreeLabel = `${result.school}-${result.major}`;
        const addSlots = (mapped) => {
            mapped.course_ids?.forEach((id) => {
                if (!isRequirementSlotId(id)) return;
                if (isDualCasCollege && isCasCollegeSharedScheduleSlot(id)) {
                    casDegreeLabels.forEach((label) => addCourseToDegreeMap(degMap, id, label));
                } else {
                    addCourseToDegreeMap(degMap, id, degreeLabel);
                }
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
