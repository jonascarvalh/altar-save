import type { ReactNode } from "react";
import { CheckCircle2, Info, TriangleAlert, X, XCircle } from "lucide-react";
import { create } from "zustand";
import { cn } from "@/lib/format";

type ToastTone = "success" | "error" | "info" | "warning";

interface Toast {
  id: number;
  tone: ToastTone;
  title: string;
  description?: string;
}

interface ToastStore {
  toasts: Toast[];
  push: (toast: Omit<Toast, "id">) => void;
  dismiss: (id: number) => void;
}

let nextId = 1;

export const useToasts = create<ToastStore>((set, get) => ({
  toasts: [],
  push: (toast) => {
    const id = nextId++;
    set({ toasts: [...get().toasts.slice(-3), { ...toast, id }] });
    setTimeout(() => get().dismiss(id), toast.tone === "error" ? 7000 : 4000);
  },
  dismiss: (id) => set({ toasts: get().toasts.filter((t) => t.id !== id) }),
}));

export const toast = {
  success: (title: string, description?: string) => useToasts.getState().push({ tone: "success", title, description }),
  error: (title: string, description?: string) => useToasts.getState().push({ tone: "error", title, description }),
  info: (title: string, description?: string) => useToasts.getState().push({ tone: "info", title, description }),
  warning: (title: string, description?: string) => useToasts.getState().push({ tone: "warning", title, description }),
};

const icons: Record<ToastTone, ReactNode> = {
  success: <CheckCircle2 className="size-4 text-success" />,
  error: <XCircle className="size-4 text-danger" />,
  info: <Info className="size-4 text-info" />,
  warning: <TriangleAlert className="size-4 text-warning" />,
};

export function Toaster() {
  const { toasts, dismiss } = useToasts();
  return (
    <div className="pointer-events-none fixed right-5 bottom-12 z-50 flex w-80 flex-col gap-2">
      {toasts.map((t) => (
        <div
          key={t.id}
          className={cn(
            "pointer-events-auto flex animate-slide-up items-start gap-3 rounded-xl border border-line-strong",
            "bg-surface-3/95 px-4 py-3 shadow-2xl shadow-black/50 backdrop-blur",
          )}
        >
          <div className="mt-0.5">{icons[t.tone]}</div>
          <div className="min-w-0 flex-1">
            <p className="text-[13px] font-medium text-fg">{t.title}</p>
            {t.description && <p className="mt-0.5 text-xs break-words text-muted">{t.description}</p>}
          </div>
          <button onClick={() => dismiss(t.id)} className="text-subtle hover:text-fg" aria-label="Fechar">
            <X className="size-3.5" />
          </button>
        </div>
      ))}
    </div>
  );
}
