/** Parse a serialized Rust Requirement enum from the API. */
export function parseRequirement(req) {
  if (!req || typeof req !== "object") return { type: "Unknown", data: {} };
  const variants = [
    "SingleCourse",
    "CourseGroup",
    "AnyOf",
    "AllOf",
    "Concentration",
    "Restriction",
    "CoursePool",
  ];
  for (const v of variants) {
    if (req[v] !== undefined) return { type: v, data: req[v] };
  }
  if (req.possibilities) return { type: "SingleCourse", data: req };
  if (req.department !== undefined || req.attr !== undefined) {
    return { type: "Restriction", data: req };
  }
  return { type: "Unknown", data: req };
}

/** Must stay in sync with Rust `restriction_required_cu`. */
export function restrictionRequiredCu(number, cuField) {
  if (cuField != null) return cuField / 10;
  return number ?? 1;
}

const MAX_LISTED_COURSES = 4;
const MAX_SCHEDULE_LISTED_COURSES = 3;
const CU_EPS = 0.001;

/** Must stay in sync with Rust `format_restriction_description` CU prefix. */
export function formatCuLabel(number, cuField) {
  const target = restrictionRequiredCu(number, cuField);
  if (Math.abs(target - Math.round(target)) < CU_EPS) {
    return `${Math.round(target)} CU`;
  }
  return `${target} CU`;
}

/** Must stay in sync with Rust `format_truncated_list`. */
export function formatTruncatedList(items, prefix = "") {
  const list = (items || []).filter((item) => item != null && item !== "");
  if (list.length === 0) {
    return `${prefix}(options not specified)`;
  }
  if (list.length === 1) {
    return list[0];
  }
  if (list.length <= MAX_LISTED_COURSES) {
    return `${prefix}${list.join(", ")}`;
  }
  const shown = list.slice(0, MAX_LISTED_COURSES);
  const more = list.length - MAX_LISTED_COURSES;
  return `${prefix}${shown.join(", ")} (+${more} more)`;
}

/** Must stay in sync with Rust `format_schedule_single_course_label`. */
export function formatScheduleSingleCourseLabel(possibilities) {
  const list = (possibilities || []).filter((item) => item != null && item !== "");
  if (list.length === 0) {
    return "1 CU from (options not specified)";
  }
  if (list.length === 1) {
    return list[0];
  }
  if (list.length <= MAX_SCHEDULE_LISTED_COURSES) {
    return `1 CU from ${list.join(", ")}`;
  }
  const shown = list.slice(0, MAX_SCHEDULE_LISTED_COURSES);
  const more = list.length - MAX_SCHEDULE_LISTED_COURSES;
  return `1 CU from ${shown.join(", ")} (+${more})`;
}

/** Must stay in sync with Rust `format_restriction_description`. */
export function formatRestriction(data) {
  let response = formatCuLabel(data.number, data.cu);
  if (data.department?.length) {
    const depts = Array.isArray(data.department) ? data.department : [data.department];
    response += ` from ${depts.join("/")}`;
  }
  if (data.level != null) {
    response += ` min. level ${data.level}`;
  }
  if (data.attr?.length) {
    const attrs = data.attr.filter((a) => typeof a === "string");
    if (attrs.length) response += ` from attribute ${attrs.join("/")}`;
  }
  if (data.no_school) response += ` not from ${data.no_school}`;
  return response || "Restriction requirement";
}

