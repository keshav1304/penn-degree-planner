# Glossary

Terms used across the backend, frontend, and docs. Prefer these words in new code and comments.

## Course and catalog

**Course code**  
Canonical id for a catalog course, e.g. `STAT 4300`. Department + space + number.

**CU**  
Course unit. Usually `1.0` or `0.5`. Restriction data sometimes uses tenths (`5` → 0.5 CU).

**Also offered as**  
Alternate spellings of the same offering. The planner treats related codes as the same course for counting when relations say so.

**Mutually exclusive**  
Courses that cannot both count on a plan. Surfaced as a cross-degree / plan violation when both appear.

**Attribute**  
Catalog tag (e.g. writing, specific Wharton tags) used by Restriction matching and pool constraints.

## Degree structure

**School code**  
Short school id used in APIs and data, e.g. `CAS`, `SEAS`, `WHARTON`, `NURS`, `SEAS_MS`.

**Major / minor / program**  
A selectable degree entry. The API field is often still named `major` even for minors; `kind` distinguishes `"major"` vs `"minor"`.

**API code**  
Stable program id sent to the backend (not always the display name).

**Concentration**  
Optional specialization under a program. Requirements for it may be a `Concentration` node or a separate map on the `Major`.

**Requirement tree**  
Nested `Requirement` enum values that define what a program needs. Authored in `penn_data`, interpreted by `requirement`.

**Category**  
Optional label on a requirement node for UI grouping (e.g. “Foundational Courses”).

## Mapping and fulfillment

**Mapped requirement**  
A requirement node plus the course ids (or placeholder ids) assigned to it, plus optional `instance_id`, partial flag, and AnyOf branch commitment.

**Instance id**  
Stable id for a leaf or pool slot within a degree’s tree (e.g. `"0"`, `"1:f0"`, `"1:p0"`, `"1:c0"`). Used to build schedule placeholder ids and to navigate the requirements panel.

**Fulfilled / unfulfilled / suggested**  
Buckets after validation: done slots, open slots, and proposals for open slots.

**Partial**  
Some courses assigned, but the slot is not fully satisfied yet.

**Committed AnyOf branch**  
When an `AnyOf` is partially filled, which alternative path the engine has locked onto.

## Pools

**Course pool**  
Shared bucket: fixed slots + flexible slots + coverage constraints.

**Fixed slot**  
Non-fungible requirement inside a pool (often a specific course or nested requirement). Instance segments often look like `f0`, `f1`, …

**Flexible slot**  
Generic 1-CU placeholder in a pool. Instance segments look like `p0`, `p1`, …. These can appear on the schedule.

**Coverage constraint**  
A labeled rule evaluated against courses in the pool. Instance segments look like `c0`, `c1`, …. These are *not* separate schedule CU by themselves; the UI must not treat them like ordinary open slots.

**Consumption group**  
Limits how pool courses can be reused across related constraints.

**Double-count (pool)**  
A pool course may satisfy a limited number of coverage constraints (currently two), subject to consumption groups.

## Schedule identifiers

**Requirement slot id**  
String starting with `req:`. Placeholder for an open requirement on the schedule (not a real course code).

**Schedulable requirement slot**  
A `req:…` id that represents real schedule CU: ordinary open slots, pool fixed/flex slots, or overlap groups, but not pool *coverage constraint* slots.

**Overlap schedule group id**  
Starts with `req:overlap:`. One grid block standing for a paired cross-degree requirement placement (one CU).

**Slot label**  
Human-readable text for a requirement slot id, returned in `slot_labels` from schedule generation.

## Student plan state (frontend)

**Taken**  
Course codes the student counts toward degrees. Stored in local state / `localStorage`.

**Frozen**  
Pinned year/semester placements (courses or schedulable slot ids). Sent to the API as `frozen`.

**Assigned**  
Taken courses placed on a specific semester in the UI (typically shown as completed/green).

**Suggested**  
Scheduler output the user has not pinned.

## Cross-degree

**Overlap plan**  
Computed opportunities for shared courses / paired slots across selected degrees.

**Cross-degree summary**  
Aggregated sharing and violation information for the current multi-degree selection.

**Violation**  
Plan problem such as too many degrees, undergrad–grad CU cap exceeded, mutually exclusive courses, duplicate also-offered spellings, or missing prerequisites (warn-style).

## Infrastructure

**API base**  
Backend URL used by the frontend (`NEXT_PUBLIC_API_URL`, defaulting to the Fly.io deployment).

**Anon session id**  
Browser-local id for optional analytics on schedule generates. Not authentication.
