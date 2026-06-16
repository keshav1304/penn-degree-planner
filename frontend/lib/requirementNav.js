import {
  isOverlapScheduleGroupId,
  isPoolConstraintInstanceId,
  isRequirementSlotId,
} from "@/lib/courseUtils";

/** DOM id for a requirement row (degree tab + instance id). */
export function reqRowDomId(degreeIndex, instanceId) {
  const safe = encodeURIComponent(String(instanceId ?? "")).replace(/%/g, "_");
  return `req-d${degreeIndex}-${safe}`;
}

/** Instance scope embedded in a `req:…` slot id (e.g. `1:f0:c0`). */
export function requirementSlotScope(slotId) {
  if (!isRequirementSlotId(slotId)) return null;
  const scope = slotId.slice(4).split(":R:")[0];
  return scope || null;
}

/** Slot refs encoded in `req:overlap:{degree}@{slot_key}+…`. */
export function parseOverlapGroupSlots(groupId) {
  if (!isOverlapScheduleGroupId(groupId)) return [];
  const body = groupId.slice("req:overlap:".length);
  if (!body) return [];
  return body
    .split("+")
    .map((part) => {
      const at = part.indexOf("@");
      if (at <= 0) return null;
      const degreeIndex = Number.parseInt(part.slice(0, at), 10);
      const slotKey = part.slice(at + 1);
      if (Number.isNaN(degreeIndex) || !slotKey) return null;
      return { degreeIndex, slotKey };
    })
    .filter(Boolean);
}

export function findMappedForInstance(result, instanceId) {
  if (!result || !instanceId) return null;
  const lists = [
    result.fulfilled_requirements,
    result.suggested_for_unfulfilled,
    result.unfulfilled_requirements,
  ];
  for (const list of lists) {
    const found = list?.find((m) => m.instance_id === instanceId);
    if (found) return found;
  }
  return null;
}

export function findMappedForSlot(result, slotId) {
  if (!result || !slotId) return null;
  const scope = requirementSlotScope(slotId);
  const lists = [
    result.fulfilled_requirements,
    result.suggested_for_unfulfilled,
    result.unfulfilled_requirements,
  ];
  for (const list of lists) {
    const found = list?.find(
      (m) =>
        m.course_ids?.includes(slotId)
        || (m.instance_id && slotId.startsWith(`req:${m.instance_id}:`))
        || (scope && m.instance_id === scope),
    );
    if (found) return found;
  }
  if (scope) return findMappedForInstance(result, scope);
  return null;
}

/** Stable instance id for a pool coverage constraint row (`21:c1`). */
export function poolConstraintInstanceId(poolIndex, constraintIndex) {
  return `${poolIndex}:c${constraintIndex}`;
}

/** Parse `{pool_index}:c{constraint_index}` overlap slot keys. */
export function parsePoolConstraintSlotKey(slotKey) {
  if (!slotKey || typeof slotKey !== "string") return null;
  const match = slotKey.match(/^(\d+):c(\d+)$/);
  if (!match) return null;
  return {
    poolIndex: Number.parseInt(match[1], 10),
    constraintIndex: Number.parseInt(match[2], 10),
  };
}

/** Nav target for a pool coverage constraint (WH LAS WUNM, CAS gen-ed pool tags, etc.). */
export function findPoolConstraintNav(result, slotKey) {
  const parsed = parsePoolConstraintSlotKey(slotKey);
  if (!parsed || !result?.pool_coverage_info) return null;
  const pool = result.pool_coverage_info.find(
    (p) => p.pool_index === parsed.poolIndex,
  );
  const constraint = pool?.constraints?.[parsed.constraintIndex];
  if (!pool || !constraint) return null;
  return {
    instanceId: poolConstraintInstanceId(parsed.poolIndex, parsed.constraintIndex),
    category: pool.category || "Other",
    rowLabel: constraint.description || constraint.label || pool.category,
  };
}

function categoryFromRequirement(req) {
  if (!req) return "Other";
  if (req.category) return req.category;
  for (const variant of [
    "SingleCourse",
    "CourseGroup",
    "AnyOf",
    "AllOf",
    "Concentration",
    "Restriction",
    "CoursePool",
  ]) {
    if (req[variant]?.category) return req[variant].category;
  }
  return "Other";
}

/**
 * Resolve schedule/requirements navigation for an overlap slot.
 * Pool constraints (`21:c1`) live in `pool_coverage_info`, not mapped requirement lists.
 */
export function resolveOverlapSlotNav(result, slotKey, scheduleSlotId) {
  const keys = [slotKey, scheduleSlotId ? requirementSlotScope(scheduleSlotId) : null]
    .filter(Boolean);
  for (const key of keys) {
    if (!isPoolConstraintInstanceId(key)) continue;
    const poolNav = findPoolConstraintNav(result, key);
    if (poolNav) return poolNav;
  }

  for (const key of keys) {
    const mapped = findMappedForInstance(result, key);
    if (mapped) {
      return {
        instanceId: mapped.instance_id ?? key,
        category: categoryFromRequirement(mapped.requirement),
        rowLabel: categoryFromRequirement(mapped.requirement),
        mapped,
      };
    }
  }

  if (scheduleSlotId) {
    const mapped = findMappedForSlot(result, scheduleSlotId);
    if (mapped) {
      return {
        instanceId: mapped.instance_id ?? requirementSlotScope(scheduleSlotId),
        category: categoryFromRequirement(mapped.requirement),
        rowLabel: categoryFromRequirement(mapped.requirement),
        mapped,
      };
    }
  }

  return null;
}

function overlapSlotKey(slot) {
  return `${slot.degree_index}:${slot.slot_key}`;
}

/** True when two overlap slot lists name the same open slots. */
export function overlapSlotsEqual(a, b) {
  if (!a?.length || !b?.length || a.length !== b.length) return false;
  const keys = new Set(a.map(overlapSlotKey));
  return b.every((s) => keys.has(overlapSlotKey(s)));
}

/** Build attribute code → fulfilled course ids from API payload. */
export function attributeFulfillmentMap(mappedOrItem) {
  const rows = mappedOrItem?.attribute_fulfillment;
  if (!Array.isArray(rows)) return new Map();
  const map = new Map();
  rows.forEach((row) => {
    if (row?.attribute && Array.isArray(row.course_ids)) {
      map.set(row.attribute, row.course_ids);
    }
  });
  return map;
}