/** Must stay in sync with Rust `create_requirement_description`. */
export function createRequirementDescription(req) {
  const { type, data } = parseRequirement(req);
  switch (type) {
    case "SingleCourse":
      return formatTruncatedList(data.possibilities, "One of: ");
    case "CourseGroup":
      return `Complete ${data.number ?? "N"} of ${(data.possibilities || []).length} areas`;
    case "Restriction":
      return formatRestriction(data);
    case "AnyOf":
      if (data.possibilities?.length === 1) {
        return createRequirementDescription(data.possibilities[0]);
      }
      return "One of the following options";
    case "AllOf": {
      const parts = (data.requirements || [])
        .map((sub) => createRequirementDescription(sub))
        .filter(Boolean);
      if (parts.length === 0) {
        return `Complete all ${(data.requirements || []).length} sub-requirements`;
      }
      return parts.join(" + ");
    }
    case "Concentration":
      return `Concentration: ${data.number} CU`;
    case "CoursePool": {
      const flexCount = data.flexible_slots ?? 0;
      const constraintLabels = (data.constraints || []).map((c) => c.label || createRequirementDescription(c.requirement));
      const fixedPart = (data.fixed_slots || []).length
        ? `${(data.fixed_slots || []).length} major course(s)`
        : "";
      const poolPart = flexCount > 0 ? `${flexCount} pool elective(s)` : "";
      const slots = [fixedPart, poolPart].filter(Boolean).join(" + ");
      if (constraintLabels.length) {
        return `${slots}. Coverage: ${constraintLabels.join(", ")}`;
      }
      return slots || "Course pool";
    }
    default:
      return "Requirement";
  }
}

/** AnyOf with 2+ possibilities is shown as indented sub-rows in the requirements panel. */
export function isExpandableAnyOf(req) {
  const { type, data } = parseRequirement(req);
  return type === "AnyOf" && (data.possibilities?.length ?? 0) > 1;
}

export function getAnyOfPossibilities(req) {
  const { type, data } = parseRequirement(req);
  if (type !== "AnyOf") return [];
  return data.possibilities || [];
}

export function getAnyOfCategory(req) {
  const { type, data } = parseRequirement(req);
  if (type !== "AnyOf") return null;
  const cat = data.category;
  return typeof cat === "string" && cat.trim() ? cat.trim() : null;
}

/** CourseGroup with child areas is shown as indented sub-rows in the requirements panel. */
export function isExpandableCourseGroup(req) {
  const { type, data } = parseRequirement(req);
  return type === "CourseGroup" && (data.possibilities?.length ?? 0) > 0;
}

export function getCourseGroupChildren(req) {
  const { type, data } = parseRequirement(req);
  if (type !== "CourseGroup") return [];
  return data.possibilities || [];
}

export function courseGroupHeaderLabel(requirement) {
  const { type, data } = parseRequirement(requirement);
  if (type !== "CourseGroup") return "Complete areas";
  const need = data.number ?? 1;
  const total = (data.possibilities || []).length;
  return `Complete ${need} of ${total} areas:`;
}

export function getCourseGroupAreaLabel(childReq) {
  const { type, data } = parseRequirement(childReq);
  const cat = data.category;
  if (typeof cat === "string" && cat.trim()) return cat.trim();
  if (type === "SingleCourse") {
    const count = (data.possibilities || []).filter(Boolean).length;
    return count > 1 ? "One of" : "Course";
  }
  return getRequirementStem(childReq) || "Area";
}

function childFullyFulfills(childReq, courses) {
  if (!courses?.length) return false;
  const { type, data } = parseRequirement(childReq);
  if (type === "SingleCourse") {
    const allowed = new Set(data.possibilities || []);
    return courses.some((c) => allowed.has(c));
  }
  if (type === "AllOf") {
    const pool = [...courses];
    return (data.requirements || []).every((sub) => {
      const matches = coursesMatchingChildLeaf(sub, pool);
      if (matches.length === 0) return false;
      matches.forEach((c) => {
        const idx = pool.indexOf(c);
        if (idx >= 0) pool.splice(idx, 1);
      });
      return true;
    });
  }
  if (type === "CourseGroup") {
    const pool = [...courses];
    let fulfilled = 0;
    const need = data.number ?? 1;
    for (const child of data.possibilities || []) {
      if (fulfilled >= need) break;
      const matches = coursesMatchingChildLeaf(child, pool);
      if (matches.length > 0 && childFullyFulfills(child, pool)) {
        fulfilled += 1;
        matches.forEach((c) => {
          const idx = pool.indexOf(c);
          if (idx >= 0) pool.splice(idx, 1);
        });
      }
    }
    return fulfilled >= need;
  }
  return childPartiallyMatches(childReq, courses);
}

