import { useNavigate } from "react-router";
import { AlertTriangle, CloudOff, FolderOpen, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { toast } from "@/components/ui/Toaster";
import { api } from "@/lib/api/commands";
import { errorMessage } from "@/lib/api/errors";
import type { SourceOverview, UnitStatus } from "@/lib/api/types";
import { useSettings } from "@/stores/settingsStore";
import { useSync } from "@/stores/syncStore";
import { formatRelative } from "@/lib/format";

/** Banner de destaque da fonte, no estilo das páginas de jogo de launchers. */
export function SourceHero({ source }: { source: SourceOverview }) {
  const navigate = useNavigate();
  const cloud = useSettings((s) => s.settings?.cloud);
  const running = useSync((s) => s.running[source.info.id]);
  const syncNow = useSync((s) => s.syncNow);

  const count = (status: UnitStatus) => source.units.filter((u) => u.status === status).length;
  const lastSynced = source.units
    .map((u) => u.lastSyncedAt)
    .filter(Boolean)
    .sort()
    .at(-1);

  const sync = async () => {
    try {
      const summary = await syncNow(source.info.id);
      const changed = summary.uploaded + summary.downloaded + summary.deleted;
      if (summary.conflicts > 0) {
        toast.warning("Sincronizado com conflitos", `${summary.conflicts} save(s) precisam da sua decisão.`);
      } else if (summary.errors > 0) {
        toast.error("Sincronizado com erros", `${summary.errors} save(s) falharam. Veja a Atividade.`);
      } else {
        toast.success("Sincronização concluída", changed > 0 ? `${changed} save(s) atualizados.` : "Tudo já estava em dia.");
      }
    } catch (e) {
      toast.error("Falha na sincronização", errorMessage(e));
    }
  };

  return (
    <div className="relative overflow-hidden rounded-2xl border border-line bg-surface">
      {/* Arte de fundo */}
      <div className="absolute inset-0 bg-grid opacity-60" />
      <div className="absolute -top-24 -left-16 size-80 rounded-full bg-[#1f6feb]/25 blur-3xl" />
      <div className="absolute -right-10 -bottom-32 size-96 rounded-full bg-accent-2/20 blur-3xl" />
      <div className="absolute top-6 right-8 font-black tracking-tighter text-tint/[0.035] select-none text-[140px] leading-none">
        PS2
      </div>
      <div className="absolute inset-0 bg-gradient-to-t from-surface via-surface/40 to-transparent" />

      <div className="relative flex flex-col gap-6 p-7">
        <div className="flex flex-wrap items-end justify-between gap-6">
          <div className="min-w-0">
            <p className="text-[11px] font-semibold tracking-[0.16em] text-accent uppercase">Automação · Memory cards</p>
            <h1 className="mt-2 text-4xl font-bold tracking-tight text-fg">{source.info.name}</h1>
            <p className="mt-2 max-w-xl truncate font-mono text-xs text-muted" title={source.info.savesDir ?? ""}>
              {source.info.savesDir ?? "Pasta não detectada"}
            </p>
          </div>

          <div className="flex items-center gap-2">
            {source.info.detected && (
              <Button
                icon={<FolderOpen className="size-4" />}
                onClick={() => api.openSavesFolder(source.info.id).catch((e) => toast.error("Erro", errorMessage(e)))}
              >
                Abrir pasta
              </Button>
            )}
            {cloud ? (
              <Button
                variant="primary"
                size="lg"
                icon={<RefreshCw className={running ? "size-4 animate-spin" : "size-4"} />}
                disabled={!!running || !source.enabled || !source.info.detected}
                onClick={sync}
              >
                {running ? "Sincronizando…" : "Sincronizar agora"}
              </Button>
            ) : (
              <Button variant="primary" size="lg" icon={<CloudOff className="size-4" />} onClick={() => navigate("/cloud")}>
                Configurar nuvem
              </Button>
            )}
          </div>
        </div>

        <div className="flex flex-wrap gap-2">
          <Stat label="Saves" value={source.units.length} />
          <Stat label="Em dia" value={count("synced")} tone="text-success" />
          <Stat label="Pendentes" value={count("pending") + count("unknown")} tone="text-info" />
          {count("conflict") > 0 && <Stat label="Conflitos" value={count("conflict")} tone="text-warning" />}
          {count("deferred") > 0 && <Stat label="Aguardando" value={count("deferred")} tone="text-accent" />}
          {count("error") > 0 && <Stat label="Erros" value={count("error")} tone="text-danger" />}
          <Stat label="Último sync" value={formatRelative(lastSynced)} />
        </div>

        {count("deferred") > 0 && (
          <div className="flex items-center gap-2 rounded-lg border border-accent/20 bg-accent/8 px-3 py-2 text-xs text-accent">
            <AlertTriangle className="size-4" />
            {count("deferred") === 1 ? "1 save aguarda" : `${count("deferred")} saves aguardam`} o PCSX2 ser fechado
            para sincronizar. Isso acontece automaticamente.
          </div>
        )}
      </div>
    </div>
  );
}

function Stat({ label, value, tone = "text-fg" }: { label: string; value: string | number; tone?: string }) {
  return (
    <div className="rounded-lg border border-line bg-bg/50 px-3.5 py-2 backdrop-blur">
      <p className="text-[10px] font-medium tracking-wide text-subtle uppercase">{label}</p>
      <p className={`mt-0.5 text-sm font-semibold ${tone}`}>{value}</p>
    </div>
  );
}
