# Workspaces

## Rename a workspace

In the Web interface, hover over a workspace in the sidebar and choose **Rename workspace**.
Enter a display name of up to 60 characters and select **Rename** (or press Enter).

Renaming changes only the workspace's displayed name. Its Git branch, worktree folder,
open sessions, and session history remain unchanged.

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