/** Per-area fulfillment for a CourseGroup parent (N-of areas). */
export function evaluateCourseGroupChildren(parent) {
  const { type, data } = parseRequirement(parent.requirement);
  if (type !== "CourseGroup") return [];

  const children = data.possibilities || [];
  const need = data.number ?? 1;
  const parentCourses = collectParentCourses(parent);
  let allocPool = [...parentCourses];
  const credited = new Set();

  for (let childIdx = 0; childIdx < children.length; childIdx += 1) {
    if (credited.size >= need) break;
    const childReq = children[childIdx];
    const matched = coursesMatchingChildLeaf(childReq, allocPool);
    if (matched.length > 0 && childFullyFulfills(childReq, allocPool)) {
      credited.add(childIdx);
      matched.forEach((c) => {
        const idx = allocPool.indexOf(c);
        if (idx >= 0) allocPool.splice(idx, 1);
      });
    }
  }

  return children.map((childReq, childIdx) => {
    const fulfilledCourses = coursesMatchingChildLeaf(childReq, parentCourses);
    const hasMatch = fulfilledCourses.length > 0;
    const creditedArea = credited.has(childIdx);
    let tone = "open";
    if (creditedArea) {
      tone = "fulfilled";
    } else if (hasMatch) {
      tone = "partial";
    }
    return { childReq, childIdx, fulfilledCourses, tone, credited: creditedArea };
  });
}

/** Number of areas credited toward a CourseGroup's N-of requirement. */
export function courseGroupCreditedCount(parent) {
  return evaluateCourseGroupChildren(parent).filter((row) => row.credited).length;
}

/** Required number of areas for a CourseGroup (defaults to 1). */
export function courseGroupNeedCount(parent) {
  const { type, data } = parseRequirement(parent.requirement);
  if (type !== "CourseGroup") return 1;
  return data.number ?? 1;
}

/** Whether enough areas are credited to satisfy the CourseGroup. */
export function courseGroupIsComplete(parent) {
  return courseGroupCreditedCount(parent) >= courseGroupNeedCount(parent);
}

/** True when a requirement row is satisfied (CourseGroup uses credited areas, not API flag). */
export function itemEffectiveFulfilled(item) {
  if (isExpandableCourseGroup(item.requirement)) {
    return courseGroupIsComplete(item);
  }
  return item.fulfilled;
}

/**
 * Progress in "slots" for a category list.
 * CourseGroup rows contribute N credits (e.g. 2/3 areas); other rows count as 1 each.
 */
export function categoryProgressCounts(items) {
  let done = 0;
  let total = 0;
  for (const item of items || []) {
    const { type, data } = parseRequirement(item.requirement);
    if (type === "CourseGroup" && (data.possibilities?.length ?? 0) > 0) {
      done += courseGroupCreditedCount(item);
      total += courseGroupNeedCount(item);
    } else {
      total += 1;
      if (itemEffectiveFulfilled(item)) done += 1;
    }
  }
  return { done, total };
}

export function formatCategoryProgress(items) {
  const { done, total } = categoryProgressCounts(items);
  return `${done}/${total}`;
}

function courseMeetsRestriction(courseId, data, attrMap) {
  const parts = courseId.split(" ");
  if (parts.length < 2) return false;
  const dept = parts[0];
  const num = parseInt(parts[1], 10);
  if (data.department?.length && !data.department.includes(dept)) return false;
  if (data.level != null && (Number.isNaN(num) || num < data.level)) return false;
  if (data.excluding?.includes(courseId)) return false;
  if (data.attr?.length) {
    return data.attr.some((a) => (attrMap?.get(a) || []).includes(courseId));
  }
  return true;
}

