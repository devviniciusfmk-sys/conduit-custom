# Documentation migration report

## Scope

The 65 tracked Markdown documents were inventoried and classified. The documentation was reorganized using `git mv` so Git can retain file history. Technical prose, examples, commands, and semantics were not rewritten. Only filesystem locations and references that identify those locations were updated.

## Target architecture

| Destination | Purpose |
| --- | --- |
| `docs/guide/` | Existing mdBook user/product guide (formerly `docs/src/`) |
| `docs/development/` | Contribution and testing documentation |
| `docs/operations/` | Security and operational policy |
| `docs/roadmap/` | Roadmaps and implementation plans |
| `docs/research/` | UX/design research and visual studies |
| `docs/history/` | Changelog and release history |

## File-by-file moves

| Previous path | New path | Classification |
| --- | --- | --- |
| `CHANGELOG.md` | `docs/history/CHANGELOG.md` | History / release notes |
| `CONTRIBUTING.md` | `docs/development/CONTRIBUTING.md` | Development / contribution guide |
| `SECURITY.md` | `docs/operations/SECURITY.md` | Operations / security policy |
| `FORK_SESSION_PLAN.md` | `docs/roadmap/FORK_SESSION_PLAN.md` | Roadmap / implementation plan |
| `WEB_ROADMAP.md` | `docs/roadmap/WEB_ROADMAP.md` | Roadmap / product parity |
| `docs/TESTING_PLAN.md` | `docs/development/TESTING_PLAN.md` | Development / test plan |
| `docs/WEB_UI_TESTING_CHECKLIST.md` | `docs/development/WEB_UI_TESTING_CHECKLIST.md` | Development / UI test checklist |
| `plans/tui-design-research.md` | `docs/research/tui-design-research.md` | UX / design research |
| `plans/tui-visual-polish.md` | `docs/research/tui-visual-polish.md` | UX / visual implementation study |

The former `docs/src/` tree moved one-to-one to `docs/guide/`. Every Markdown file in that tree retained its filename and relative position:

```text
docs/src/<path>.md  ->  docs/guide/<path>.md
```

Explicit file mapping:

- `docs/src/SUMMARY.md` → `docs/guide/SUMMARY.md`
- `docs/src/introduction.md` → `docs/guide/introduction.md`
- `docs/src/advanced/data-storage.md` → `docs/guide/advanced/data-storage.md`
- `docs/src/advanced/session-import.md` → `docs/guide/advanced/session-import.md`
- `docs/src/advanced/tokens-cost.md` → `docs/guide/advanced/tokens-cost.md`
- `docs/src/advanced/troubleshooting.md` → `docs/guide/advanced/troubleshooting.md`
- `docs/src/commands/conduit.md` → `docs/guide/commands/conduit.md`
- `docs/src/commands/debug-keys.md` → `docs/guide/commands/debug-keys.md`
- `docs/src/commands/migrate-theme.md` → `docs/guide/commands/migrate-theme.md`
- `docs/src/concepts/agents.md` → `docs/guide/concepts/agents.md`
- `docs/src/concepts/agents/claude-code.md` → `docs/guide/concepts/agents/claude-code.md`
- `docs/src/concepts/agents/codex.md` → `docs/guide/concepts/agents/codex.md`
- `docs/src/concepts/agents/gemini.md` → `docs/guide/concepts/agents/gemini.md`
- `docs/src/concepts/build-plan-mode.md` → `docs/guide/concepts/build-plan-mode.md`
- `docs/src/concepts/projects.md` → `docs/guide/concepts/projects.md`
- `docs/src/concepts/sessions.md` → `docs/guide/concepts/sessions.md`
- `docs/src/concepts/tabs.md` → `docs/guide/concepts/tabs.md`
- `docs/src/concepts/workspaces.md` → `docs/guide/concepts/workspaces.md`
- `docs/src/configuration/config-file.md` → `docs/guide/configuration/config-file.md`
- `docs/src/configuration/keybindings.md` → `docs/guide/configuration/keybindings.md`
- `docs/src/configuration/overview.md` → `docs/guide/configuration/overview.md`
- `docs/src/configuration/themes.md` → `docs/guide/configuration/themes.md`
- `docs/src/configuration/themes/builtin.md` → `docs/guide/configuration/themes/builtin.md`
- `docs/src/configuration/themes/custom.md` → `docs/guide/configuration/themes/custom.md`
- `docs/src/configuration/themes/vscode-migration.md` → `docs/guide/configuration/themes/vscode-migration.md`
- `docs/src/configuration/tool-paths.md` → `docs/guide/configuration/tool-paths.md`
- `docs/src/getting-started/first-session.md` → `docs/guide/getting-started/first-session.md`
- `docs/src/getting-started/installation.md` → `docs/guide/getting-started/installation.md`
- `docs/src/getting-started/quick-start.md` → `docs/guide/getting-started/quick-start.md`
- `docs/src/git/branch-status.md` → `docs/guide/git/branch-status.md`
- `docs/src/git/pr-tracking.md` → `docs/guide/git/pr-tracking.md`
- `docs/src/git/worktrees.md` → `docs/guide/git/worktrees.md`
- `docs/src/reference/config.md` → `docs/guide/reference/config.md`
- `docs/src/reference/shortcuts.md` → `docs/guide/reference/shortcuts.md`
- `docs/src/reference/theme-properties.md` → `docs/guide/reference/theme-properties.md`
- `docs/src/shortcuts/chat.md` → `docs/guide/shortcuts/chat.md`
- `docs/src/shortcuts/dialogs.md` → `docs/guide/shortcuts/dialogs.md`
- `docs/src/shortcuts/global.md` → `docs/guide/shortcuts/global.md`
- `docs/src/shortcuts/quick-reference.md` → `docs/guide/shortcuts/quick-reference.md`
- `docs/src/shortcuts/scrolling.md` → `docs/guide/shortcuts/scrolling.md`
- `docs/src/shortcuts/sidebar.md` → `docs/guide/shortcuts/sidebar.md`
- `docs/src/ui/chat-view.md` → `docs/guide/ui/chat-view.md`
- `docs/src/ui/command-palette.md` → `docs/guide/ui/command-palette.md`
- `docs/src/ui/overview.md` → `docs/guide/ui/overview.md`
- `docs/src/ui/sidebar.md` → `docs/guide/ui/sidebar.md`
- `docs/src/ui/status-bar.md` → `docs/guide/ui/status-bar.md`
- `docs/src/ui/tab-bar.md` → `docs/guide/ui/tab-bar.md`

## Files intentionally retained in place

- `README.md` — root project overview and repository entry point.
- `AGENTS.md` — repository instruction file discovered by development tooling.
- `.github/ISSUE_TEMPLATE/*.md` and `.github/PULL_REQUEST_TEMPLATE.md` — GitHub requires these paths.
- `mockups/README.md`, `web/README.md`, `website/README.md`, and `website/scripts/README.md` — package-local READMEs documenting their own subprojects.

## Reference updates

- `docs/book.toml` now points mdBook at `guide`.
- `website/README.md` now refers to `docs/guide/`.
- Contribution-template links now resolve from `docs/development/`.
- The root README continues to link to the canonical `docs/` directory.

## Validation performed

- All Markdown files were inventoried and classified.
- Local relative Markdown links were checked after the moves.
- `git diff --check` was run.
- `git diff --summary` / rename detection should show the moves after staging or commit.
