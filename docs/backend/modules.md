# Backend modules

File-by-file map of the `degree_planner` Rust crate (`degree_planner/src/`).

For *what* a feature does and *why* it was built that way, see [Features](./features.md). This page is the map of *where* the code lives and what each file is responsible for.

Read [Domain](../domain.md) first if the vocabulary is new.

## How the crate is split

| Piece | Path | Role |
| --- | --- | --- |
| Library | `lib.rs` | Declares modules; re-exports `Course`, `Major`, `Requirement`. |
| HTTP binary | `main.rs` | Axum routes, CORS, optional Postgres pool. |
| Analytics CLI | `bin/analytics_report.rs` | Local reports from analytics tables. |

**Principle:** academic rules live in the library. `main.rs` parses JSON, calls one library function, returns JSON. If you find degree logic in a handler, move it.

---

## `lib.rs`

Thin module list. Nothing else belongs here.

---

## `main.rs`

HTTP surface for the library.

**Owns:** route table, request/response DTOs that are transport-only, CORS, wiring `DATABASE_URL` into analytics.

**Does not own:** requirement trees, mapping, packing. Handlers should call `resolve_major`, `validate_courses_for_degree`, `generate_schedule`, catalog getters, and so on.

**Principle:** keep DTOs aligned with what the frontend already sends. Prefer extending `ScheduleInput` / `ScheduleOutput` over inventing a parallel API for the same job.

Main routes: catalog GETs, legacy `POST /` (single-degree validate), `POST /generate_schedule` (full path the UI uses).

---

## `course.rs`

The catalog course type and small helpers around it.

**Owns:**
- `Course` fields (code, title, CU, prereq text, also-offered-as, mutually exclusive, coreq, …)
- `is_valid_course_code` (dept + space + digits)
- graduate-level helpers used by cross-degree rules
- `/search_courses` hit shaping and in-memory search over the embedded catalog

**Principle:** a course code string is the universal id. Almost every other module assumes that shape. Do not invent a second identity scheme here.

Catalog *storage* is in `penn_data/courses_data.rs`; this file is the type and the operations on codes.

---

## `course_relations.rs`

Also-offered-as clusters and mutual exclusion.

**Owns:**
- normalize codes (NBSP → space, collapse whitespace)
- canonical code per alias cluster (lex-min of the cluster)
- `equivalent`, `aliases`, `codes_conflict`
- set/list helpers that treat aliases as the same course (`set_contains_equiv`, `retain_without_equiv`, …)

Built once from the embedded catalog (`OnceLock`).

**Principles:**
1. Counting the same offering twice under two spellings is a bug. Canonicalize before claiming or de-duplicating.
2. Mutex is closed under aliases: if A mutex B and B is also C, A conflicts with C.
3. Equivalent courses are *not* mutex with each other.

Used heavily by validation (consume one alias from the taken pool), scheduling, overlap, and violation detection.

---

## `prereq.rs`

Parse messy catalog prereq *text* into boolean expressions over course codes, then check plans.

**Owns:**
- tokenizer / parser (`AND`, `OR`, parentheses, implicit AND between adjacent codes)
- per-course expression map built once from catalog `prereq` fields
- “missing prereq” detection for courses on a plan relative to the rest of the plan

**Principles:**
1. **Warn-only.** Missing prereqs become `MissingPrerequisite` violations (or similar messaging). They do not block schedule generation. Catalog text is too noisy to hard-fail on.
2. Best-effort parse. Trailing prose after a usable expression is ignored when possible.
3. Equivalence from `course_relations` applies when checking whether a prereq code is satisfied.

---

## `penn_data/`

Embedded Penn data and authored degree trees. This directory is the source of truth for *what* each program requires.

### `courses_data.rs`

`include_str!("courses.json")`, deserialized once into `Vec<Course>`, plus a CU map.

**Principle:** the API process carries the catalog in-process. No runtime DB for course rows. Update the JSON (and regenerate the frontend index if needed) when the catalog changes.

### `attributes_data.rs`

Attribute → course list (and lookups the other way as needed). Powers Restriction `attr` matching and pool coverage that keys off tags (writing, Wharton tags, and so on).

### `requirement_builders.rs`

Fluent helpers so school files do not hand-write every `Requirement::Restriction { … }` struct.

**Principle:** if a slot shape repeats (unrestricted elective, dept+level elective, common pools), add a builder here and use it from school modules. Keeps trees readable and consistent.

### School modules

