# Identifiers

The planner mixes real course codes with synthetic ids for open requirements. Mixing them up is a common source of bugs. This page is the cheat sheet.

## Course codes

Shape: `DEPT NUMBER` with a single space, e.g. `CIS 1600`.

- Validated by `course::is_valid_course_code` (Rust) and `isValidCourseCode` (JS).
- Appear in taken lists, fulfilled `course_ids`, and `SemesterPlan.courses`.

Also-offered spellings may normalize to a canonical code during validation and scheduling.

## Instance ids (inside a degree tree)

Assigned while mapping a major’s requirement list. They identify a *slot*, not a catalog course.

Examples of shapes you will see:

| Example | Meaning |
| --- | --- |
| `0`, `3` | Top-level requirement index |
| `1:f0` | Fixed slot inside a pool |
| `1:p0` | Flexible (generic) pool slot |
| `1:c0` | Pool *coverage constraint* (not a separate schedule CU) |
| `1:f0:c0` | Nested id under a fixed pool slot (not the same as a bare coverage id) |

Exact formatting is produced in `requirement.rs`. Frontend predicates in `lib/courseUtils.js` decide which of these may become schedule placements.

## Requirement slot ids (`req:…`)

Open slots on the schedule use ids that start with `req:`.

Typical forms:

- `req:{instance…}` or `req:{scope}:{fingerprint}`: ordinary open slot
- `req:{scope}:BB:Business_Breadth`: Wharton business-breadth style slots (scoped)
- `req:overlap:…`: paired cross-degree block (one CU on the grid)

Rules of thumb:

1. If it does **not** start with `req:`, it should be a course code (or invalid junk to strip).
2. Pool **coverage** constraint slots must **not** be treated as schedulable CU. Use `isSchedulableRequirementSlotId` / the Rust equivalents.
3. Overlap groups are schedulable, but they represent a *pair* of requirements, not one catalog course.

## Labels vs ids

`slot_labels` on the schedule response maps a `req:…` id to human text for the grid. Never store the label as if it were an id (legacy taken lists sometimes did; the UI now filters those out).

## Where enforcement lives

| Concern | Backend | Frontend |
| --- | --- | --- |
| Build instance / `req:` ids | `requirement.rs` | - |
| Decide schedulable vs not | `requirement` helpers | `courseUtils.js` |
| Pin slots across regenerates | accepts frozen `req:` ids | `frozenCourses` + cleanup vs latest suggestions |
| Navigate panel ↔ grid | instance ids in mapped results | `requirementNav.js` |

When adding a new placeholder kind, update both sides’ predicates and this page.
