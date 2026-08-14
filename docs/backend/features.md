# Backend features

How major backend features work, which modules implement them, and why certain choices were made.

Pair with the [module map](./modules.md) (where code lives) and [Domain](../domain.md) (vocabulary). Paths below are under `degree_planner/src/` unless noted.

---

## 1. Embedded catalog

**What:** The full course list (and CU map, attributes) ships inside the API binary.

**Where:** `penn_data/courses_data.rs`, `penn_data/attributes_data.rs`, `course.rs`.

**How:** JSON is `include_str!`’d and parsed once into process-wide `OnceLock` data. Search and CU lookups read those statics.

**Why:**
- Degree logic needs fast, local access to thousands of courses on every generate.
- Avoids a second runtime dependency for “what exists in the catalog.”
- Deploy stays a single binary on a small VM.

**Tradeoff:** Catalog updates mean rebuilding/redeploying the API (and regenerating the frontend slim index). That is accepted for correctness and speed.

---

## 2. Course identity (aliases and mutex)

**What:** Treat also-offered-as spellings as one course; flag mutually exclusive pairs.

**Where:** `course_relations.rs`; used from `requirement`, `scheduler`, `cross_degree`, `overlap_planner`, `prereq`.

**How:**
- Normalize codes, build alias clusters, pick a canonical (lex-min).
- When assigning a course to a slot, remove all equivalents from the remaining taken pool.
- Mutex checks are closed under aliases.

**Why:**
- Students and Penn listings use different codes for the same offering. Counting both would understate remaining work and overstate CU.
- Mutex is a hard academic constraint and must show up as a plan violation, not a silent double-count.

**Choice:** Canonicalize at the edges of validation/scheduling rather than rewriting every authored requirement list to one spelling. Trees can list any accepted code; equivalence handles the rest.

---

## 3. Authoring degree trees

**What:** Each major/minor is a nested `Requirement` tree written in Rust.

**Where:** `penn_data/*_data.rs`, helpers in `requirement_builders.rs`, registration in `major.rs`.

**How:** School modules construct `Major { requirements, concentrations, schedule_hints }`. `resolve_major` / `resolve_minor` dispatch by school + API code + concentrations and cache the result. Catalogs only expose programs that pass the “implemented” check (no placeholder-only stubs).

**Why:**
- Penn rules are irregular. A small DSL enum (`SingleCourse`, `AnyOf`, `Restriction`, `CoursePool`, …) is expressive enough without a separate rules language.
- Keeping trees in code gives reviewable diffs, tests, and compile-time structure.
- Builders reduce noise and keep Restriction fields consistent.

**Choice:** Not a database of rules. Editing a major is a code change. That favors correctness and versioning over runtime CMS-style editing.

See also [Adding a degree](../flows/adding-a-degree.md).

---

## 4. Requirement mapping (audit)

**What:** Given taken (and frozen course) codes, mark which slots are fulfilled, partial, or open.

**Where:** `requirement::validate_courses_for_degree` and helpers; orchestrated from `scheduler::generate_schedule` (and legacy `POST /`).

**How (simplified):**
1. Expand multi-CU restrictions into unit slots.
2. Sort a working copy by **fill order** (specific before broad; WRIT early; business breadth last) but keep **original indices** for `instance_id`.
3. For each slot, try to consume matching courses from a mutable taken pool (alias-aware).
4. Composites (`AllOf` / `AnyOf` / …) recurse; `AnyOf` can commit to a best-matching branch when partially filled.
5. Pools fill fixed then flex, then evaluate coverage constraints on the pool’s courses.
6. Emit fulfilled / unfulfilled mapped requirements + pool coverage info.

**Why fill order exists:** Broad electives will happily absorb courses that a specific core slot needed. Matching specific slots first keeps the audit closer to how advisors think.

**Why greedy, not ILP:** Full global optimization over every Penn constraint is slow and hard to explain. Greedy ordered assignment is fast enough for interactive regenerate and produces stable, debuggable results. Edge cases get special cases (CAS pools, cross-degree reconciliation) rather than a general solver.

**Why instance ids:** The UI freezes and navigates slots across regenerates. Ids must not depend on sort order or description text.

---

## 5. Suggestions and schedule placeholders

**What:** For open slots, propose concrete courses or `req:…` placeholders the scheduler can place.

**Where:** `requirement::suggest_courses_for_requirements`; id helpers in `requirement.rs`; packing in `scheduler.rs`.

**How:** Walk unfulfilled mapped requirements (skipping pool **coverage** instances). Ask each requirement for suggestions given taken courses and optional cross-degree filters. Filter to schedulable ids only.

