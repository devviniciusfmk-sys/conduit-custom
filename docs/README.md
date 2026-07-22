# Conduit documentation

This directory is the canonical home for project documentation. The documentation is organized by audience and lifecycle so that product guidance, engineering practices, operational policy, and project history remain easy to discover.

## Start here

- [Product and user guide](./guide/SUMMARY.md) — the mdBook entry point for installation, concepts, configuration, commands, shortcuts, UI, Git, and advanced usage.
- [Development](./development/) — contribution, testing, and web UI verification material.
- [Architecture](./architecture/) — system, backend, frontend, database, and API architecture references.
- [Product](./product/) — product vision, definition, features, milestones, and governing principles.
- [Operations](./operations/) — security and operational policy.
- [Roadmap](./roadmap/) — planned work and implementation plans.
- [Research](./research/) — design research and visual polish studies.
- [History](./history/CHANGELOG.md) — released changes and project history.

## Documentation architecture

| Area | Purpose | Primary audience |
| --- | --- | --- |
| `guide/` | Published mdBook product and user documentation | Users and operators |
| `development/` | Contribution and test workflows | Contributors and maintainers |
| `architecture/` | System and technical architecture references | Engineers and maintainers |
| `product/` | Product direction and decision framework | Product stakeholders and maintainers |
| `operations/` | Security and operational policy | Operators and maintainers |
| `roadmap/` | Future work and implementation planning | Maintainers and product stakeholders |
| `research/` | Design investigations and visual studies | Product and UX contributors |
| `history/` | Release history | All project readers |

### Roadmap distinction

- [`product/ROADMAP.md`](./product/ROADMAP.md) is the strategic roadmap for the Conduit AI Studio product.
- [`roadmap/WEB_ROADMAP.md`](./roadmap/WEB_ROADMAP.md) is the legacy, technical roadmap for the Web implementation.
- [`roadmap/FORK_SESSION_PLAN.md`](./roadmap/FORK_SESSION_PLAN.md) is an operational or historical plan for the evolution of the fork-session capability.

The root `README.md` remains the project overview and entry point. `AGENTS.md` remains at the repository root because tooling discovers it there. GitHub workflow templates and package-local READMEs remain beside the systems they document.

## Building the guide

The mdBook source is in `docs/guide/`; `docs/book.toml` configures the book build. From the repository root:

```bash
cd docs && mdbook build
```

For the migration inventory, see [MIGRATION-REPORT.md](./MIGRATION-REPORT.md).
