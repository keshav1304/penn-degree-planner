# Adding or changing a degree

How a program becomes something the planner can select and enforce.

## Where trees are authored

Requirement trees are written in Rust under `degree_planner/src/penn_data/`:

| Module | Typical content |
| --- | --- |
| `college_data.rs` | CAS (College) majors, gen-ed / pool patterns |
| `seas_data.rs` | SEAS undergraduate |
| `seas_grad_data.rs` | SEAS master’s (`SEAS_MS`) |
| `wharton_data.rs` | Wharton |
| `nursing_data.rs` | Nursing |
| `requirement_builders.rs` | Shared helpers for common slot shapes |
| `courses_data.rs` / `attributes_data.rs` | Catalog and attribute indexes the trees rely on |

A program is a `Major` struct (`major.rs`): display/short names, a `Vec<Requirement>`, optional concentration map, and optional `schedule_hints` (preferred year/semester for certain indices or courses).

## Wiring into the catalog

`major.rs` is the registry:

- `degree_catalog` / `minor_catalog`: what the UI lists (only entries considered implemented).
- `resolve_major` / `resolve_minor`: build or cache the `Major` for a school + API code + concentrations.
- `concentrations_for_program` / `all_concentrations`: concentration pickers.
- `major_is_implemented` / `major_has_authored_requirements`: hide stubs and placeholder-only trees from the catalog.

School-specific `build_*` functions inside `major.rs` (and the data modules they call) are where a new API code is hooked up.

## Checklist for a new major or minor

1. **Author the tree** in the right `penn_data` module using existing `Requirement` variants. Prefer helpers in `requirement_builders` when the pattern already exists (electives, unrestricted slots, and so on).
2. **Register** it in the school’s catalog list and in `resolve_major` / `resolve_minor` wiring.
3. **Concentrations**: if needed, add concentration requirement maps and ensure `concentrations_for` returns the names the UI should show.
4. **Schedule hints** (optional): if core courses should land in specific terms, add hints on the `Major`.
5. **Implemented gate**: confirm the tree is not placeholder-only so it appears in `/degree_catalog` or `/minor_catalog`.
6. **Tests**: add or extend a case under `degree_planner/tests/` that maps a small taken list and asserts fulfilled slots / smoke behavior.
7. **Manual UI check**: select the program, generate a schedule, confirm panel categories and open slots look right; for multi-degree, spot-check overlap if this program often pairs with another.

## What not to do

- Do not copy the full requirement tree into markdown or into the frontend. The Rust tree is the source of truth.
- Do not add one-off matching rules in JavaScript for a single major. If the engine cannot express the rule, extend `Requirement` / validation carefully and document the new variant in [Domain](../domain.md).
- Do not expose unfinished stubs in the catalog; keep them out via the implemented checks until the tree is real.

## Related reading

- [Domain: requirements as a tree](../domain.md#2-requirements-as-a-tree)
- [Backend modules](../backend/modules.md)
- [Backend features: authoring degree trees](../backend/features.md#3-authoring-degree-trees)
- `Requirement` docs at the top of `degree_planner/src/requirement.rs`
