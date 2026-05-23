/** DOM id for a requirement row (degree tab + instance id). */
export function reqRowDomId(degreeIndex, instanceId) {
  const safe = encodeURIComponent(String(instanceId ?? "")).replace(/%/g, "_");
  return `req-d${degreeIndex}-${safe}`;
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
