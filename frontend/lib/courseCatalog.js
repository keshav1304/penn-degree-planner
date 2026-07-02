/** @typedef {{ dept_code: string, course_code: string, title: string, cu: number }} CourseIndexRow */

const DEFAULT_SEARCH_LIMIT = 50;
const MAX_SEARCH_LIMIT = 100;

/**
 * Pre-lowercase searchable fields once at load time.
 * @param {CourseIndexRow[]} rows
 */
export function prepareCourseCatalog(rows) {
  return rows.map((row) => ({
    dept_code: row.dept_code,
    course_code: row.course_code,
    title: row.title,
    cu: row.cu,
    _lc: {
      code: row.course_code.toLowerCase(),
      title: row.title.toLowerCase(),
      dept: row.dept_code.toLowerCase(),
    },
  }));
}

/** @param {ReturnType<typeof prepareCourseCatalog>} catalog */
export function cuMapFromCatalog(catalog) {
  /** @type {Record<string, number>} */
  const map = {};
  for (const row of catalog) {
    map[row.course_code] = row.cu;
  }
  return map;
}

/**
 * @param {ReturnType<typeof prepareCourseCatalog>} catalog
 * @param {string} query
 * @param {number} [limit]
 */
export function searchCourses(catalog, query, limit = DEFAULT_SEARCH_LIMIT) {
  const q = query.trim().toLowerCase();
  if (!q || !catalog?.length) return [];

  const cap = Math.min(Math.max(limit, 1), MAX_SEARCH_LIMIT);
  const hits = [];
  for (const course of catalog) {
    const { _lc } = course;
    if (
      _lc.code.includes(q) ||
      _lc.title.includes(q) ||
      _lc.dept.includes(q)
    ) {
      hits.push({
        dept_code: course.dept_code,
        course_code: course.course_code,
        title: course.title,
        cu: course.cu,
      });
      if (hits.length >= cap) break;
    }
  }
  return hits;
}
