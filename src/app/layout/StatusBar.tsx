import { Eye, EyeOff, Loader2, CheckCircle2, XCircle } from "lucide-react";
import { useSync } from "@/stores/syncStore";
import { formatTime } from "@/lib/format";
import type { SyncSummary } from "@/lib/api/types";

const PHASE_LABEL = {
  checking: "Verificando",
  uploading: "Enviando",
  downloading: "Baixando",
  deleting: "Excluindo",
  done: "Concluído",
};

export function StatusBar() {
  const running = useSync((s) => s.running);
  const lastSummary = useSync((s) => s.lastSummary);
  const watching = useSync((s) => s.watching);
  const library = useSync((s) => s.library);

  const active = Object.entries(running)[0];
  const latest = Object.values(lastSummary).sort((a, b) => (b.finishedAt ?? "").localeCompare(a.finishedAt ?? ""))[0];
  const watchedNames = library.filter((s) => watching[s.info.id]).map((s) => s.info.name);

  return (
    <footer className="flex h-8 shrink-0 items-center gap-4 border-t border-line bg-surface px-4 text-[11px] text-muted">
      <div className="flex min-w-0 flex-1 items-center gap-2">
        {active ? (
          <ActiveSync sourceName={library.find((s) => s.info.id === active[0])?.info.name ?? active[0]} {...active[1]} />
        ) : latest ? (
          <LastSync summary={latest} />
        ) : (
          <span>Nenhuma sincronização nesta sessão</span>
        )}
      </div>

      <div className="flex items-center gap-1.5" title="Monitoramento automático de mudanças">
        {watchedNames.length > 0 ? (
          <>
            <Eye className="size-3.5 text-accent" />
            <span>Monitorando {watchedNames.join(", ")}</span>
          </>
        ) : (
          <>
            <EyeOff className="size-3.5" />
            <span>Sync automático inativo</span>
          </>
        )}
      </div>
      <span className="text-subtle">v0.1.0</span>
    </footer>
  );
}

function ActiveSync({
  sourceName,
  total,
  index,
  title,
  phase,
}: {
  sourceName: string;
  total: number;
  index: number;
  title: string | null;
  phase: keyof typeof PHASE_LABEL | null;
}) {
  const progress = total > 0 ? ((index + (phase === "done" ? 1 : 0.5)) / total) * 100 : 0;
  return (
    <>
      <Loader2 className="size-3.5 animate-spin text-accent" />
      <span className="truncate text-fg">
        {sourceName}
        {title && (
          <span className="text-muted">
            {" "}
            · {PHASE_LABEL[phase ?? "checking"]} {title} ({Math.min(index + 1, total)}/{total})
          </span>
        )}
      </span>
      <div className="h-1 w-32 overflow-hidden rounded-full bg-white/8">
        <div className="h-full bg-accent-gradient transition-all duration-300" style={{ width: `${progress}%` }} />
      </div>
    </>
  );
}

function LastSync({ summary }: { summary: SyncSummary }) {
  if (summary.error) {
    return (
      <>
        <XCircle className="size-3.5 text-danger" />
        <span className="truncate">Último sync às {formatTime(summary.finishedAt)} falhou: {summary.error}</span>
      </>
    );
  }
  const parts = [
    summary.uploaded && `${summary.uploaded} enviado(s)`,
    summary.downloaded && `${summary.downloaded} baixado(s)`,
    summary.deleted && `${summary.deleted} excluído(s)`,
    summary.conflicts && `${summary.conflicts} conflito(s)`,
    summary.deferred && `${summary.deferred} aguardando`,
    summary.errors && `${summary.errors} erro(s)`,
  ].filter(Boolean);
  return (
    <>
      <CheckCircle2 className="size-3.5 text-success" />
      <span className="truncate">
        Último sync às {formatTime(summary.finishedAt)} · {parts.length ? parts.join(" · ") : "tudo em dia"}
      </span>
    </>
  );
}
