# Frontend modules

Map of the Next.js app under `frontend/`. Academic rules still live in Rust; this side orchestrates state and presentation.

## Entry points

| Path | Role |
| --- | --- |
| `app/page.js` | Main client page: bootstrap, plan state, debounced `generate_schedule`, DnD handlers, persistence. |
| `app/layout.js` | App shell / metadata. |
| `lib/api.js` | `API_BASE` (`NEXT_PUBLIC_API_URL` or the Fly.io default). |

`page.js` is large on purpose today: it owns the single source of plan state the children share. When reading it, skim the state declarations and `generateSchedule` first, then the handlers.

## Components (`app/components/`)

| Component | Role |
| --- | --- |
| `DegreeSelector.js` | Pick schools / majors / minors / concentrations. |
| `DegreeProgramPopover.js` | Program picking UI details. |
| `CourseSearch.js` | Search and add courses (uses local catalog index). |
| `RequirementsPanel.js` | Renders `degree_results`: categories, fulfilled/open rows, pools, navigation into the grid. |
| `ScheduleGrid.js` | Semester columns, CU totals, drop targets. |
| `DroppableSemester.js` | Per-semester droppable region. |
| `DraggableCourse.js` | Draggable course / slot chip. |

Drag-and-drop uses `@dnd-kit`. Dropping updates frozen/assigned state in `page.js`, which retriggers generate.

## Libraries (`lib/`)

### Identity and plan hygiene

| File | Role |
| --- | --- |
| `courseUtils.js` | Course-code vs `req:` vs overlap-group vs pool-constraint predicates; schedule status; filters for taken/frozen/assigned. **Read this before changing what may live on the grid.** |
| `requirementNav.js` | DOM ids, overlap slot parsing, pool constraint instance helpers, panel↔grid navigation. |
| `requirementText.js` | Labels and instance id helpers for requirement rows. |

### Catalog and display

| File | Role |
| --- | --- |
| `courseCatalog.js` | Prepare slim index, local search, CU map. |
| `courseRelations.js` | Client-side also-offered / mutex relations from index rows. |
| `courseOrdering.js` | Ordering helpers for lists. |
| `degreeDisplay.js` | Display names / presentation for degrees. |
| `degreeColors.js` | Color coding per degree on the UI. |
| `scheduleDisplay.js` | Schedule-facing display helpers. |
| `casRequirementsLayout.js` | CAS-specific requirements panel layout. |
| `crossDegree.js` | Client helpers for allocations, violation maps, concentration filtering for multi-degree views. |

### Schedule policy (UI-side defaults only)

| File | Role |
| --- | --- |
| `semesterOptions.js` | Max years, default CU limits by school mix, building the limits map sent to the API. |

Defaults here must stay consistent with backend CU policy helpers in `scheduler.rs`. If they drift, dual-degree load limits will look wrong in the UI before the API corrects packing.

### Export and misc

| File | Role |
| --- | --- |
| `exportScheduleImage.js` | JPEG export of the schedule. |
| `exportScheduleExcel.js` | Excel export. |
| `exportRequirementsFlat.js` | Flattened requirements export helpers. |
| `anonSession.js` | Persistent anonymous id for analytics. |
| `planPersistence.js` | `localStorage` for plan inputs and the last matching generate response. |
| `perfLog.js` | Optional timing logs during bootstrap / fetches. |

## Static assets

- `public/course_index.json`: generated course search index.
- `scripts/generate-course-index.mjs`: regenerates that index (`npm run generate:catalog`).

## State that matters

Owned in `page.js` (names approximate):

- `degrees`, `takenCourses`, `frozenCourses`, `assignedCourses`
- `scheduleData`: last `/generate_schedule` response (restored from `localStorage` on reload when inputs match, then refreshed in the background)
- `allowSummer`, `semesterCuLimits`
- catalogs: `degreeCatalog`, `minorCatalog`, `concentrationCatalog`, `courseCatalog`, …

Children should receive props/callbacks rather than inventing a second plan store.

## How to navigate a UI bug

| Symptom | Likely place |
| --- | --- |
| Wrong programs in the picker | API catalogs; then `DegreeSelector` |
| Search results wrong | `course_index.json` / `courseCatalog.js` |
| Panel shows wrong fulfillment | Backend `degree_results`; then `RequirementsPanel` rendering |
| Grid shows a coverage constraint as a course | `courseUtils` predicates + freeze filters in `page.js` |
| Clicking a requirement does not scroll/highlight | `requirementNav.js` / panel id wiring |
| DnD pin lost after regenerate | freeze/assign merge logic in `page.js` vs new `scheduleData` |
| Export missing slots | export helpers + what `scheduleData` contains |