/** Whether this AnyOf child is the branch satisfied by the parent's fulfilling courses. */
export function childMatchesAnyOfFulfillment(childReq, parent, childIdx = null) {
  const courses = collectParentCourses(parent);
  if (courses.length === 0) return false;

  if (parent.partial && parent.committedAnyofBranch != null && childIdx != null) {
    if (childIdx !== parent.committedAnyofBranch) return false;
    return childPartiallyMatches(childReq, courses, parent.attributeFulfillment);
  }

  if (!parent.fulfilled) return false;

  const attrMap = parent.attributeFulfillment;
  const { type, data } = parseRequirement(childReq);

  switch (type) {
    case "SingleCourse": {
      const allowed = new Set(data.possibilities || []);
      const matched = courses.filter((c) => allowed.has(c));
      return matched.length > 0 && matched.length === courses.length;
    }
    case "AllOf": {
      const pool = [...courses];
      return (data.requirements || []).every((sub) => {
        const { type: st, data: sd } = parseRequirement(sub);
        if (st === "SingleCourse") {
          const opts = sd.possibilities || [];
          const idx = pool.findIndex((c) => opts.includes(c));
          if (idx < 0) return false;
          pool.splice(idx, 1);
          return true;
        }
        return childMatchesAnyOfFulfillment(sub, {
          fulfilled: true,
          fulfilledCourses: pool,
          attributeFulfillment: attrMap,
        });
      });
    }
    case "CourseGroup":
      return childFullyFulfills(childReq, courses);
    case "Restriction":
      return courses.every((c) => courseMeetsRestriction(c, data, attrMap))
        && courses.some((c) => courseMeetsRestriction(c, data, attrMap));
    default:
      return false;
  }
}

function collectParentCourses(parent) {
  const courses = [];
  (parent.fulfilledCourses || []).forEach((c) => {
    if (c && typeof c === "string") courses.push(c);
  });
  parent.attributeFulfillment?.forEach((ids) => {
    ids.forEach((c) => { if (c && typeof c === "string") courses.push(c); });
  });
  return [...new Set(courses)];
}

/** Whether a child requirement has at least one matching course from the parent's pool (partial OK). */
export function childPartiallyMatches(childReq, courses, attrMap = null) {
  if (!courses?.length) return false;
  const { type, data } = parseRequirement(childReq);

  switch (type) {
    case "SingleCourse": {
      const allowed = new Set(data.possibilities || []);
      return courses.some((c) => allowed.has(c));
    }
    case "AllOf":
      return (data.requirements || []).some((sub) =>
        childPartiallyMatches(sub, courses, attrMap)
      );
    case "CourseGroup":
      return (data.possibilities || []).some((sub) =>
        childPartiallyMatches(sub, courses, attrMap)
      );
    case "Restriction":
      return courses.some((c) => courseMeetsRestriction(c, data, attrMap));
    default:
      return false;
  }
}

/** Courses from the parent pool that fulfill a nested leaf (SingleCourse / CourseGroup). */
export function coursesMatchingChildLeaf(childReq, parentCourses) {
  const { type, data } = parseRequirement(childReq);
  if (type === "SingleCourse") {
    const allowed = new Set(data.possibilities || []);
    return parentCourses.filter((c) => allowed.has(c));
  }
  if (type === "CourseGroup") {
    const pool = [...parentCourses];
    const matched = [];
    const need = data.number ?? 1;
    let count = 0;
    for (const child of data.possibilities || []) {
      if (count >= need) break;
      const subMatches = coursesMatchingChildLeaf(child, pool);
      if (subMatches.length > 0) {
        subMatches.forEach((c) => {
          matched.push(c);
          const idx = pool.indexOf(c);
          if (idx >= 0) pool.splice(idx, 1);
        });
        count += 1;
      }
    }
    return matched;
  }
  if (type === "AllOf") {
    const pool = [...parentCourses];
    const matched = [];
    (data.requirements || []).forEach((sub) => {
      const subMatches = coursesMatchingChildLeaf(sub, pool);
      subMatches.forEach((c) => {
        matched.push(c);
        const idx = pool.indexOf(c);
        if (idx >= 0) pool.splice(idx, 1);
      });
    });
    return matched;
  }
  return [];
}

/** Stable left-side stem for the requirements panel (never changes with fulfillment state). */
export function getRequirementStem(req) {
  const { type, data } = parseRequirement(req);
  switch (type) {
    case "SingleCourse": {
      const count = (data.possibilities || []).filter(Boolean).length;
      return count > 1 ? "One of" : null;
    }
    case "CourseGroup":
      return null;
    case "Restriction":
      return formatRestriction(data);
    case "AnyOf":
      if (data.possibilities?.length === 1) return getRequirementStem(data.possibilities[0]);
      return null;
    case "AllOf":
      return "Complete all";
    case "Concentration":
      return `Concentration (${data.number} CU)`;
    case "CoursePool":
      return createRequirementDescription(req);
    default:
      return createRequirementDescription(req);
  }
}

