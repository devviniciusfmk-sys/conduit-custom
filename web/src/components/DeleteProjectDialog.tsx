import { useEffect, useRef, useState } from 'react';
import { AlertTriangle, Loader2, X } from 'lucide-react';
import type { Repository, RepositoryDeletePreflightResponse } from '../types';
import { cn } from '../lib/cn';

interface DeleteProjectDialogProps {
  repository: Repository | null;
  preflight?: RepositoryDeletePreflightResponse;
  isPending?: boolean;
  error?: string | null;
  /** Set once the server refused for lack of a system trash */
  requiresPermanent?: boolean;
  onConfirm: (permanent: boolean) => void;
  onClose: () => void;
}

/**
 * Confirmation for the only action that deletes a folder Conduit does not
 * manage. The confirm button stays disabled until the project's name is typed
 * exactly, so it cannot be reached by a stray click.
 */
export function DeleteProjectDialog({
  repository,
  preflight,
  isPending,
  error,
  requiresPermanent,
  onConfirm,
  onClose,
}: DeleteProjectDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const [typedName, setTypedName] = useState('');

  const isOpen = !!repository;
  const expectedName = repository?.name ?? '';
  const nameMatches = typedName === expectedName;
  const isBlocked = !!preflight?.blocked_reason;
  const canConfirm = isOpen && nameMatches && !isBlocked && !isPending;

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    if (isOpen) {
      dialog.showModal();
      requestAnimationFrame(() => inputRef.current?.focus());
    } else {
      dialog.close();
    }
  }, [isOpen]);

  // The typed name is never carried between projects: the caller keys this
  // component by repository id, so each opening starts from a fresh mount.

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    const handleCancel = (e: Event) => {
      e.preventDefault();
      if (!isPending) onClose();
    };
    dialog.addEventListener('cancel', handleCancel);
    return () => dialog.removeEventListener('cancel', handleCancel);
  }, [isPending, onClose]);

  const handleConfirm = () => {
    if (!canConfirm) return;
    onConfirm(!!requiresPermanent);
  };

  // Say plainly where the folder goes, since it decides whether this is
  // recoverable at all.
  const destination = requiresPermanent
    ? 'The system trash is unavailable, so the folder will be erased and cannot be recovered.'
    : preflight?.trash_available
      ? 'The folder will be moved to the system trash.'
      : 'The folder will be erased and cannot be recovered.';

  return (
    <dialog
      ref={dialogRef}
      onClick={(e) => {
        if (e.target === dialogRef.current && !isPending) onClose();
      }}
      className="m-auto w-full max-w-lg rounded-xl border border-border bg-surface p-0 shadow-xl backdrop:bg-black/50"
    >
      <div className="flex flex-col">
        <div className="flex items-center justify-between border-b border-border px-6 py-4">
          <h2 className="flex items-center gap-2 text-lg font-semibold text-text">
            <AlertTriangle className="h-5 w-5 text-red-400" />
            Delete "{expectedName}" permanently?
          </h2>
          <button
            onClick={onClose}
            disabled={isPending}
            className="rounded-md p-1 text-text-muted transition-colors hover:bg-surface-elevated hover:text-text disabled:opacity-50"
            aria-label="Close dialog"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        <div className="px-6 py-5">
          <p className="text-sm text-text-muted">
            This deletes the project folder with every workspace, branch and commit inside it.{' '}
            {destination}
          </p>

          {preflight?.project_path && (
            <p className="mt-3 break-all rounded-lg bg-surface-elevated px-3 py-2 font-mono text-xs text-text">
              {preflight.project_path}
            </p>
          )}

          {preflight?.blocked_reason ? (
            <div className="mt-4 rounded-lg bg-red-500/10 px-3 py-2 text-sm text-red-400">
              Cannot be deleted: {preflight.blocked_reason}
            </div>
          ) : (
            <>
              {preflight?.warnings && preflight.warnings.length > 0 && (
                <ul className="mt-4 space-y-1">
                  {preflight.warnings.map((warning) => (
                    <li key={warning} className="flex items-start gap-2 text-sm text-amber-400">
                      <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
                      <span>{warning}</span>
                    </li>
                  ))}
                </ul>
              )}

              <label className="mt-5 block text-sm text-text-muted" htmlFor="delete-project-name">
                Type <span className="font-mono text-text">{expectedName}</span> to confirm
              </label>
              <input
                id="delete-project-name"
                ref={inputRef}
                value={typedName}
                onChange={(event) => setTypedName(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter') {
                    event.preventDefault();
                    handleConfirm();
                  }
                }}
                autoComplete="off"
                className={cn(
                  'mt-1 w-full rounded-lg border bg-surface-elevated px-3 py-2 font-mono text-sm text-text',
                  'focus:outline-none focus:ring-2',
                  nameMatches
                    ? 'border-red-500/60 focus:border-red-500 focus:ring-red-500/30'
                    : 'border-border focus:border-accent focus:ring-accent/30'
                )}
                placeholder={expectedName}
              />
            </>
          )}

          {error && (
            <div className="mt-3 rounded-lg bg-red-500/10 px-3 py-2 text-sm text-red-400">
              {error}
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
            onClick={handleConfirm}
            disabled={!canConfirm}
            className={cn(
              'flex items-center gap-2 rounded-lg bg-red-500 px-4 py-2 text-sm font-medium text-white transition-colors',
              'hover:bg-red-400 disabled:cursor-not-allowed disabled:opacity-50',
              isPending && 'cursor-wait'
            )}
          >
            {isPending && <Loader2 className="h-4 w-4 animate-spin" />}
            {requiresPermanent ? 'Erase permanently' : 'Delete permanently'}
          </button>
        </div>
      </div>
    </dialog>
  );
}
