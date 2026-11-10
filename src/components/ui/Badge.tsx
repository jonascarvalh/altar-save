import type { ReactNode } from "react";
import { AlertTriangle, CheckCircle2, CircleDashed, CloudUpload, PauseCircle, XCircle } from "lucide-react";
import type { UnitStatus } from "@/lib/api/types";
import { cn } from "@/lib/format";

export type Tone = "neutral" | "accent" | "success" | "warning" | "danger" | "info";

const tones: Record<Tone, string> = {
  neutral: "bg-tint/5 text-muted border-tint/8",
  accent: "bg-accent/10 text-accent border-accent/20",
  success: "bg-success/10 text-success border-success/20",
  warning: "bg-warning/10 text-warning border-warning/20",
  danger: "bg-danger/10 text-danger border-danger/20",
  info: "bg-info/10 text-info border-info/20",
};

export function Badge({ tone = "neutral", icon, children, className }: {
  tone?: Tone;
  icon?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-flex h-6 items-center gap-1 rounded-md border px-2 text-[11px] font-medium whitespace-nowrap",
        tones[tone],
        className,
      )}
    >
      {icon}
      {children}
    </span>
  );
}

export const STATUS_META: Record<UnitStatus, { label: string; tone: Tone; icon: ReactNode }> = {
  synced: { label: "Em dia", tone: "success", icon: <CheckCircle2 className="size-3.5" /> },
  pending: { label: "Pendente", tone: "info", icon: <CloudUpload className="size-3.5" /> },
  conflict: { label: "Conflito", tone: "warning", icon: <AlertTriangle className="size-3.5" /> },
  deferred: { label: "Aguardando", tone: "accent", icon: <PauseCircle className="size-3.5" /> },
  error: { label: "Erro", tone: "danger", icon: <XCircle className="size-3.5" /> },
  unknown: { label: "Não sincronizado", tone: "neutral", icon: <CircleDashed className="size-3.5" /> },
};

export function StatusBadge({ status }: { status: UnitStatus }) {
  const meta = STATUS_META[status];
  return (
    <Badge tone={meta.tone} icon={meta.icon}>
      {meta.label}
    </Badge>
  );
}
