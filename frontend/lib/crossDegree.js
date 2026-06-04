import { isRequirementSlotId } from "@/lib/courseUtils";

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
                if (!degMap[courseId]) degMap[courseId] = [];
                if (!degMap[courseId].includes(label)) degMap[courseId].push(label);
            });
        });
    }

    // Requirement slots are placeholders, not in cross_degree allocations.
    degreeResults.forEach((result) => {
        const degreeLabel = `${result.school}-${result.major}`;
        const addSlots = (mapped) => {
            mapped.course_ids?.forEach((id) => {
                if (!isRequirementSlotId(id)) return;
                if (!degMap[id]) degMap[id] = [];
                if (!degMap[id].includes(degreeLabel)) degMap[id].push(degreeLabel);
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
