import { useEffect, useRef, useState } from 'react';
import type { Workspace, WorkspaceColor } from '../types';
import { ApiError } from '../lib/api';
import { useUpdateWorkspaceIdentity } from '../hooks/useApi';
import { cn } from '../lib/cn';

const ICONS = ['📁', '🎬', '⚙️', '🧪', '🚀', '💻', '🎨', '🔧'];
const COLORS: Array<{ value: WorkspaceColor; className: string }> = [
  { value: 'gray', className: 'bg-gray-400' },
  { value: 'blue', className: 'bg-blue-500' },
  { value: 'green', className: 'bg-green-500' },
  { value: 'orange', className: 'bg-orange-500' },
  { value: 'purple', className: 'bg-purple-500' },
  { value: 'red', className: 'bg-red-500' },
];

interface WorkspaceSettingsDialogProps {
  workspace: Workspace;
  onClose: () => void;
}

export function WorkspaceSettingsDialog({ workspace, onClose }: WorkspaceSettingsDialogProps) {
  const [name, setName] = useState(workspace.name);
  const [icon, setIcon] = useState(workspace.icon);
  const [color, setColor] = useState<WorkspaceColor>(workspace.color);
  const mutation = useUpdateWorkspaceIdentity();
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
      { id: workspace.id, data: { name, icon, color } },
      { onSuccess: onClose }
    );
  };

  const errorMessage = mutation.error
    ? mutation.error instanceof ApiError && mutation.error.status === 409
      ? 'A workspace with this name already exists in this repository.'
      : mutation.error.message || 'Unable to save workspace settings. Please try again.'
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
        aria-labelledby="workspace-settings-title"
        className="w-full max-w-md rounded-lg border border-border bg-surface p-5 shadow-xl"
      >
        <h2 id="workspace-settings-title" className="text-lg font-semibold text-text">
          Workspace settings
        </h2>
        <p className="mt-1 text-sm text-text-muted">
          Customize its display. The branch and folder stay the same.
        </p>
        <form className="mt-4 space-y-4" onSubmit={(event) => { event.preventDefault(); submit(); }}>
          <div>
            <label htmlFor="workspace-name" className="text-sm font-medium text-text">Name</label>
            <input ref={inputRef} id="workspace-name" value={name} maxLength={60}
              disabled={mutation.isPending} onChange={(event) => setName(event.target.value)}
              className="mt-2 w-full rounded-md border border-border bg-surface-elevated px-3 py-2 text-text outline-none focus:border-accent disabled:opacity-60" />
          </div>
          <fieldset>
            <legend className="text-sm font-medium text-text">Icon</legend>
            <div className="mt-2 flex flex-wrap gap-2">
              {ICONS.map((option) => (
                <button key={option} type="button" aria-label={`Icon ${option}`}
                  aria-pressed={icon === option} disabled={mutation.isPending}
                  onClick={() => setIcon(option)}
                  className={cn('size-10 rounded-md border text-xl', icon === option ? 'border-accent bg-accent/15' : 'border-border bg-surface-elevated')}>
                  {option}
                </button>
              ))}
            </div>
          </fieldset>
          <fieldset>
            <legend className="text-sm font-medium text-text">Color</legend>
            <div className="mt-2 flex flex-wrap gap-3">
              {COLORS.map((option) => (
                <button key={option.value} type="button" aria-label={`Color ${option.value}`}
                  aria-pressed={color === option.value} disabled={mutation.isPending}
                  onClick={() => setColor(option.value)}
                  className={cn('size-7 rounded-full border-2', option.className,
                    color === option.value ? 'border-white ring-2 ring-accent' : 'border-transparent')} />
              ))}
            </div>
          </fieldset>
          {errorMessage && <p className="text-sm text-error" role="alert">{errorMessage}</p>}
          <div className="flex justify-end gap-2 pt-1">
            <button type="button" onClick={onClose} disabled={mutation.isPending}
              className="rounded-md px-4 py-2 text-sm text-text-muted hover:bg-surface-elevated disabled:opacity-60">Cancel</button>
            <button type="submit" disabled={!isValid || mutation.isPending}
              className="rounded-md bg-accent px-4 py-2 text-sm font-medium text-white disabled:opacity-50">
              {mutation.isPending ? 'Saving…' : 'Save'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
