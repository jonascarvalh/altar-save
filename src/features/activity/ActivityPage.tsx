import { useEffect, useMemo, useState, type ReactNode } from "react";
import {
  AlertTriangle,
  ArchiveRestore,
  CloudDownload,
  CloudUpload,
  History,
  Info,
  PauseCircle,
  Trash2,
  XCircle,
} from "lucide-react";
import { Page, PageHeader } from "@/components/ui/PageHeader";
import { Card } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { EmptyState } from "@/components/ui/EmptyState";
import { toast } from "@/components/ui/Toaster";
import { api } from "@/lib/api/commands";
import { errorMessage } from "@/lib/api/errors";
import type { ActivityEntry, ActivityKind } from "@/lib/api/types";
import { useSync } from "@/stores/syncStore";
import { cn, formatDateTime, formatRelative } from "@/lib/format";

const KIND_META: Record<ActivityKind, { label: string; icon: ReactNode; className: string }> = {
  upload: { label: "Envio", icon: <CloudUpload className="size-4" />, className: "bg-info/10 text-info" },
  download: { label: "Download", icon: <CloudDownload className="size-4" />, className: "bg-success/10 text-success" },
  delete: { label: "Exclusão", icon: <Trash2 className="size-4" />, className: "bg-danger/10 text-danger" },
  conflict: { label: "Conflito", icon: <AlertTriangle className="size-4" />, className: "bg-warning/10 text-warning" },
  deferred: { label: "Aguardando", icon: <PauseCircle className="size-4" />, className: "bg-accent/10 text-accent" },
  error: { label: "Erro", icon: <XCircle className="size-4" />, className: "bg-danger/10 text-danger" },
  info: { label: "Info", icon: <Info className="size-4" />, className: "bg-tint/5 text-muted" },
};

type Filter = "all" | "upload" | "download" | "delete" | "conflict" | "error";

const FILTERS: { id: Filter; label: string }[] = [
  { id: "all", label: "Tudo" },
  { id: "upload", label: "Envios" },
  { id: "download", label: "Downloads" },
  { id: "delete", label: "Exclusões" },
  { id: "conflict", label: "Conflitos" },
  { id: "error", label: "Erros" },
];

export function ActivityPage() {
  const revision = useSync((s) => s.revision);
  const [entries, setEntries] = useState<ActivityEntry[]>([]);
  const [filter, setFilter] = useState<Filter>("all");

  useEffect(() => {
    api
      .getActivity(200)
      .then(setEntries)
      .catch((e) => toast.error("Falha ao carregar atividade", errorMessage(e)));
  }, [revision]);

  const visible = useMemo(
    () => (filter === "all" ? entries : entries.filter((e) => e.kind === filter)),
    [entries, filter],
  );

  return (
    <Page>
      <PageHeader
        title="Atividade"
        description="Histórico das sincronizações. Antes de qualquer download, a versão local vai para a pasta de backups."
        actions={
          <Button
            icon={<ArchiveRestore className="size-4" />}
            onClick={() => api.openBackupsFolder().catch((e) => toast.error("Erro", errorMessage(e)))}
          >
            Abrir backups
          </Button>
        }
      />

      <div className="flex gap-1.5">
        {FILTERS.map((f) => (
          <button
            key={f.id}
            type="button"
            onClick={() => setFilter(f.id)}
            className={cn(
              "h-8 rounded-lg px-3 text-xs font-medium transition-colors",
              filter === f.id ? "bg-tint/8 text-fg" : "text-muted hover:bg-tint/4 hover:text-fg",
            )}
          >
            {f.label}
          </button>
        ))}
      </div>

      {visible.length === 0 ? (
        <EmptyState
          icon={<History className="size-5" />}
          title="Nada por aqui ainda"
          description="Quando seus saves forem sincronizados, o histórico aparece aqui."
        />
      ) : (
        <Card className="divide-y divide-line">
          {visible.map((entry) => {
            const meta = KIND_META[entry.kind];
            return (
              <div key={entry.id} className="flex items-start gap-4 px-5 py-3.5">
                <div className={cn("grid size-8 shrink-0 place-items-center rounded-lg", meta.className)}>{meta.icon}</div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <p className="truncate text-[13px] font-medium text-fg">{entry.title}</p>
                    <span className="text-[11px] text-subtle">{meta.label}</span>
                  </div>
                  <p className="mt-0.5 text-xs break-words text-muted">{entry.message}</p>
                </div>
                <time className="shrink-0 text-[11px] text-subtle" title={formatDateTime(entry.at)}>
                  {formatRelative(entry.at)}
                </time>
              </div>
            );
          })}
        </Card>
      )}
    </Page>
  );
}