| File | School / programs |
| --- | --- |
| `college_data.rs` | CAS majors, shared gen-ed / pool patterns, CAS CU constants |
| `seas_data.rs` | SEAS undergrad majors + schedule templates |
| `seas_grad_data.rs` | SEAS master’s (`SEAS_MS`) |
| `wharton_data.rs` | Wharton tracks (language / no-language, MT variants), LAS pools, concentrations |
| `nursing_data.rs` | BSN / nutrition tracks, often with semester-shaped templates |

**Principles:**
1. Author trees in Rust using the `Requirement` enum. Do not encode degree rules in the frontend.
2. Prefer builders and shared pool helpers over copy-paste.
3. Attach schedule hints when a program has a conventional term order (`schedule_template` helpers).
4. Placeholders (`(placeholder)` categories) mark unfinished programs; `major.rs` hides them from catalogs.

When a major’s *rules* are wrong, edit the matching school file first. When the *engine* mis-applies a correct tree, edit `requirement.rs` / `scheduler.rs`.

---

## `major.rs`

Registry and resolution layer between API codes and authored `Major` values.

**Owns:**
- `Major` struct: names, requirement tree, optional concentration map, `schedule_hints`
- `degree_catalog` / `minor_catalog` for the UI
- `resolve_major` / `resolve_minor` (build + cache by school + code + concentrations)
- concentration lists
- `major_is_implemented` / `major_has_authored_requirements` (stub filter)

**Principles:**
1. The UI only lists implemented programs (real trees, not placeholders).
2. Resolution is cached; building large trees repeatedly on every generate is wasteful.
3. School-specific `create_*_major` functions live in `penn_data`; this file only dispatches and normalizes.

---

## `requirement.rs`

Largest and most important domain file. Interprets requirement trees.

### What it owns

1. **`Requirement` enum** and related types (`PoolConstraint`, `MappedRequirement`, pool coverage info, concentration info).
2. **Expansion** before matching (e.g. multi-CU Restriction → multiple 1-CU slots).
3. **Validation / mapping** (`validate_courses_for_degree`): consume taken courses into slots, assign `instance_id`s, mark partial / AnyOf branch commitment.
4. **Fill order** (specific slots before broad ones; WRIT siloed early; business breadth last).
5. **Pools:** fixed/flex fill, coverage evaluation, double-count limit, consumption groups, CAS-specific pool evaluation.
6. **Suggestions** for open slots (`suggest_courses_for_requirements`), including `req:` placeholders; skips coverage-constraint instances as non-schedulable.
7. **Id helpers** for schedule placeholders and pool constraint vs flex distinction (see [Identifiers](../ids.md)).
8. **Cross-degree reconciliation helpers** used after per-degree validation (`resolve_cross_degree_conflicts`, allocation filters, CAS double-major claim reconciliation, concentration claim merges).

### Core principles

1. **Trees are data; this file is the interpreter.** School modules must not reimplement matching.
2. **Greedy + ordered.** Courses are assigned in fill-order, removed from the remaining taken pool (with alias awareness). This is intentional: exact global optimization of every Penn rule set is not the goal; stable, explainable assignment is.
3. **Instance ids are stable by original tree index**, even when matching sorts a working copy. The UI depends on that stability across regenerates.
4. **Pools separate “slots” from “coverage.”** Filling flex/fixed consumes CU; coverage constraints are labels over the same bucket and must not become extra schedule CU.
5. **Suggestions are schedule inputs**, not a second audit. Coverage constraints are skipped when suggesting placements.

If mapping looks wrong: confirm the authored tree, then read fill order, expansion, and the relevant `fulfills` / pool path here.

---

## `cross_degree.rs`

Policy for more than one selected program.

**Owns:**
- applicability flags (`cross_degree_optimizer_applicable`, `overlap_plan_applicable`, dual-undergrad / all-CAS helpers)
- undergrad↔grad shared CU cap (`UNDERGRAD_GRAD_CU_LIMIT`, currently 3.0)
- `CrossDegreeState` (claims, grad course ownership, running CU, concentration course claims)
- `can_claim` / claim updates
- violation kinds and detection (`TooManyDegrees`, `GradGradOverlap`, `UndergradGradCuCap`, mutex, also-offered duplicates, missing prereqs)
- summary types returned to the UI

**Principles:**
1. **Overlap discovery** can run whenever two or more degrees are selected (including undergrad↔grad).
2. The **fuller optimizer** (aggressive sharing / paired blocks / allocation filtering for undergrad duals) applies only when every selected degree is undergraduate.
3. A course generally cannot serve more than two degrees; grad-level courses have tighter grad↔grad rules.
4. Violations are reported in the summary; some are repaired by stripping claims in `requirement::resolve_cross_degree_conflicts`, others remain as warnings (e.g. prereqs).

