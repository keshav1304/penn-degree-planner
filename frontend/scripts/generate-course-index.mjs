/**
 * Build slim course search index from embedded Rust catalog (courses.json).
 * Run: npm run generate:catalog
 */
import { readFileSync, writeFileSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const coursesPath = join(root, "degree_planner/src/penn_data/courses.json");
const outPath = join(root, "frontend/public/course_index.json");

const courses = JSON.parse(readFileSync(coursesPath, "utf8"));
const slim = courses.map((c) => ({
  dept_code: c.dept_code,
  course_code: c.course_code,
  title: c.title,
  cu: c.cu,
}));

writeFileSync(outPath, JSON.stringify(slim));
console.log(`Wrote ${slim.length} courses (${(Buffer.byteLength(JSON.stringify(slim)) / 1024).toFixed(0)} KB) → ${outPath}`);
