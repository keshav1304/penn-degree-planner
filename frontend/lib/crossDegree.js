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
        .filter((r) => r?.school === "CAS" && r?.kind !== "minor")
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
    const casDegreeLabels = casDegreeLabelsFromResults(degreeResults);
    const isDualCasCollege = casDegreeLabels.length >= 2;

    degreeResults.forEach((result) => {
        const degreeLabel = labelForDegreeResult(result);
        const addCourses = (mapped) => {
            const category = mapped?.requirement
                ? (mapped.requirement.category
                    || mapped.requirement.Restriction?.category
                    || mapped.requirement.SingleCourse?.category
                    || mapped.requirement.CoursePool?.category
                    || "")
                : "";
            const collegeWide =
                isDualCasCollege
                && (category === "Unrestricted Electives"
                    || category === "Writing Seminar"
                    || category === "General Education");
            mapped.course_ids?.forEach((id) => {
                if (!isValidCourseCode(id)) return;
                if (collegeWide) {
                    casDegreeLabels.forEach((label) => addCourseToDegreeMap(degMap, id, label));
                } else {
                    addCourseToDegreeMap(degMap, id, degreeLabel);
                }
            });
        };
        result.fulfilled_requirements?.forEach(addCourses);
        result.suggested_for_unfulfilled?.forEach(addCourses);
        result.unfulfilled_requirements?.forEach((mapped) => {
            if (mapped.partial) addCourses(mapped);
        });
        // Pool coverage matches are navigable (same source as req-nav arrows).
        (result.pool_coverage_info || []).forEach((pool) => {
            (pool.constraints || []).forEach((constraint) => {
                (constraint.matched_courses || []).forEach((id) => {
                    if (!isValidCourseCode(id)) return;
                    if (isDualCasCollege && (pool.category === "General Education")) {
                        casDegreeLabels.forEach((label) => addCourseToDegreeMap(degMap, id, label));
                    } else {
                        addCourseToDegreeMap(degMap, id, degreeLabel);
                    }
                });
            });
        });
    });
    return degMap;
}

/** Re-apply fulfillment from every program after allocation merge (minors always double-count). */
function ensureFulfillmentLabels(degMap, degreeResults = []) {
    degreeResults.forEach((result) => {
        if (!result) return;
        const degreeLabel = labelForDegreeResult(result);
        const addCourse = (id) => {
            if (isValidCourseCode(id)) addCourseToDegreeMap(degMap, id, degreeLabel);
        };
        const processMapped = (mapped) => mapped.course_ids?.forEach(addCourse);
        result.fulfilled_requirements?.forEach(processMapped);
        result.suggested_for_unfulfilled?.forEach(processMapped);
        result.unfulfilled_requirements?.forEach((mapped) => {
            if (mapped.partial) processMapped(mapped);
        });
        (result.pool_coverage_info || []).forEach((pool) => {
            (pool.constraints || []).forEach((constraint) => {
                (constraint.matched_courses || []).forEach(addCourse);
            });
        });
    });
}

export function buildCourseDegreesMapFromAllocations(summary, degreeResults = []) {
    // Seed from navigable requirement rows only (fulfilled / partial / suggested / pool matches).
    // This keeps schedule stripes aligned with req-nav arrows. Courses that fulfill nothing
    // never enter the map and therefore get no degree stripes.
    const degMap = buildCourseMapFromDegreeResults(degreeResults);
    const casDegreeLabels = casDegreeLabelsFromResults(degreeResults);
    const isDualCasCollege = casDegreeLabels.length >= 2;

    // Merge backend allocations (shared / conflict-resolved ownership).
    if (summary?.course_allocations) {
        Object.entries(summary.course_allocations).forEach(([courseId, allocs]) => {
            if (!isValidCourseCode(courseId)) return;
            // Only merge labels for courses already navigable from requirement rows,
            // so allocation-only orphans cannot invent stripes.
            if (!degMap[courseId]?.length) return;
            const allocLabels = [];
            allocs.forEach((alloc) => {
                const label = labelForAllocation(alloc, degreeResults);
                if (label && !allocLabels.includes(label)) allocLabels.push(label);
            });
            if (!allocLabels.length) return;
            const merged = [...(degMap[courseId] || [])];
            allocLabels.forEach((label) => {
                if (!merged.includes(label)) merged.push(label);
            });
            degMap[courseId] = merged;
        });
    }

    // Requirement slots are placeholders, not in cross_degree allocations.
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

    ensureFulfillmentLabels(degMap, degreeResults);

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