**Why placeholders exist:** Many slots are Restriction-shaped (“any CIS 1000+”). Until the student picks a course, the grid still needs a CU-sized block with a stable id and a label (`slot_labels`).

**Why coverage constraints are not suggested as CU:** They are labels over courses already in the pool, not extra courses to take. Putting them on the grid would double-count work. See [Identifiers](../ids.md).

---

## 6. Course pools

**What:** One shared bucket of courses that must satisfy fixed slots, flexible CU, and labeled coverage rules (CAS gen-eds, Wharton LAS-style pools, etc.).

**Where:** `Requirement::CoursePool` authoring in school data; evaluation in `requirement.rs` (`evaluate_pool_constraints`, `evaluate_cas_pool_constraints`, flex fill helpers).

**How:**
- Fixed slots consume specific sub-requirements first; courses join `pool_courses`.
- Flexible slots pull remaining eligible courses into the bucket.
- Coverage units expand by `count`, prioritize specific attributes over broad catch-alls, and mark courses used.
- Default: a course may satisfy at most **two** coverage constraints across the pool; **consumption groups** still block reuse within the same group (e.g. CAS foundational vs sector groupings).
- CAS has a dedicated evaluator for sector/FA nuances (including major↔sector double-count limits).

**Why pools instead of flat Restrictions only:** Penn often requires “these courses together must cover X and Y,” not only “N isolated electives.” Pools model the bucket + coverage story without fake extra CU.

**Why a double-count cap:** Matches common Penn practice that one course can stretch across a limited number of overlapping labels, not every label at once.

---

## 7. Cross-degree sharing and violations

**What:** When multiple programs are selected, decide how courses may be claimed, what breaks the rules, and what summary the UI shows.

**Where:** `cross_degree.rs`; claim repair / filtering in `requirement::resolve_cross_degree_conflicts` and related filters; orchestration in `scheduler.rs`.

**How:**
- Track claims: course (canonical) → set of degree indices.
- `can_claim` enforces caps (e.g. at most two degrees; grad↔grad restrictions; undergrad↔grad CU budget).
- After independent per-degree validation, reconcile conflicts: drop weaker claims for `TooManyDegrees` / `GradGradOverlap`, account for CU cap, surface mutex / also-offered / prereq issues.
- Build `CrossDegreeSummary` (allocations + violations) for the panel.

**Why validate per degree first, then reconcile:** Each tree stays understandable in isolation. Sharing policy is layered on top instead of a single mega-tree that mixes schools.

**Why a 3.0 CU undergrad↔grad cap:** Encodes a planner rule for limited dual-counting across levels; enforced as budget in `CrossDegreeState`, not only as a soft UI hint.

**Why distinguish optimizer vs overlap applicability:** Undergrad duals get deeper sharing / paired-block behavior. Grad mixes still need overlap *visibility* and CU/violation checks without applying undergrad-only optimizers blindly.

---

## 8. Overlap discovery

**What:** Find courses (or paired open slots) that can advance requirements on more than one degree at once.

**Where:** `overlap_planner.rs`; consumed when building schedule groups and hints in `scheduler.rs`.

**How:**
1. Extract open slots from per-degree validation + major context; compile each to a `CourseMatcher`.
2. Use attribute/dept indexes to get candidate sets (capped in size).
3. Invert: course → slots it can fill; group slots that share courses across degrees.
4. Emit an `OverlapPlan` and optionally `overlap_schedule_groups` (`req:overlap:…`) so the grid shows one CU for a paired pair.

**Why index the catalog:** Restriction slots like “any course with attribute X” are huge if scanned naively every time. Inverted indexes keep dual-degree planning interactive.

**Why cap candidates:** Completeness is less important than returning useful, computable overlaps. Truncation is explicit.

**Why not fold this into mapping:** Mapping answers “what do I already satisfy?” Overlap answers “what single future course helps two open slots?” Different question, different stage.

Unconstrained electives and pool flex slots are still overlap-eligible, so the 800-course cap does not keep dual-degree generates interactive (often 15–35 s). See [Overlap performance](./overlap-performance.md) for the current search, three proposed cuts, UI/CU tradeoffs, and why that has not been changed yet.

---

## 9. Schedule packing

**What:** Lay courses and open slot ids across years/semesters under CU limits, respecting pins and hints.

**Where:** `scheduler::generate_schedule`; defaults in the same file; hints from `schedule_template.rs` / major data.

