import { useEffect, useRef, useState } from 'react';
import { Loader2, X } from 'lucide-react';
import {
  useAddOnboardingProject,
  useCreateOnboardingProject,
  useOnboardingBaseDir,
} from '../../hooks';
import { cn } from '../../lib/cn';

interface AddProjectDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onAdded: () => void;
}

type Mode = 'existing' | 'new';

/** Client-side name check; the server enforces the same rules for real. */
function validateName(name: string): string | null {
  const trimmed = name.trim();
  if (!trimmed) return null;
  if (trimmed.includes('/') || trimmed.includes('\\')) {
    return "Project name cannot contain '/' or '\\'";
  }
  if (trimmed === '.' || trimmed === '..') {
    return 'Project name is reserved';
  }
  return null;
}

export function AddProjectDialog({ isOpen, onClose, onAdded }: AddProjectDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const nameInputRef = useRef<HTMLInputElement>(null);
  const addProject = useAddOnboardingProject();
  const createProject = useCreateOnboardingProject();
  const { data: baseDirData } = useOnboardingBaseDir({ enabled: isOpen });

  const [mode, setMode] = useState<Mode>('existing');
  const [value, setValue] = useState('');
  const [name, setName] = useState('');
  const [parent, setParent] = useState('');

  // Only the active tab's mutation drives the buttons and the error box
  const active = mode === 'existing' ? addProject : createProject;
  const isPending = active.isPending;
  const error = active.error;

  const nameError = validateName(name);
  const trimmedName = name.trim();
  const trimmedParent = parent.trim();
  const preview =
    trimmedName && trimmedParent
      ? `${trimmedParent.replace(/\/+$/, '')}/${trimmedName}`
      : null;

  const canSubmit =
    mode === 'existing'
      ? Boolean(value.trim())
      : Boolean(trimmedName && trimmedParent && !nameError);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    if (isOpen) {
      dialog.showModal();
      // Focus input after dialog animation
      requestAnimationFrame(() => {
        inputRef.current?.focus();
      });
    } else {
      dialog.close();
      addProject.reset();
      createProject.reset();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isOpen]);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    const handleCancel = (e: Event) => {
      e.preventDefault();
      if (!isPending) {
        onClose();
      }
    };
    dialog.addEventListener('cancel', handleCancel);
    return () => dialog.removeEventListener('cancel', handleCancel);
  }, [isPending, onClose]);

  // Reset every field each time the dialog opens
  const prefilledRef = useRef(false);
  useEffect(() => {
    if (!isOpen) {
      prefilledRef.current = false;
      return;
    }
    setMode('existing');
    setValue('');
    setName('');
    setParent('');
  }, [isOpen]);

  // Prefill the parent folder with the configured projects directory, once per
  // opening, so typing into the field is never overwritten.
  const baseDir = baseDirData?.base_dir ?? '';
  useEffect(() => {
    if (!isOpen || prefilledRef.current || !baseDir) return;
    prefilledRef.current = true;
    setParent(baseDir);
  }, [isOpen, baseDir]);

  const switchMode = (next: Mode) => {
    if (isPending || next === mode) return;
    // Drop the other tab's error so it does not linger over this form
    addProject.reset();
    createProject.reset();
    setMode(next);
    requestAnimationFrame(() => {
      if (next === 'existing') {
        inputRef.current?.focus();
      } else {
        nameInputRef.current?.focus();
      }
    });
  };

  const handleSubmit = () => {
    if (!canSubmit || isPending) return;
    if (mode === 'existing') {
      addProject.mutate({ path: value.trim() }, { onSuccess: () => onAdded() });
    } else {
      createProject.mutate(
        { name: trimmedName, parent: trimmedParent },
        { onSuccess: () => onAdded() }
      );
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSubmit();
    }
  };

  const handleBackdropClick = (e: React.MouseEvent<HTMLDialogElement>) => {
    if (e.target === dialogRef.current && !isPending) {
      onClose();
    }
  };

  const tabClass = (tab: Mode) =>
    cn(
      'rounded-full px-3 py-1 text-sm transition-colors disabled:opacity-50',
      mode === tab
        ? 'bg-accent/20 text-text'
        : 'text-text-muted hover:bg-surface-elevated hover:text-text'
    );

  const inputClass = cn(
    'w-full rounded-lg border border-border bg-surface-elevated px-3 py-2 text-sm text-text',
    'focus:border-accent focus:outline-none focus:ring-2 focus:ring-accent/30'
  );

  return (
    <dialog
      ref={dialogRef}
      onClick={handleBackdropClick}
      className="m-auto w-full max-w-lg rounded-xl border border-border bg-surface p-0 shadow-xl backdrop:bg-black/50"
    >
      <div className="flex flex-col">
        <div className="flex items-center justify-between border-b border-border px-6 py-4">
          <h2 className="text-lg font-semibold text-text">Add custom project</h2>
          <button
            onClick={onClose}
            disabled={isPending}
            className="rounded-md p-1 text-text-muted transition-colors hover:bg-surface-elevated hover:text-text disabled:opacity-50"
            aria-label="Close dialog"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        <div className="flex gap-2 border-b border-border px-6 py-3" role="tablist">
          <button
            role="tab"
            aria-selected={mode === 'existing'}
            disabled={isPending}
            onClick={() => switchMode('existing')}
            className={tabClass('existing')}
          >
            Add existing
          </button>
          <button
            role="tab"
            aria-selected={mode === 'new'}
            disabled={isPending}
            onClick={() => switchMode('new')}
            className={tabClass('new')}
          >
            Create new project
          </button>
        </div>

        <div className="px-6 py-5">
          {mode === 'existing' ? (
            <>
              <p className="text-sm text-text-muted">Enter the path to a local git repository.</p>
              <input
                ref={inputRef}
                value={value}
                onChange={(event) => setValue(event.target.value)}
                onKeyDown={handleKeyDown}
                className={cn('mt-4', inputClass)}
                placeholder="/Users/you/projects/my-repo"
              />
            </>
          ) : (
            <>
              <p className="text-sm text-text-muted">
                Creates the folder, runs <code className="text-text">git init</code>, adds a README
                and makes the first commit.
              </p>

              <label className="mt-4 block text-sm text-text-muted" htmlFor="new-project-name">
                Project name
              </label>
              <input
                id="new-project-name"
                ref={nameInputRef}
                value={name}
                onChange={(event) => setName(event.target.value)}
                onKeyDown={handleKeyDown}
                className={cn('mt-1', inputClass)}
                placeholder="my-app"
              />

              <label className="mt-3 block text-sm text-text-muted" htmlFor="new-project-parent">
                Parent folder
              </label>
              <input
                id="new-project-parent"
                value={parent}
                onChange={(event) => setParent(event.target.value)}
                onKeyDown={handleKeyDown}
                className={cn('mt-1', inputClass)}
                placeholder="/Users/you/projects"
              />

              {nameError ? (
                <p className="mt-3 text-sm text-red-400">{nameError}</p>
              ) : (
                preview && (
                  <p className="mt-3 text-sm text-text-muted">
                    Will create: <span className="text-text">{preview}</span>
                  </p>
                )
              )}
            </>
          )}

          {error && (
            <div className="mt-3 rounded-lg bg-red-500/10 px-3 py-2 text-sm text-red-400">
              {error instanceof Error
                ? error.message
                : mode === 'existing'
                ? 'Failed to add project'
                : 'Failed to create project'}
            </div>
          )}
        </div>

        <div className="flex justify-end gap-3 border-t border-border px-6 py-4">
          <button
            onClick={onClose}
            disabled={isPending}
            className="rounded-lg px-4 py-2 text-sm font-medium text-text-muted transition-colors hover:bg-surface-elevated hover:text-text disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={isPending || !canSubmit}
            className={cn(
              'flex items-center gap-2 rounded-lg bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-70',
              isPending && 'cursor-wait'
            )}
          >
            {isPending && <Loader2 className="h-4 w-4 animate-spin" />}
            {mode === 'existing' ? 'Add project' : 'Create project'}
          </button>
        </div>
      </div>
    </dialog>
  );
}