/** Human-readable label for a requirement (never joins nested objects). */
export function getRequirementLabel(req) {
  const { type, data } = parseRequirement(req);
  switch (type) {
    case "Restriction":
      return formatRestriction(data);
    case "AnyOf":
      if (data.possibilities?.length === 1) {
        return getRequirementLabel(data.possibilities[0]);
      }
      return "One of the following options";
    case "AllOf":
      return "Complete all sub-requirements";
    case "SingleCourse":
      return "Complete one listed course";
    case "CourseGroup":
      return `Complete ${data.number ?? "N"} of ${(data.possibilities || []).length} areas`;
    case "Concentration":
      return `Concentration (${data.number ?? "N"} CU)`;
    case "CoursePool":
      return "Course pool requirement";
    default:
      return "Requirement";
  }
}

/** Must stay in sync with Rust `scoped_slot_id` + `schedulable_placeholder_id`. */
function slotIdMatchesScopedFingerprint(slotId, fingerprint) {
  if (!slotId || !fingerprint) return false;
  if (slotId === `req:${fingerprint}`) return true;
  const marker = `:${fingerprint}`;
  return slotId.startsWith("req:") && slotId.endsWith(marker) && slotId.length > 4 + marker.length;
}

function buildSingleCourseSlotId(data, scope) {
  const fp = `S:${(data.possibilities || []).map((p) => slotScopeSlug(p)).join("/")}`;
  return scope ? `req:${scope}:${fp}` : `req:${fp}`;
}

function buildAnyOfCategorySlotId(category, scope) {
  const fp = `A:${slotScopeSlug(category)}`;
  return scope ? `req:${scope}:${fp}` : `req:${fp}`;
}

/** Find nested requirement that owns a schedule slot id. */
function matchesSlotId(req, slotId) {
  if (!req || !slotId) return false;
  const { type, data } = parseRequirement(req);
  if (isBusinessBreadthSlotId(slotId) && type === "AnyOf") {
    return isBusinessBreadthCategory(data.category);
  }
  if (type === "SingleCourse") {
    return slotIdMatchesScopedFingerprint(
      slotId,
      `S:${(data.possibilities || []).map((p) => slotScopeSlug(p)).join("/")}`,
    );
  }
  if (type === "AnyOf" && data.category && !isBusinessBreadthCategory(data.category)) {
    return slotIdMatchesScopedFingerprint(slotId, `A:${slotScopeSlug(data.category)}`);
  }
  if (type === "Restriction") {
    const rest = slotId.startsWith("req:") ? slotId.slice(4) : "";
    const scopeEnd = rest.indexOf(":R:");
    if (scopeEnd > 0) {
      const scope = rest.slice(0, scopeEnd);
      return buildRestrictionSlotId(data, scope) === slotId;
    }
    return buildRestrictionSlotId(data, null) === slotId;
  }
  return false;
}

export function findRequirementForSlotId(req, slotId) {
  if (!req || !slotId) return null;
  if (matchesSlotId(req, slotId)) return req;
  const { type, data } = parseRequirement(req);
  if (type === "AnyOf") {
    for (const child of data.possibilities || []) {
      const found = findRequirementForSlotId(child, slotId);
      if (found) return found;
    }
  }
  if (type === "AllOf" || type === "Concentration") {
    for (const child of data.requirements || []) {
      const found = findRequirementForSlotId(child, slotId);
      if (found) return found;
    }
  }
  if (type === "CourseGroup") {
    for (const child of data.possibilities || []) {
      const found = findRequirementForSlotId(child, slotId);
      if (found) return found;
    }
  }
  if (type === "CoursePool") {
    for (const child of data.fixed_slots || []) {
      const found = findRequirementForSlotId(child, slotId);
      if (found) return found;
    }
  }
  return null;
}

function slotScopeSlug(s) {
  return String(s).replace(/[^a-zA-Z0-9]/g, "_");
}

const BB_SLOT_FINGERPRINT = "BB:Business_Breadth";

