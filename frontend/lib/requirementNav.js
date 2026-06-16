import { isOverlapScheduleGroupId, isRequirementSlotId } from "@/lib/courseUtils";

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
