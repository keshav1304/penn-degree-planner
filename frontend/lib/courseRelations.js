/** Client-side also_offered / mutex helpers from the slim course index. */

export function normalizeCode(code) {
  if (!code || typeof code !== "string") return "";
  return code.replace(/\u00a0/g, " ").trim().replace(/\s+/g, " ");
}

function parseCodeList(raw) {
  if (!raw || typeof raw !== "string") return [];
  return raw
    .split(",")
    .map((s) => normalizeCode(s))
    .filter(Boolean);
}

/**
 * @param {Array<{ course_code: string, also_offered_as?: string|null, mutually_exclusive?: string|null }>} rows
 */
export function buildCourseRelations(rows) {
  /** @type {Map<string, Set<string>>} */
  const aliasEdges = new Map();
  const ensure = (code) => {
    const n = normalizeCode(code);
    if (!n) return null;
    if (!aliasEdges.has(n)) aliasEdges.set(n, new Set([n]));
    return n;
  };

  for (const row of rows || []) {
    const self = ensure(row.course_code);
    if (!self) continue;
    for (const other of parseCodeList(row.also_offered_as)) {
      const o = ensure(other);
      if (!o) continue;
      aliasEdges.get(self).add(o);
      aliasEdges.get(o).add(self);
    }
  }

  // Union-find style cluster expansion
  /** @type {Map<string, string>} */
  const canonicalOf = new Map();
  const visited = new Set();
  for (const start of aliasEdges.keys()) {
    if (visited.has(start)) continue;
    const stack = [start];
    const component = [];
    while (stack.length) {
      const cur = stack.pop();
      if (visited.has(cur)) continue;
      visited.add(cur);
      component.push(cur);
      for (const n of aliasEdges.get(cur) || []) {
        if (!visited.has(n)) stack.push(n);
      }
    }
    component.sort();
    const canon = component[0];
    for (const c of component) canonicalOf.set(c, canon);
  }

  /** @type {Map<string, string[]>} */
  const mutex = new Map();
  const addMutex = (a, b) => {
    if (a === b) return;
    if (!mutex.has(a)) mutex.set(a, []);
    if (!mutex.get(a).includes(b)) mutex.get(a).push(b);
  };

  for (const row of rows || []) {
    const self = normalizeCode(row.course_code);
    if (!self) continue;
    for (const other of parseCodeList(row.mutually_exclusive)) {
      addMutex(self, other);
      addMutex(other, self);
    }
  }

  // Close mutex under also-offered clusters.
  const closed = new Map();
  for (const code of new Set([...mutex.keys(), ...canonicalOf.keys()])) {
    const partners = new Set();
    const cluster = [...canonicalOf.entries()]
      .filter(([, c]) => c === (canonicalOf.get(code) || code))
      .map(([k]) => k);
    const seeds = cluster.length ? cluster : [code];
    for (const s of seeds) {
      for (const p of mutex.get(s) || []) {
        partners.add(p);
        const pc = canonicalOf.get(p) || p;
        for (const [k, c] of canonicalOf.entries()) {
          if (c === pc) partners.add(k);
        }
      }
    }
    partners.delete(code);
    if (partners.size) closed.set(code, [...partners].sort());
  }

  return {
    canonical(code) {
      const n = normalizeCode(code);
      return canonicalOf.get(n) || n;
    },
    equivalent(a, b) {
      return this.canonical(a) === this.canonical(b);
    },
    /** True if list already has an equivalent code. */
    listContainsEquiv(list, code) {
      return (list || []).some((c) => this.equivalent(c, code));
    },
    mutexPartners(code) {
      const n = normalizeCode(code);
      return closed.get(n) || [];
    },
    codesConflict(a, b) {
      if (this.equivalent(a, b)) return false;
      const nb = normalizeCode(b);
      return this.mutexPartners(a).some((p) => p === nb || this.equivalent(p, nb));
    },
    /**
     * Factual mutex messages for codes that both appear on the grid (schedule + Credits).
     * @param {string[]} gridCodes
     * @returns {Record<string, string>}
     */
    mutexViolationsOnGrid(gridCodes) {
      const codes = [...new Set((gridCodes || []).map(normalizeCode).filter(Boolean))];
      /** @type {Record<string, string>} */
      const out = {};
      const seen = new Set();
      for (const a of codes) {
        for (const partner of this.mutexPartners(a)) {
          const b = codes.find((c) => c === partner || this.equivalent(c, partner));
          if (!b || this.equivalent(a, b)) continue;
          const key = [this.canonical(a), this.canonical(b)].sort().join("|");
          if (seen.has(key)) continue;
          seen.add(key);
          const message = `${a} is mutually exclusive with ${b}.`;
          out[a] = out[a] || message;
          out[b] = out[b] || message;
        }
      }
      return out;
    },
    /**
     * When two+ also-offered spellings appear in the plan, warn they are the same course.
     * @param {string[]} planCodes
     * @returns {Record<string, string>}
     */
    alsoOfferedDuplicatesInPlan(planCodes) {
      const codes = [...new Set((planCodes || []).map(normalizeCode).filter(Boolean))];
      /** @type {Map<string, string[]>} */
      const byCanon = new Map();
      for (const code of codes) {
        const canon = this.canonical(code);
        if (!byCanon.has(canon)) byCanon.set(canon, []);
        const list = byCanon.get(canon);
        if (!list.includes(code)) list.push(code);
      }
      /** @type {Record<string, string>} */
      const out = {};
      for (const spellings of byCanon.values()) {
        if (spellings.length < 2) continue;
        const ordered = [...spellings].sort();
        const message =
          ordered.length === 2
            ? `${ordered[0]} and ${ordered[1]} are the same course.`
            : `${ordered.slice(0, -1).join(", ")}, and ${ordered[ordered.length - 1]} are the same course.`;
        for (const code of ordered) out[code] = message;
      }
      return out;
    },
  };
}
