# Workspaces

## Workspace identity

Each workspace has a display name, emoji icon, and color. The sidebar combines these fields,
for example `🎬 Render Engine`, so workspaces are easier to recognize at a glance.

In the Web interface, hover over a workspace and choose **Workspace settings**. You can edit:

- Name (up to 60 characters)
- Icon from the emoji picker
- Color: gray, blue, green, orange, purple, or red

Existing workspaces default to the `📁` icon and gray color. These visual settings do not
change the Git branch, worktree folder, path, open sessions, or session history.

A **workspace** is a working context within a project, tied to a git branch.

## What is a Workspace?

- Each workspace corresponds to a git branch
- Workspaces use git worktrees for isolation
- Session history is stored per workspace

## Git Worktrees

Conduit automatically creates git worktrees for each workspace. This allows:

- Working on multiple branches simultaneously
- Independent file states per branch
- Clean separation between features

## Creating Workspaces

When you open a project on a new branch, Conduit creates a workspace automatically.

## Workspace Storage

Workspace data is stored in:
```
~/.conduit/workspaces/<project>/<branch>/
```

## Archiving Workspaces

Press `x` on a workspace in the sidebar to archive it. This:
- Removes it from the sidebar
- Preserves session history
- Cleans up the worktree