/** Must stay in sync with Rust `business_breadth_slot_id`. */
export function businessBreadthSlotId(scope) {
  return scope ? `req:${scope}:${BB_SLOT_FINGERPRINT}` : `req:${BB_SLOT_FINGERPRINT}`;
}

function isBusinessBreadthSlotId(slotId) {
  return typeof slotId === "string" && slotId.includes(":BB:Business_Breadth");
}

/** Must stay in sync with Rust `requirement_slot_id(scope)`. */
function buildRestrictionSlotId(data, scope) {
  const dept = Array.isArray(data.department) ? data.department.join("/") : "";
  const attr = Array.isArray(data.attr) ? data.attr.join("/") : "";
  const excl = Array.isArray(data.excluding) ? data.excluding.join(",") : "";
  const lvl = data.level != null ? String(data.level) : "";
  const school = data.no_school || "";
  const fp = `R:${data.number ?? ""}:${dept}:${lvl}:${attr}:${excl}:${school}`;
  return scope ? `req:${scope}:${fp}` : `req:${fp}`;
}

function isBusinessBreadthCategory(category) {
  return typeof category === "string" && category.toLowerCase().includes("business breadth");
}

function businessBreadthScheduleLabel() {
  return "1 WH Business Breadth";
}

/** Business breadth slots use short labels like "WH Business Breadth". */
export function businessBreadthLabelForSlot(req, slotId) {
  if (!req || !slotId) return null;
  const { type, data } = parseRequirement(req);
  if (type === "AnyOf" && isBusinessBreadthCategory(data.category)) {
    if (isBusinessBreadthSlotId(slotId)) {
      return businessBreadthScheduleLabel();
    }
  }
  if (type === "AllOf" || type === "Concentration") {
    for (const child of data.requirements || []) {
      const label = businessBreadthLabelForSlot(child, slotId);
      if (label) return label;
    }
  }
  if (type === "CoursePool") {
    for (const child of data.fixed_slots || []) {
      const label = businessBreadthLabelForSlot(child, slotId);
      if (label) return label;
    }
  }
  return null;
}

/** Label for one side of a dual-degree overlap schedule block. */
export function formatOverlapMemberLabel(slotLabel, memberLabel) {
  const memberText = (memberLabel || "").split(/\n↳/)[0].trim();
  const resolved =
    slotLabel && slotLabel !== "Open requirement" ? slotLabel : memberText;
  let text = (resolved || "Open requirement").split(/\n↳/)[0].trim();

  if (/^\d+(\.\d+)? CU/.test(text)) {
    return text;
  }

  const looksLikeCourse = /^[A-Z]{2,6}\s+\d{4}$/.test(text);
  if (
    memberText
    && (looksLikeCourse || text === "One of the following options" || text.startsWith("One of:") || text === memberText)
  ) {
    return `1 CU from ${memberText}`;
  }

  return text;
}

/** Label for a schedule requirement slot card. */
export function getSlotLabel(req, slotId, apiLabels = {}) {
  const bbLabel = businessBreadthLabelForSlot(req, slotId);
  if (bbLabel) return bbLabel;
  const apiLabel = apiLabels[slotId];
  if (apiLabel && typeof apiLabel === "string" && !apiLabel.includes("[object Object]") && apiLabel !== "Open requirement") {
    return apiLabel;
  }
  const matched = findRequirementForSlotId(req, slotId);
  if (matched) {
    const { type, data } = parseRequirement(matched);
    if (type === "SingleCourse" && (data.possibilities?.length ?? 0) > 1) {
      return formatScheduleSingleCourseLabel(data.possibilities);
    }
    return createRequirementDescription(matched);
  }
  if (typeof apiLabel === "string") return apiLabel;
  return "Open requirement";
}

/** Stable row identity from API (preferred over description fingerprint). */
export function getRequirementInstanceId(mappedOrItem) {
  if (mappedOrItem?.instance_id) return mappedOrItem.instance_id;
  const req = mappedOrItem?.requirement ?? mappedOrItem;
  if (!req) return "unknown";
  const { type, data } = parseRequirement(req);
  if (type === "AnyOf" && isBusinessBreadthCategory(data.category)) {
    return businessBreadthSlotId(mappedOrItem?.instance_id);
  }
  return getRequirementLabel(req);
}
