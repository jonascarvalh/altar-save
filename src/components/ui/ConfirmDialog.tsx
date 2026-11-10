import { useEffect, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { Button } from "./Button";

interface ConfirmDialogProps {
  open: boolean;
  icon?: ReactNode;
  title: string;
  children: ReactNode;
  confirmLabel: string;
  danger?: boolean;
  loading?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  open,
  icon,
  title,
  children,
  confirmLabel,
  danger,
  loading,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && !loading && onCancel();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, loading, onCancel]);

  if (!open) return null;

  // Portal: um ancestral com `transform` (ex.: hover dos cards) prenderia o `fixed` dentro dele.
  return createPortal(
    <div
      className="fixed inset-0 z-50 grid animate-fade-in place-items-center bg-black/60 p-6 backdrop-blur-sm"
      onMouseDown={(e) => e.target === e.currentTarget && !loading && onCancel()}
    >
      <div
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="confirm-title"
        className="w-full max-w-md animate-slide-up rounded-2xl border border-line-strong bg-surface-2 p-6 shadow-2xl shadow-black/60"
      >
        <div className="flex gap-4">
          {icon && (
            <div
              className={
                danger
                  ? "grid size-10 shrink-0 place-items-center rounded-xl bg-danger/10 text-danger"
                  : "grid size-10 shrink-0 place-items-center rounded-xl bg-surface-3 text-accent"
              }
            >
              {icon}
            </div>
          )}
          <div className="min-w-0 flex-1">
            <h2 id="confirm-title" className="text-[15px] font-semibold text-fg">
              {title}
            </h2>
            <div className="mt-2 text-[13px] leading-relaxed text-muted">{children}</div>
          </div>
        </div>
        <div className="mt-6 flex justify-end gap-2">
          <Button variant="ghost" onClick={onCancel} disabled={loading} autoFocus>
            Cancelar
          </Button>
          <Button
            variant={danger ? "danger" : "primary"}
            className={danger ? "bg-danger/15 hover:bg-danger/25" : undefined}
            loading={loading}
            onClick={onConfirm}
          >
            {confirmLabel}
          </Button>
        </div>
      </div>
    </div>,
    document.body,
  );
}
