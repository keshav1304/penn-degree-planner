# Domain model

This project helps a Penn student plan which courses to take, check those courses against degree requirements, and lay out a multi-year schedule. It supports one or more majors/minors at once, including combinations across schools.

The rest of this page builds the model from the ground up. Later docs assume these ideas.

## 1. Courses

A **course** is a catalog entry: department, code, title, credit units (CU), prereqs, and a few relation fields (also-offered-as, mutually exclusive, corequisites).

Course codes look like `CIS 1200`: department letters, a space, then a number. The code is the main identifier almost everywhere in the app.

Credits are measured in **CU**. Most courses are 1.0 CU; some are 0.5. Restriction slots sometimes store CU in tenths in the data (`5` means 0.5 CU).

The catalog is embedded in the Rust backend and also shipped to the frontend as a slim search index (`course_index.json`). Search and CU lookups can happen on either side; degree logic always runs on the backend.

## 2. Requirements as a tree

A degree is not a flat checklist. It is a **tree of requirements**.

Each node is one of a few shapes:

| Shape | Meaning |
| --- | --- |
| `SingleCourse` | Pick exactly one course from a list (e.g. CIS 5190 or CIS 5200). |
| `AllOf` | Satisfy every child. |
| `AnyOf` | Satisfy one of several alternative paths. |
| `CourseGroup` | Satisfy N of M child areas. |
| `Concentration` | Like `AllOf`, but tied to a chosen concentration. |
| `Restriction` | A flexible slot matched by department, level, attributes, exclusions, etc. |
| `CoursePool` | A shared bucket of fixed + flexible slots, plus coverage rules over that bucket. |

Nodes can nest. “Take MEAM 1100 and one of MEAM 1470 / PHYS 0150” is an `AllOf` containing a `SingleCourse` and an `AnyOf`.

Optional `category` labels (e.g. “Foundational Courses”) are for grouping in the UI. They are not separate rule types.

School-specific files under `penn_data/` *author* these trees in Rust. The engine that *interprets* them lives mainly in `requirement.rs`.

## 3. Mapping courses onto requirements

Given a set of courses the student already has (or plans to count), the engine **maps** those courses onto leaves of the tree.

The result is not just pass/fail. For each slot you get a **mapped requirement**: the original requirement node, which course codes filled it (if any), and a stable `instance_id` so the UI can refer to that slot later.

Outcomes fall into buckets:

- **Fulfilled**: the slot is satisfied.
- **Unfulfilled**: still open.
- **Suggested**: for open slots, the engine proposes concrete courses or placeholder slot IDs the scheduler can place.
- **Unapplicable**: courses in the student’s list that did not land on any requirement for that degree.

Matching is greedy and ordered. Some requirement types expand first (for example, a Restriction asking for 3 CU becomes three 1-CU slots). `AnyOf` commits to the best-matching branch when the student’s courses partially fit one path.

## 4. Course pools

Some degrees (notably parts of CAS and Wharton) use a **CoursePool**: one bucket of courses that must cover several labeled rules at once.

A pool has:

- **Fixed slots**: specific courses or sub-requirements that must be filled.
- **Flexible slots**: generic 1-CU placeholders in the same bucket.
- **Constraints**: coverage rules evaluated against courses already in the pool (e.g. “at least two writing courses”).

A single pool course can usually count toward at most two coverage constraints (double-count limit). Consumption groups can still block reuse within the same group.

Pools matter for IDs: fixed slots, flexible slots (`…:p0`), and coverage constraints (`…:c0`) are different things. Only some of those IDs belong on the schedule grid. See the [glossary](./glossary.md) and [Identifiers](./ids.md).

## 5. Taken, frozen, and suggested

The UI keeps three related ideas about courses on a plan:

- **Taken**: courses the student counts as done (or firmly planned for credit). They feed requirement mapping.
- **Frozen**: placements pinned to a year/semester. Frozen items can be real course codes or open requirement slot IDs. They constrain the scheduler; course codes among them also count toward fulfillment.
- **Suggested**: what the generator placed that the student has not pinned. Shown on the grid until the student freezes or assigns them.

**Assigned** courses in the frontend are taken courses that have been placed onto a specific semester (often shown in green). Frozen placements are the orange pins.

When the schedule is regenerated, taken + frozen course codes are the set used for requirement validation. Suggested placements are recomputed around the pins.

## 6. Multiple degrees

Students can select more than one major or minor. Each degree gets its own requirement tree and its own mapped results.

When two or more degrees are selected, the backend also looks for **overlap**: courses or requirement slots that can satisfy more than one degree at once. That work lives in `cross_degree` and `overlap_planner`.

Important distinctions:

- Overlap *discovery* can run for undergrad↔grad mixes.
- The fuller cross-degree *optimizer* (shared courses, paired blocks, allocation filtering) applies when every selected degree is undergraduate.
- Undergrad↔grad sharing is capped (currently 3.0 CU of shared credit). Some combinations are treated as violations (too many degrees, grad–grad overlap rules, mutually exclusive courses, missing prereqs, and so on).

The schedule grid may show **overlap groups**: one CU block that stands for two paired requirements across degrees.

## 7. Scheduling

After requirements are known, **schedule generation** lays courses and open slots across years and semesters.

Inputs include:

- taken courses
- selected degrees (school, program code, kind major/minor, concentrations)
- frozen placements
- whether summer is allowed
- optional per-semester CU limits

Outputs include:

- a list of semester plans (courses + requirement slot IDs + total CU)
- per-degree requirement results (same shape as a standalone validation)
- human-readable labels for open slots
- optional cross-degree summary and overlap plan
- overlap schedule groups for the grid

The scheduler respects school-specific CU defaults, dual-degree year adjustments, schedule hints authored on majors, and the pins the user already set. It does not invent new degree rules; it places work implied by the mapped trees.

Prerequisites are parsed from catalog text and checked in a **warn-only** way: missing prereqs show up as violations/warnings rather than hard blockers that refuse a plan.

## 8. Where logic lives (preview)

Rough split:

1. **Catalog + attributes**: `course`, `penn_data/courses_data`, `attributes_data`, `prereq`, `course_relations`
2. **Authored degree trees**: `penn_data/college_data`, `seas_data`, `seas_grad_data`, `wharton_data`, `nursing_data`, plus helpers in `requirement_builders` and wiring in `major`
3. **Mapping and pools**: `requirement`
4. **Cross-degree rules and overlap**: `cross_degree`, `overlap_planner`
5. **Semester layout**: `scheduler`, `schedule_template`
6. **HTTP**: `main.rs` (Axum routes)
7. **UI**: Next.js app: selection, search, requirements panel, drag-and-drop schedule, local persistence

The next document, [Architecture](./architecture.md), shows how those pieces are deployed and which HTTP routes call which library functions.