**How (conceptual):**
1. Normalize taken/frozen; build validation course set.
2. Resolve each degree; validate; cross-degree reconcile; suggest; compute overlap.
3. Place frozen items at their year/semester.
4. Place remaining suggested courses and schedulable `req:` ids using hints (fixed vs flexible), CU caps, summer flag, and dual-degree year policy.
5. Expand semester list until items fit (within policy).
6. Return schedule + per-degree results + labels + overlap metadata in payload degree order.

**CU policy choices:**
- Default semester caps; tighter caps for dual undergrad (non-all-CAS); summers optional and capped separately.
- Dual undergrad across schools often uses a **5-year** undergrad grid so load stays realistic; dual CAS stays on a 4-year shape with different sharing rules.
- Year remapping helpers exist so dual plans do not front-load like a single major.

**Why pins are first-class:** The UI is an editor. Regenerating must not rip out courses the student locked. Frozen slot ids keep open electives parked while the student decides.

**Why one API for panel + grid:** Two endpoints would drift (different taken sets, different overlap). One generate keeps audit and layout coherent.

---

## 10. Schedule hints (templates)

**What:** Conventional term targets for cores and sequenced requirements.

**Where:** Authored with majors via `schedule_template.rs` helpers; applied in `scheduler.rs`.

**How:** Hints map requirement index or course code → `(year, semester, Fixed|Flexible)`. Flexible may slide earlier if capacity exists; Fixed holds unless the user freezes elsewhere.

**Why:** Without hints, packing is CU-feasible but academically unordered (senior labs in year 1). Templates encode department norms without hard-coding every major inside the scheduler.

---

## 11. Prerequisite warnings

**What:** Flag planned courses whose catalog prereqs are not met by the rest of the plan.

**Where:** `prereq.rs`; surfaced through cross-degree violation machinery.

**How:** Parse prereq strings into `And`/`Or` trees over codes; evaluate against plan codes with alias awareness.

**Why warn-only:** Catalog prereq fields are inconsistent free text. Hard-blocking would reject many valid plans and fight the parser. Warnings still help students notice gaps.

**Choice:** Do not try to model every departmental exception in the parser; improve messages and coverage incrementally.

---

## 12. Minors vs majors

**What:** Minors use the same `Requirement` engine with `kind: "minor"` and a separate catalog/resolver.

**Where:** `major.rs` (`minor_catalog`, `resolve_minor`, …); `DegreeInput.kind` in `scheduler.rs`; some filters in `requirement.rs` for minor mapped rows.

**Why same engine:** A minor is still a tree of slots. Separate codepaths would duplicate bugs. Kind mainly affects catalog, labeling, and a few allocation/display filters.

---

## 13. Concentrations

**What:** Optional specialization blocks under a program.

**Where:** Authored on `Major` (embedded `Concentration` nodes and/or concentration maps); resolved with concentration args in `major.rs`; info extraction and claim helpers in `requirement.rs`.

**How:** Selected concentration names choose which subtrees apply. UI gets `concentration_info` / available lists. Cross-degree logic can treat some concentration courses as stronger claims when repairing conflicts.

**Why:** Concentrations change the tree shape; they are not a post-filter on an identical audit. Building them into resolve keeps validation honest.

---

## 14. HTTP API and analytics

**What:** Axum exposes catalog + generate; optional Postgres records compact generate summaries.

**Where:** `main.rs`, `analytics.rs`, `bin/analytics_report.rs`.

**Why thin HTTP:** All product complexity is library-testable without spinning a server.

**Why analytics are optional and compact:** Local and prod should plan without a DB. Events store counts, combo keys, violation types, latency, anon session id: enough for product questions without storing full schedules by default.

---

## Feature → primary files

| Feature | Primary files |
| --- | --- |
| Catalog embed / search | `penn_data/courses_data.rs`, `course.rs` |
| Aliases / mutex | `course_relations.rs` |
| Tree authoring | `penn_data/*`, `requirement_builders.rs`, `major.rs` |
| Mapping / pools / suggestions | `requirement.rs` |
| Cross-degree policy | `cross_degree.rs`, `requirement.rs` (reconcile) |
| Overlap | `overlap_planner.rs` |
| Packing / CU policy | `scheduler.rs`, `schedule_template.rs` |
| Prereqs | `prereq.rs` |
| API / analytics | `main.rs`, `analytics.rs` |

When behavior surprises you, ask: is this a **rule** (school data), an **interpreter** choice (`requirement`), a **sharing** policy (`cross_degree` / overlap), or a **layout** choice (`scheduler`)? That split usually finds the right file quickly.
