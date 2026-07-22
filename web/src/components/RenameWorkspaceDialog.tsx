import { useEffect, useRef, useState } from 'react';
import type { Workspace } from '../types';
import { ApiError } from '../lib/api';
import { useRenameWorkspace } from '../hooks/useApi';

interface RenameWorkspaceDialogProps {
  workspace: Workspace;
  onClose: () => void;
}

export function RenameWorkspaceDialog({ workspace, onClose }: RenameWorkspaceDialogProps) {
  const [name, setName] = useState(workspace.name);
  const mutation = useRenameWorkspace();
  const inputRef = useRef<HTMLInputElement>(null);
  const trimmedName = name.trim();
  const isValid = trimmedName.length > 0 && Array.from(trimmedName).length <= 60;

  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  useEffect(() => {
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !mutation.isPending) onClose();
    };
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [mutation.isPending, onClose]);

  const submit = () => {
    if (!isValid || mutation.isPending) return;
    mutation.mutate(
      { id: workspace.id, data: { name } },
      { onSuccess: onClose }
    );
  };

  const errorMessage = mutation.error
    ? mutation.error instanceof ApiError && mutation.error.status === 409
      ? 'A workspace with this name already exists in this repository.'
      : mutation.error.message || 'Unable to rename the workspace. Please try again.'
    : null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      role="presentation"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget && !mutation.isPending) onClose();
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="rename-workspace-title"
        className="w-full max-w-md rounded-lg border border-border bg-surface p-5 shadow-xl"
      >
        <h2 id="rename-workspace-title" className="text-lg font-semibold text-text">
          Rename workspace
        </h2>
        <p className="mt-1 text-sm text-text-muted">
          This changes only the displayed name. The branch and folder stay the same.
        </p>
        <form
          className="mt-4"
          onSubmit={(event) => {
            event.preventDefault();
            submit();
          }}
        >
          <label htmlFor="workspace-name" className="text-sm font-medium text-text">
            Workspace name
          </label>
          <input
            ref={inputRef}
            id="workspace-name"
            value={name}
            maxLength={60}
            disabled={mutation.isPending}
            onChange={(event) => setName(event.target.value)}
            className="mt-2 w-full rounded-md border border-border bg-surface-elevated px-3 py-2 text-text outline-none focus:border-accent disabled:opacity-60"
          />
          {errorMessage && <p className="mt-2 text-sm text-error" role="alert">{errorMessage}</p>}
          <div className="mt-5 flex justify-end gap-2">
            <button
              type="button"
              onClick={onClose}
              disabled={mutation.isPending}
              className="rounded-md px-4 py-2 text-sm text-text-muted hover:bg-surface-elevated disabled:opacity-60"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={!isValid || mutation.isPending}
              className="rounded-md bg-accent px-4 py-2 text-sm font-medium text-white disabled:opacity-50"
            >
              {mutation.isPending ? 'Renaming…' : 'Rename'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