---

## `overlap_planner.rs`

Finds *opportunities* for one course (or a paired block) to advance open slots on different degrees.

**Owns:**
- catalog indexes (by attribute, by department) for fast Restriction candidate sets
- `CourseMatcher` compiled from open requirement slots
- `compute_overlap_plan`: open slots → candidate courses → groups of slots shareable by one course
- overlap schedule group ids (`req:overlap:…`) and member metadata for the grid
- remapping helpers when degree indices are compacted vs payload order

**Principles:**
1. Work from **open** slots after validation, not from the full tree every time.
2. Cap candidate set sizes so dual-degree Restriction pairs stay computable.
3. Broad/unrestricted matchers are handled carefully (often via peer candidates) so the planner does not explode into “every undergrad course.”
4. Overlap plan is input to the scheduler’s placement / grouping; it does not replace requirement validation.

---

## `schedule_template.rs`

Preferred term placement for authored majors.

**Owns:**
- `ScheduleHint` / `ScheduleHintMode` (`Fixed` vs `Flexible`)
- semester constants (`Y1F`, `Y2S`, …)
- helpers: `scheduled`, `append_semester`, `insert_fixed_course_hints`, …

**Principles:**
1. **Fixed:** only user freeze should override (used for must-land-here courses).
2. **Flexible:** prefer the template term; may backfill earlier open capacity, then at-or-after the target.
3. Hints key off requirement index strings and/or course codes on the `Major`.

School data builds hints; `scheduler.rs` consumes them while packing.

---

## `scheduler.rs`

End-to-end `generate_schedule`.

**Owns:**
- `ScheduleInput` / `ScheduleOutput` / `SemesterPlan` / `DegreeResult`
- input normalization (valid codes, alias de-dupe, frozen courses vs frozen slot ids)
- orchestration: resolve degrees → validate each → cross-degree resolve → suggestions → overlap plan → pack semesters
- CU policy defaults (`default_semester_cu_limit`, `undergrad_schedule_years`, dual-undergrad 5-year stretch, summer limits)
- CAS dual-major schedule quirks (shared gen-ed handling, excess unrestricted filtering, overlap group labeling)
- assembling results back into **payload degree order** so the UI index stays aligned

**Principles:**
1. **One response drives panel + grid.** Do not split “audit” and “schedule” into two divergent code paths for the main UI.
2. Taken ∪ frozen *course* codes feed validation; frozen *slot* ids pin layout only.
3. Pins win over hints; suggestions fill what remains under CU caps.
4. Payload order is sacred for multi-degree UI; remap overlap ids when internal compact indices differ.

When packing looks wrong but the requirements panel looks right, debug this file (and hints). When both look wrong, start with validation.

---

## `analytics.rs`

Optional product analytics for `/generate_schedule`.

**Owns:** schema ensure, insert of compact generate events (counts, combo key, violation types, latency, anon session id). No course lists of full plans in the happy path: keep rows small and privacy-light.

**Principle:** planning must succeed if the DB is down or unset. Analytics failures are logged and ignored for the HTTP response.

---

## `bin/analytics_report.rs`

CLI reader for those rows (day windows, per-session diffs). Not part of the request path.

---

## Tests (`degree_planner/tests/`)

Behavioral coverage for catalogs, builders, relations, caps, pools, overlap, scheduling, properties.

**Principle:** when changing engine behavior, add or extend a test that states the Penn rule in plain language. Prefer tests over long comments that go stale.

---

## Bug → file cheat sheet

| Symptom | Start here |
| --- | --- |
| Program missing from picker | `major.rs` implemented gate / catalog |
| Wrong requirements for a program | matching `penn_data/*_data.rs` |
| Course maps to wrong slot | `requirement.rs` fill order / matching; then tree |
| Pool coverage or double-count odd | `requirement.rs` pool evaluation; authored constraints |
| Alias counted twice / mutex missed | `course_relations.rs` |
| Dual-degree claim / CU cap wrong | `cross_degree.rs`, then `resolve_cross_degree_conflicts` |
| Overlap suggestions weak or huge | `overlap_planner.rs` |
| Panel OK, grid packing wrong | `scheduler.rs`, `schedule_template.rs` |
| Prereq warning wrong | `prereq.rs` (remember: warn-only) |
| Route / JSON shape | `main.rs` |

Next: [Features](./features.md) for feature-level design and rationale.
