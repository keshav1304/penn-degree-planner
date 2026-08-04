# Architecture

## Big picture

The repo has two main packages:

| Path | Role |
| --- | --- |
| `degree_planner/` | Rust library + Axum HTTP server. Owns catalog data, degree trees, mapping, overlap, and schedule generation. |
| `frontend/` | Next.js app. Owns the interactive UI, local persistence, and calls to the API. |

Almost all academic logic runs in Rust. The frontend prepares inputs, displays results, and lets the user pin and rearrange the plan. It does not re-implement requirement trees.

```
┌─────────────────────┐         HTTP JSON          ┌──────────────────────────┐
│  Next.js frontend   │ ─────────────────────────► │  Axum API (main.rs)      │
│  app/, lib/         │ ◄───────────────────────── │  :8080                   │
└─────────────────────┘                            └────────────┬─────────────┘
         │                                                      │
         │ static /course_index.json                            ▼
         │                                         ┌──────────────────────────┐
         ▼                                         │  degree_planner library  │
┌─────────────────────┐                            │  requirement, scheduler, │
│  Slim course index  │                            │  penn_data, …            │
│  (search / CU map)  │                            └────────────┬─────────────┘
└─────────────────────┘                                         │
                                                                ▼
                                                   ┌──────────────────────────┐
                                                   │  Embedded Penn data      │
                                                   │  (+ optional Postgres    │
                                                   │   for analytics only)    │
                                                   └──────────────────────────┘
```

## Backend layout

Inside `degree_planner/`:

- `src/lib.rs`: library modules (the domain).
- `src/main.rs`: HTTP server to parse request, call library, return JSON.
- `src/penn_data/`: authored course catalog, attributes, and per-school requirement trees.
- `src/bin/analytics_report.rs`: optional CLI for reading analytics from Postgres.
- `tests/`: behavioral tests for majors, relations, caps, and smoke cases.

Deployment: the API is configured for [Fly.io](https://fly.io) (`fly.toml`), default public URL used by the frontend: `https://degree-planner.fly.dev`.

Optional `DATABASE_URL` enables recording schedule-generate events. If it is missing or the DB is unreachable, the API still runs; analytics are simply skipped.

## Frontend layout

Inside `frontend/`:

- `app/page.js`: main client page: state, bootstrap fetches, debounced schedule generation, drag-and-drop wiring.
- `app/components/`: degree selector, course search, schedule grid, requirements panel, and related pieces.
- `lib/`: pure helpers (IDs, labels, catalog prep, export, semester CU defaults, cross-degree display helpers).
- `public/course_index.json`: generated slim catalog for client-side search (see `npm run generate:catalog`).

The UI keeps plan state in React state and mirrors a subset into `localStorage` under `penn_degree_planner_state`.

## Data ownership

| Data | Where it lives |
| --- | --- |
| Full course catalog + CU map | Embedded in Rust (`penn_data/courses_data`) |
| Course attributes | Embedded in Rust (`attributes_data`) |
| Degree / minor requirement trees | Authored in Rust school modules; resolved through `major` |
| Slim search index | Frontend static JSON (generated from catalog tooling) |
| User plan (degrees, taken, frozen, CU limits) | Browser (`localStorage` + React state) |
| Schedule generate analytics | Optional Postgres |

There is no server-side user account or saved plan store in the current design. Refreshing the browser relies on `localStorage`.

## HTTP API

All routes are on the Axum app in `main.rs`. CORS is open for browser use.

### Catalog and discovery

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/all_courses` | Full embedded course list |
| `GET` | `/search_courses?q=&limit=` | Lightweight course search |
| `GET` | `/course_cu_map` | Course code → CU |
| `GET` | `/course?code=` | Single course record |
| `GET` | `/all_majors` | School → major codes map |
| `GET` | `/degree_catalog` | Schools + majors with display names (implemented programs only) |
| `GET` | `/minor_catalog` | Same shape for minors |
| `GET` | `/concentrations?school=&major=&kind=` | Concentrations for one program |
| `GET` | `/all_concentrations` | All concentrations keyed for the UI |

### Requirements (legacy single-degree)

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/` | Validate `taken` courses against one `school` + `major` (+ optional concentration). Returns fulfilled / unfulfilled / suggested / pool coverage. |

The current UI’s main path is schedule generation (below), which already returns per-degree requirement results. `POST /` remains useful for simpler clients or debugging a single degree.

### Schedule generation

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/generate_schedule` | Full multi-degree validation + overlap + semester layout |

Request body (conceptual):

```json
{
  "taken": ["CIS 1200", "MATH 1400"],
  "degrees": [
    {
      "kind": "major",
      "school": "SEAS",
      "major": "CIS",
      "concentrations": []
    }
  ],
  "frozen": [
    { "course_id": "CIS 1200", "year": 1, "semester": "Fall" }
  ],
  "allow_summer": false,
  "semester_cu_limits": { "1-Fall": 5.5 },
  "anon_session_id": "optional-uuid"
}
```

Response (conceptual): `schedule`, `degree_results`, `slot_labels`, `cross_degree_summary`, `overlap_plan`, `overlap_schedule_groups`, `error`.

## What the frontend calls today

On load:

1. `/course_index.json` from the Next app (not the Rust API)
2. `GET /degree_catalog`
3. `GET /minor_catalog`
4. `GET /all_concentrations`

Whenever degrees, taken courses, frozen/assigned pins, summer, or CU limits change (debounced ~500ms):

5. `POST /generate_schedule`

That one call drives both the requirements panel and the schedule grid.

## Trust boundary

Treat the API as the authority for:

- whether a program exists and what its tree is
- how courses map to requirements
- how overlaps and CU caps are applied
- how a schedule is packed

Treat the frontend as the authority for:

- what the user currently selected and pinned
- presentation, navigation between panel and grid, export
- which ids are allowed in local state (see `lib/courseUtils.js`)

If UI and API disagree about academic rules, fix the Rust side (or the authored `penn_data`), not a parallel rule set in JavaScript.

For a deeper backend file map and feature-level rationale, see [Backend modules](./backend/modules.md) and [Backend features](./backend/features.md).
