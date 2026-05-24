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
      return "One of the following options";
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
