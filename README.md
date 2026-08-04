# Penn Degree Planner

Plan Penn courses against degree requirements and lay out a multi-year schedule. Supports one or more majors/minors, including cross-school combinations.

## Repo layout

| Path | What it is |
| --- | --- |
| `degree_planner/` | Rust library + Axum API (requirements, overlap, scheduling, embedded catalog) |
| `frontend/` | Next.js UI |
| `docs/` | How the system works |

## Why Rust for the backend

The hard work here is not CRUD. Each schedule generate walks requirement trees, maps courses onto slots, searches for cross-degree overlaps, and packs semesters, often for more than one program at once. That path is CPU-heavy and runs on every meaningful UI edit (debounced), so the backend needs to stay fast without a large machine.

Rust fits that shape of problem:

- **Performance** for combinatorial mapping and packing, so multi-degree plans stay responsive on a small Fly.io VM.
- **Types and ownership** for a large, nested requirement model (pools, restrictions, instance ids) where silent shape mistakes are expensive.
- **A single static binary** with the catalog and degree data compiled in: simple deploy, no separate rule DB for the core engine.

The UI stays in Next.js because that is the right tool for interactive layout and state. Academic rules stay in Rust so there is one place that decides fulfillment and scheduling.

## Documentation

Read in order:

1. [docs/README.md](./docs/README.md): index
2. [Domain model](./docs/domain.md): first principles
3. [Glossary](./docs/glossary.md)
4. [Architecture](./docs/architecture.md)
5. [Identifiers](./docs/ids.md): course codes vs schedule slot ids
6. [Flows](./docs/flows/README.md)
7. [Backend modules](./docs/backend/modules.md) (file map) / [Backend features](./docs/backend/features.md) (design and why)
8. [Frontend modules](./docs/frontend/modules.md)

## Run locally

### API

```bash
cd degree_planner
cargo run
```

Listens on `0.0.0.0:8080`. Optional: set `DATABASE_URL` in a gitignored `.env` next to `Cargo.toml` if you want schedule-generate analytics.

### Frontend

```bash
cd frontend
npm install
npm run dev
```

Point the UI at a local API when needed:

```bash
NEXT_PUBLIC_API_URL=http://localhost:8080 npm run dev
```

If `NEXT_PUBLIC_API_URL` is unset, the app defaults to the deployed API (`https://degree-planner.fly.dev`).

## Tests

```bash
cd degree_planner
cargo test
```

## Deploy notes

- API: Fly.io (`degree_planner/fly.toml`)
- Frontend: standard Next.js host (e.g. Vercel); set `NEXT_PUBLIC_API_URL` to the API you want that deployment to call
