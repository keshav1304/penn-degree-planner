import {
  isValidCourseCode,
  isRequirementSlotId,
  isSchedulableRequirementSlotId,
  isOverlapScheduleGroupId,
} from "@/lib/courseUtils";
import { resolveSemesterCuLimit, isGapSemester } from "@/lib/semesterOptions";

const SEM_ORDER = ["Fall", "Spring", "Summer"];

/**
 * Shared year/term display resolution for ScheduleGrid and Excel export.
 * Mirrors the merge rules in ScheduleGrid (pins win over API placement).
 */
export function buildScheduleDisplay({
  scheduleData,
  frozenCourses = [],
  assignedCourses = [],
  allowSummer = false,
  degrees = [],
  semesterCuLimits = {},
  gapSemesters = {},
  courseCuMap = {},
  requirementSlotLabels = {},
}) {
  const schedule = scheduleData?.schedule || [];
  const getCu = (courseId) => courseCuMap[courseId] ?? 1.0;

  const pinnedYears = [
    ...frozenCourses.map((f) => f.year),
    ...(assignedCourses || []).filter((a) => a.year > 0).map((a) => a.year),
  ];
  const uniqueYears = [...new Set([
    ...schedule.map((s) => s.year),
    ...pinnedYears,
  ])].filter((y) => y > 0).sort((a, b) => a - b);

  const getSemesterPlan = (year, semester) =>
    schedule.find((s) => s.year === year && s.semester === semester);

  const pinnedIds = new Set([
    ...frozenCourses.map((f) => f.courseId),
    ...(assignedCourses || []).map((a) => a.courseId),
  ]);

  const frozenIdSet = new Set(frozenCourses.map((f) => f.courseId));
  const assignedIdSet = new Set((assignedCourses || []).map((a) => a.courseId));

  const getDisplayCourses = (year, semester) => {
    const plan = getSemesterPlan(year, semester);
    const apiCourses = (plan?.courses || []).filter(
      (id) => !pinnedIds.has(id) && isValidCourseCode(id),
    );
    const pinnedHere = [
      ...frozenCourses
        .filter((f) => f.year === year && f.semester === semester)
        .map((f) => f.courseId),
      ...(assignedCourses || [])
        .filter((a) => a.year === year && a.semester === semester)
        .map((a) => a.courseId),
    ];
    return [...new Set([...pinnedHere, ...apiCourses])].filter(isValidCourseCode);
  };

  const overlapScheduleGroups = scheduleData?.overlap_schedule_groups ?? [];
  const overlapGroupById = Object.fromEntries(
    overlapScheduleGroups.map((g) => [g.group_id, g]),
  );
  const overlapMemberSlotIds = new Set(
    overlapScheduleGroups.flatMap((g) =>
      g.members.map((m) => m.schedule_slot_id).filter(isRequirementSlotId),
    ),
  );

  const getDisplayOverlapGroups = (year, semester) => {
    const plan = getSemesterPlan(year, semester);
    const apiGroups = (plan?.requirement_slots || []).filter(
      (id) => isOverlapScheduleGroupId(id) && !pinnedIds.has(id),
    );
    const pinnedHere = frozenCourses
      .filter(
        (f) =>
          f.year === year
          && f.semester === semester
          && isOverlapScheduleGroupId(f.courseId),
      )
      .map((f) => f.courseId);
    return [...new Set([...pinnedHere, ...apiGroups])];
  };

  const openRequirementSlotIds = new Set();
  scheduleData?.degree_results?.forEach((result) => {
    result.suggested_for_unfulfilled?.forEach((mapped) => {
      mapped.course_ids?.forEach((id) => {
        if (isSchedulableRequirementSlotId(id)) openRequirementSlotIds.add(id);
      });
    });
  });

  const getDisplayRequirementSlots = (year, semester) => {
    const plan = getSemesterPlan(year, semester);
    const apiSlots = (plan?.requirement_slots || []).filter(
      (id) => !pinnedIds.has(id) && openRequirementSlotIds.has(id),
    );
    const pinnedHere = frozenCourses
      .filter(
        (f) =>
          f.year === year
          && f.semester === semester
          && isSchedulableRequirementSlotId(f.courseId)
          && openRequirementSlotIds.has(f.courseId),
      )
      .map((f) => f.courseId);
    return [...new Set([...pinnedHere, ...apiSlots])]
      .filter(isSchedulableRequirementSlotId)
      .filter((id) => !isOverlapScheduleGroupId(id))
      .filter((id) => !overlapMemberSlotIds.has(id));
  };

  const semesterHasContent = (year, semester) => (
    getDisplayCourses(year, semester).length > 0
    || getDisplayOverlapGroups(year, semester).length > 0
    || getDisplayRequirementSlots(year, semester).length > 0
  );

  const visibleYears = uniqueYears.filter((year) => {
    if (year <= 4) return true;
    return SEM_ORDER.some((sem) => {
      if (sem === "Summer" && !allowSummer) return false;
      return semesterHasContent(year, sem);
    });
  });

  const visibleSemesters = SEM_ORDER.filter((s) => s !== "Summer" || allowSummer);

  const getSlotLabel = (slotId) => requirementSlotLabels[slotId] || "Open requirement";

  const slotCu = (id) => (
    isOverlapScheduleGroupId(id) || isRequirementSlotId(id) ? 1.0 : getCu(id)
  );

  const getSemesterItems = (year, semester) => {
    const courses = getDisplayCourses(year, semester);
    const overlapGroups = getDisplayOverlapGroups(year, semester);
    const requirementSlots = getDisplayRequirementSlots(year, semester);
    const items = [
      ...courses.map((id) => ({
        id,
        kind: "course",
        label: id,
        cu: getCu(id),
        status: assignedIdSet.has(id) ? "taken" : frozenIdSet.has(id) ? "frozen" : "suggested",
      })),
      ...overlapGroups.map((id) => {
        const group = overlapGroupById[id];
        const members = group?.members?.length
          ? group.members
          : [{ schedule_slot_id: id, label: getSlotLabel(id) }];
        const label = members
          .map((m) => {
            const slotText = m.schedule_slot_id ? getSlotLabel(m.schedule_slot_id) : "";
            return (slotText || m.label || "Overlap").split(/\n↳/)[0].trim();
          })
          .join(" + ");
        return {
          id,
          kind: "overlap",
          label,
          cu: 1.0,
          status: frozenIdSet.has(id) ? "frozen" : "suggested",
        };
      }),
      ...requirementSlots.map((id) => ({
        id,
        kind: "slot",
        label: getSlotLabel(id).split(/\n↳/)[0].trim(),
        cu: 1.0,
        status: frozenIdSet.has(id) ? "frozen" : "suggested",
      })),
    ];
    const actualCu = items.reduce((sum, item) => sum + item.cu, 0);
    const limitCu = resolveSemesterCuLimit(semester, year, degrees, semesterCuLimits);
    return { items, actualCu, limitCu };
  };

  const creditsCourses = (assignedCourses || []).filter((a) => a.year === 0);

  return {
    visibleYears,
    visibleSemesters,
    getSemesterPlan,
    getDisplayCourses,
    getDisplayOverlapGroups,
    getDisplayRequirementSlots,
    getSemesterItems,
    getSlotLabel,
    getCu,
    slotCu,
    openRequirementSlotIds,
    overlapGroupById,
    overlapMemberSlotIds,
    pinnedIds,
    creditsCourses,
    isFrozen: (courseId) => frozenIdSet.has(courseId),
    isAssigned: (courseId) => assignedIdSet.has(courseId),
    isGap: (year, semester) => isGapSemester(gapSemesters, year, semester),
  };
}
