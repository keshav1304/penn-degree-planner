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
    "DoubleCount",
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

/** Must stay in sync with Rust `format_restriction_description`. */
export function formatRestriction(data) {
  let response = formatCuLabel(data.number, data.cu);
  if (data.department?.length) {
    const depts = Array.isArray(data.department) ? data.department : [data.department];
    response += ` from dept ${depts.join("/")}`;
  }
  if (data.level != null) {
    response += ` with minimum level ${data.level}`;
  }
  if (data.attr?.length) {
    const attrs = data.attr.filter((a) => typeof a === "string");
    if (attrs.length) response += ` from attribute ${attrs.join("/")}`;
  }
  if (data.excluding?.length) {
    const ex = data.excluding.filter((c) => typeof c === "string");
    if (ex.length) response += ` excluding ${ex.join(", ")}`;
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
    case "CourseGroup": {
      const prefix = `${data.number} CU from: `;
      return formatTruncatedList(data.possibilities, prefix);
    }
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
    case "DoubleCount": {
      const baseDescs = (data.base_requirements || []).map((sub) => {
        const desc = createRequirementDescription(sub);
        if (desc) return desc;
        const { data: subData } = parseRequirement(sub);
        return subData.category || "Requirement";
      });
      const dcDescs = (data.double_counting_requirements || []).map((sub) => {
        const desc = createRequirementDescription(sub);
        if (desc) return desc;
        const { data: subData } = parseRequirement(sub);
        return subData.category || "Requirement";
      });
      return `Take: ${baseDescs.join("; ")}. (${(data.double_counting_requirements || []).length} must also satisfy: ${dcDescs.join("; ")})`;
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

export function buildCourseCuMap(allCourses) {
  const map = {};
  (allCourses || []).forEach((c) => {
    if (c?.course_code != null) map[c.course_code] = c.cu ?? 1;
  });
  return map;
}

function isHalfCu(cu) {
  return Math.abs(cu - 0.5) < CU_EPS;
}

function lookupCourseCu(cuMap, courseId) {
  return cuMap[courseId] ?? 1;
}

/** Pick courses reaching target CU (mirrors Rust `select_courses_for_cu_target`, small n). */
export function selectCoursesForCuTarget(eligible, targetCu) {
  if (targetCu <= CU_EPS) return [];
  if (!eligible.length) return null;
  if (Math.abs(targetCu - 0.5) < CU_EPS) {
    const half = eligible.find(([, cu]) => isHalfCu(cu));
    return half ? [half[0]] : null;
  }
  const maxBits = Math.min(eligible.length, 14);
  const items = eligible.slice(0, maxBits);
  let best = null;
  for (let mask = 1; mask < 1 << maxBits; mask++) {
    const picked = [];
    let sum = 0;
    items.forEach(([course, cu], i) => {
      if (mask & (1 << i)) {
        sum += cu;
        picked.push([course, cu]);
      }
    });
    if (sum + CU_EPS < targetCu) continue;
    const overage = sum - targetCu;
    const hasHalf = picked.some(([, cu]) => isHalfCu(cu));
    const hasFull = picked.some(([, cu]) => !isHalfCu(cu));
    const mixed = hasHalf && hasFull;
    const courses = picked.map(([c]) => c);
    const count = courses.length;
    if (
      !best
      || overage < best.overage - CU_EPS
      || (Math.abs(overage - best.overage) < CU_EPS && count < best.count)
      || (Math.abs(overage - best.overage) < CU_EPS && count === best.count && !mixed && best.mixed)
    ) {
      best = { courses, overage, count, mixed };
    }
  }
  return best?.courses ?? null;
}

/** Courses on the schedule (taken or planned) that fulfill a restriction by accumulated CU. */
export function scheduleCoursesFulfillingRestriction(data, placedCourseIds, cuMap, attrMap) {
  const target = restrictionRequiredCu(data.number, data.cu);
  const eligible = [...placedCourseIds]
    .filter((id) => courseMeetsRestriction(id, data, attrMap))
    .map((id) => [id, lookupCourseCu(cuMap, id)]);
  return selectCoursesForCuTarget(eligible, target);
}

export function courseMeetsRestriction(courseId, data, attrMap) {
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
export function childMatchesAnyOfFulfillment(childReq, parent, parentFulfilled = parent?.fulfilled) {
  if (!parentFulfilled) return false;
  const courses = [];
  (parent.fulfilledCourses || []).forEach((c) => {
    if (c && typeof c === "string") courses.push(c);
  });
  parent.attributeFulfillment?.forEach((ids) => {
    ids.forEach((c) => { if (c && typeof c === "string") courses.push(c); });
  });
  const unique = [...new Set(courses)];
  if (unique.length === 0) return false;

  const attrMap = parent.attributeFulfillment;
  const { type, data } = parseRequirement(childReq);

  switch (type) {
    case "SingleCourse": {
      const allowed = new Set(data.possibilities || []);
      const matched = unique.filter((c) => allowed.has(c));
      return matched.length > 0 && matched.length === unique.length;
    }
    case "AllOf": {
      const pool = [...unique];
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
    case "CourseGroup": {
      const opts = data.possibilities || [];
      const matched = unique.filter((c) => opts.includes(c));
      return matched.length >= (data.number ?? 1);
    }
    case "Restriction":
      return unique.every((c) => courseMeetsRestriction(c, data, attrMap))
        && unique.some((c) => courseMeetsRestriction(c, data, attrMap));
    default:
      return false;
  }
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
      return `${data.number} CU from`;
    case "Restriction":
      return formatRestriction(data);
    case "AnyOf":
      if (data.possibilities?.length === 1) return getRequirementStem(data.possibilities[0]);
      return null;
    case "AllOf":
      return "Complete all";
    case "Concentration":
      return `Concentration (${data.number} CU)`;
    case "DoubleCount":
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
      return `${data.number ?? "N"} CU`;
    case "Concentration":
      return `Concentration (${data.number ?? "N"} CU)`;
    case "DoubleCount":
      return "Double-counted requirement";
    default:
      return "Requirement";
  }
}

/** Find nested requirement that owns a schedule slot id. */
function matchesSlotId(req, slotId) {
  if (!req || !slotId) return false;
  const { type, data } = parseRequirement(req);
  if (slotId.startsWith("req:BB:") && type === "AnyOf") {
    return isBusinessBreadthCategory(data.category) && businessBreadthSlotId(data.category) === slotId;
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
  if (type === "DoubleCount") {
    for (const child of [
      ...(data.base_requirements || []),
      ...(data.double_counting_requirements || []),
    ]) {
      const found = findRequirementForSlotId(child, slotId);
      if (found) return found;
    }
  }
  return null;
}

function slotScopeSlug(s) {
  return String(s).replace(/[^a-zA-Z0-9]/g, "_");
}

/** Must stay in sync with Rust `business_breadth_slot_id`. */
export function businessBreadthSlotId(category) {
  return `req:BB:${slotScopeSlug(category)}`;
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

function businessBreadthScheduleLabel(category) {
  if (category === "Business Breadth") return "1 WH Business Breadth";
  return `1 WH ${category}`;
}

/** Business breadth slots use short labels like "1 WH Business Breadth". */
export function businessBreadthLabelForSlot(req, slotId) {
  if (!req || !slotId) return null;
  const { type, data } = parseRequirement(req);
  if (type === "AnyOf" && isBusinessBreadthCategory(data.category)) {
    if (businessBreadthSlotId(data.category) === slotId) {
      return businessBreadthScheduleLabel(data.category);
    }
  }
  if (type === "AllOf" || type === "Concentration") {
    for (const child of data.requirements || []) {
      const label = businessBreadthLabelForSlot(child, slotId);
      if (label) return label;
    }
  }
  if (type === "DoubleCount") {
    for (const child of [
      ...(data.base_requirements || []),
      ...(data.double_counting_requirements || []),
    ]) {
      const label = businessBreadthLabelForSlot(child, slotId);
      if (label) return label;
    }
  }
  return null;
}

/** Label for a schedule requirement slot card. */
export function getSlotLabel(req, slotId, apiLabels = {}) {
  if (apiLabels[slotId] && typeof apiLabels[slotId] === "string" && !apiLabels[slotId].includes("[object Object]")) {
    return apiLabels[slotId];
  }
  const bbLabel = businessBreadthLabelForSlot(req, slotId);
  if (bbLabel) return bbLabel;
  const matched = findRequirementForSlotId(req, slotId);
  if (matched) return getRequirementLabel(matched);
  if (typeof apiLabels[slotId] === "string") return apiLabels[slotId];
  return "Open requirement";
}

/** Stable row identity from API (preferred over description fingerprint). */
export function getRequirementInstanceId(mappedOrItem) {
  if (mappedOrItem?.instance_id) return mappedOrItem.instance_id;
  const req = mappedOrItem?.requirement ?? mappedOrItem;
  if (!req) return "unknown";
  const { type, data } = parseRequirement(req);
  if (type === "AnyOf" && isBusinessBreadthCategory(data.category)) {
    return businessBreadthSlotId(data.category);
  }
  return getRequirementLabel(req);
}
