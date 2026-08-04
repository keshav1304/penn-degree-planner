# Documentation

This folder explains how the Penn Degree Planner works, starting from the ideas behind it and moving into how the code is organized.

Read in this order if you are new to the project:

1. [Domain model](./domain.md): what the planner is trying to do, and the core ideas
2. [Glossary](./glossary.md): short definitions for terms used everywhere
3. [Architecture](./architecture.md): how the frontend, API, and library fit together
4. [Identifiers](./ids.md): course codes vs `req:` slots vs pool / overlap ids
5. [Flows](./flows/README.md): what happens on the main user actions
6. [Backend modules](./backend/modules.md): file-by-file map of the Rust crate
7. [Backend features](./backend/features.md): feature-by-feature design and rationale
8. [Frontend modules](./frontend/modules.md): map of the Next.js app

The source of truth for degree rules is still the Rust code under `degree_planner/src/penn_data/`. These docs explain *how to think about that code*, not a second copy of every major’s requirements.
