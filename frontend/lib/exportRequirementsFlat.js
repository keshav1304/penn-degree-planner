import {
  isPoolConstraintInstanceId,
  isPoolFlexibleSlotInstanceId,
  isValidCourseCode,
} from "@/lib/courseUtils";
import { filterCoursesForDegree } from "@/lib/crossDegree";
import {
  normalizeCategory,
  getCategory,
  poolGroupStats,
  poolProgressLabel,
  isPoolComplete,
} from "@/lib/casRequirementsLayout";
import { formatDegreeApiLabel, catalogForProgram, isMinorProgram } from "@/lib/degreeDisplay";
import {
  createRequirementDescription,
  getRequirementInstanceId,
  itemEffectiveFulfilled,
  parseRequirement,
  courseGroupCreditedCount,
  courseGroupNeedCount,
  isExpandableCourseGroup,
  restrictionRequiredCu,
} from "@/lib/requirementText";

/**
 * Flatten one degree's requirements into Excel-friendly rows.
 * Covers fulfilled / unfulfilled / pools / concentrations — same data as RequirementsPanel.
 */
export function flattenDegreeRequirementRows({
  result,
  degree,
  degreeIndex,
  degreeCatalog = [],
  minorCatalog = [],
  courseDegreesMap = {},
  concentrationData = [],
}) {
  if (!result) return { title: "Requirements", rows: [] };

  const catalog = catalogForProgram(degree, result, degreeCatalog, minorCatalog);
  const title = `${formatDegreeApiLabel(result.school, result.major, catalog)} REQUIREMENTS`;
  const degreeLabel = `${result.school}-${result.major}`;
  const trustBackendCourses = isMinorProgram(degree, result);

  const rows = [];
  const pushHeader = (label) => {
    rows.push({
      kind: "header",
      requirement: label,
      status: "",
      cu: "",
      courses: "",
    });
  };
  const pushRow = ({ requirement, status, cu, courses }) => {
    rows.push({
      kind: "row",
      requirement: requirement || "",
      status: status || "",
      cu: cu === "" || cu == null ? "" : cu,
      courses: courses || "",
    });
  };

  const mapItem = (mapped, { fulfilledDefault, partialDefault }) => {
    const rawCourseIds = (mapped.course_ids || []).filter(isValidCourseCode);
    const fulfilledCourses = trustBackendCourses
      ? rawCourseIds
      : filterCoursesForDegree(mapped.course_ids || [], degreeLabel, courseDegreesMap);
    const hasAllocated = fulfilledCourses.length > 0;
    return {
      category: normalizeCategory(getCategory(mapped.requirement)),
      fulfilled: fulfilledDefault && hasAllocated,
      partial: partialDefault && hasAllocated,
      fulfilledCourses,
      requirement: mapped.requirement,
      instanceId: getRequirementInstanceId(mapped),
    };
  };

  const allReqs = [];
  (result.fulfilled_requirements || []).forEach((mapped) => {
    const id = getRequirementInstanceId(mapped);
    if (isPoolConstraintInstanceId(id) || isPoolFlexibleSlotInstanceId(id)) return;
    allReqs.push(mapItem(mapped, { fulfilledDefault: true, partialDefault: false }));
  });
  (result.unfulfilled_requirements || []).forEach((mapped) => {
    const id = getRequirementInstanceId(mapped);
    if (isPoolConstraintInstanceId(id) || isPoolFlexibleSlotInstanceId(id)) return;
    const req = mapped?.requirement ?? mapped;
    allReqs.push(
      mapItem(
        { ...mapped, requirement: req },
        { fulfilledDefault: false, partialDefault: Boolean(mapped.partial) },
      ),
    );
  });

  const suggestedByInstance = new Map();
  (result.suggested_for_unfulfilled || []).forEach((mapped) => {
    const id = getRequirementInstanceId(mapped);
    if (!id) return;
    const codes = (mapped.course_ids || []).filter(isValidCourseCode);
    if (codes.length) suggestedByInstance.set(id, codes);
  });

  const pools = result.pool_coverage_info || [];
  const categoryForItem = (item) => {
    const cat = normalizeCategory(item.category);
    for (const pool of pools) {
      const poolCat = normalizeCategory(pool.category);
      if (cat === poolCat || cat === `${poolCat} - Pool course`) return poolCat;
    }
    return cat;
  };

  const categoryMap = {};
  allReqs.forEach((item) => {
    const cat = categoryForItem(item);
    if (!categoryMap[cat]) categoryMap[cat] = [];
    categoryMap[cat].push(item);
  });
  pools.forEach((pool) => {
    const cat = normalizeCategory(pool.category);
    if (!categoryMap[cat]) categoryMap[cat] = [];
  });

  const orderedCategories = (result.category_order || []).map(normalizeCategory);
  Object.keys(categoryMap).forEach((c) => {
    if (!orderedCategories.includes(c)) orderedCategories.push(c);
  });

  const statusForItem = (item) => {
    if (itemEffectiveFulfilled(item)) return "Fulfilled";
    if (item.partial) return "Partial";
    return "Open";
  };

  const cuForItem = (item) => {
    const { type, data } = parseRequirement(item.requirement);
    if (type === "Restriction") return restrictionRequiredCu(data.number, data.cu);
    if (type === "CourseGroup") return courseGroupNeedCount(item);
    if (type === "SingleCourse") return 1;
    return "";
  };

  const coursesForItem = (item) => {
    if (item.fulfilledCourses?.length) return item.fulfilledCourses.join(", ");
    const suggested = suggestedByInstance.get(item.instanceId);
    if (suggested?.length) return `Suggested: ${suggested.slice(0, 6).join(", ")}`;
    if (isExpandableCourseGroup(item.requirement)) {
      return `${courseGroupCreditedCount(item)}/${courseGroupNeedCount(item)} areas`;
    }
    return "";
  };

  for (const cat of orderedCategories) {
    const items = categoryMap[cat] || [];
    const pool = pools.find((p) => normalizeCategory(p.category) === cat);
    pushHeader(cat);
    if (pool) {
      const stats = poolGroupStats(pool);
      pushRow({
        requirement: `Pool progress`,
        status: isPoolComplete(stats) ? "Fulfilled" : "Open",
        cu: "",
        courses: poolProgressLabel(stats) || "",
      });
    }
    for (const item of items) {
      pushRow({
        requirement: createRequirementDescription(item.requirement),
        status: statusForItem(item),
        cu: cuForItem(item),
        courses: coursesForItem(item),
      });
    }
  }

  const concForDegree = (concentrationData || []).filter(
    (c) => c.degreeLabel === degreeLabel || c.degree_index === degreeIndex,
  );
  if (concForDegree.length) {
    pushHeader("Concentrations");
    for (const ci of concForDegree) {
      const fulfilledCount = ci.requirements_fulfilled || 0;
      const totalCount = ci.requirements_total || 0;
      pushRow({
        requirement: ci.name || "Concentration",
        status: fulfilledCount === totalCount && totalCount > 0 ? "Fulfilled" : "Open",
        cu: "",
        courses: `${fulfilledCount}/${totalCount}`,
      });
      (ci.requirement_descriptions || []).forEach((desc, j) => {
        const ok = ci.requirement_fulfilled?.[j];
        const matched = ci.matched_courses?.[j] || [];
        pushRow({
          requirement: `  ${desc}`,
          status: ok ? "Fulfilled" : "Open",
          cu: "",
          courses: matched.join(", "),
        });
      });
    }
  }

  if (result.error) {
    pushHeader("Errors");
    pushRow({
      requirement: result.error,
      status: "Error",
      cu: "",
      courses: "",
    });
  }

  if (rows.length === 0) {
    pushRow({
      requirement: "No requirements listed",
      status: "",
      cu: "",
      courses: "",
    });
  }

  return { title, rows };
}

/** Flatten all degrees for side-by-side Excel layout. */
export function flattenAllDegreeRequirements({
  scheduleData,
  degrees = [],
  degreeCatalog = [],
  minorCatalog = [],
  courseDegreesMap = {},
  concentrationData = [],
}) {
  const results = scheduleData?.degree_results || [];
  return results.map((result, degreeIndex) =>
    flattenDegreeRequirementRows({
      result,
      degree: degrees[degreeIndex],
      degreeIndex,
      degreeCatalog,
      minorCatalog,
      courseDegreesMap,
      concentrationData,
    }),
  );
}
