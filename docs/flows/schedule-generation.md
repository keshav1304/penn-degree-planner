# Schedule generation

This is the main runtime loop. One API call returns both per-degree requirement results and a multi-semester plan. The requirements panel and the schedule grid both read that response.

## Trigger

In `app/page.js`, `generateSchedule` runs (debounced ~500ms) when any of these change:

- selected degrees
- taken courses
- frozen or assigned placements that pin items to semesters
- allow-summer
- semester CU limits
- max schedule year used to expand CU limit maps

A monotonic request id drops stale responses if a newer generate finished first.

## What the frontend sends

Conceptual payload:

1. **taken**: valid course codes from “My Courses” / taken list.
2. **degrees**: for each selection: `kind` (`major`|`minor`), `school`, `major` (API code), `concentrations` (and legacy single `concentration` for compatibility).
3. **frozen**: pinned items: real courses *and* schedulable requirement slot ids, each with `year` and `semester`. Assigned courses that already sit on a semester are included here so the backend treats them as pinned placements.
4. **allow_summer**: whether summer terms exist in the plan.
5. **semester_cu_limits**: map of semester keys to CU caps (defaults filled in `lib/semesterOptions.js`, overrides from UI state).
6. **anon_session_id**: optional analytics id from `lib/anonSession.js`.

## What the backend does (high level)

Entry point: `scheduler::generate_schedule` via `POST /generate_schedule` in `main.rs`.

### A. Normalize inputs

- Keep only valid course codes / requirement slot ids.
- Normalize and de-duplicate codes through `course_relations` (canonical forms for also-offered-as).
- Build `courses_for_validation` = taken ∪ frozen *course* codes. Frozen open slots pin layout but are not themselves catalog courses.

### B. Resolve each degree

For each payload degree:

- `resolve_major` or `resolve_minor` (see `major.rs`) loads the authored `Major` (requirement tree, concentrations, schedule hints) from the right `penn_data` school module.
- If resolution fails, that degree gets an error result and is skipped for packing.

### C. Validate requirements per degree

`requirement::validate_courses_for_degree` (and related helpers) maps `courses_for_validation` onto the tree:

- fulfilled / unfulfilled / partial mapped requirements
- pool coverage info
- concentration info where relevant

Suggestions for open slots are produced so the scheduler has concrete courses or `req:…` placeholders to place.

### D. Cross-degree and overlap

When multiple degrees are selected:

- `cross_degree` applies sharing rules, CU caps (including undergrad↔grad), and violation detection.
- `overlap_planner` looks for shared courses and paired requirement blocks.
- The scheduler may emit `overlap_schedule_groups` so the grid can show one CU for a paired pair.

Exact applicability differs: overlap discovery vs the fuller undergrad-only optimizer. See comments and helpers at the top of `cross_degree.rs`.

### E. Pack the schedule

Using schedule hints on the major, school CU policies, dual-degree year adjustments, frozen pins, and remaining suggested items, the scheduler fills `SemesterPlan` rows:

- `courses`: course codes
- `requirement_slots`: open `req:…` ids
- `total_cu`

It also returns `slot_labels` so the UI can show readable text for placeholders.

### F. Analytics (optional)

If Postgres is connected, a generate event may be recorded. Failure here must not fail the HTTP response.

## What the frontend does with the response

`setScheduleData(data)` stores the JSON.

- **Schedule grid** (`ScheduleGrid` and friends) renders semesters, courses, requirement slots, and overlap groups; drag-and-drop updates frozen/assigned state.
- **Requirements panel** (`RequirementsPanel`) reads `degree_results` (fulfilled, unfulfilled, suggestions, pool info, CAS gen-ed extras, etc.).
- Navigation helpers (`lib/requirementNav.js`, `lib/requirementText.js`) connect a panel row to a grid slot and vice versa using instance ids and `req:` ids.
- Cleanup effects drop frozen slot pins that are no longer open / schedulable after the new result.

Editing the grid or taken list updates React state → debounced generate again.

## Mental model

```
User edits plan
    → POST /generate_schedule
        → resolve trees
        → map courses → requirements
        → overlap / violations
        → pack semesters around pins
    → UI redraws panel + grid from one payload
```

Do not expect the frontend to “finish” requirements locally. If mapping looks wrong, debug Rust validation / authored trees first.

## Legacy `POST /`

`POST /` still validates a *single* school + major + taken list and returns the same style of fulfilled/unfulfilled/suggested/pool fields without building a multi-year schedule.

The current UI does not depend on this for the main screen. It is useful for API smoke tests and single-degree debugging.
