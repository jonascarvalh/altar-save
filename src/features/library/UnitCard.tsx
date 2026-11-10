import { useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  CloudDownload,
  CloudUpload,
  Gamepad2,
  HardDrive,
  Loader2,
  MemoryStick,
  Monitor,
  Trash2,
} from "lucide-react";
import { StatusBadge } from "@/components/ui/Badge";
import { Button } from "@/components/ui/Button";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";
import { toast } from "@/components/ui/Toaster";
import { errorMessage } from "@/lib/api/errors";
import type { Resolution, UnitView } from "@/lib/api/types";
import { useSync } from "@/stores/syncStore";
import { cn, formatBytes, formatRelative, hueFrom } from "@/lib/format";

const PHASE_LABEL = {
  checking: "Verificando…",
  uploading: "Enviando…",
  downloading: "Baixando…",
  deleting: "Excluindo…",
  done: "",
};

const RESOLVED_MESSAGE: Record<Resolution, (title: string, remoteDeleted: boolean) => string> = {
  keepLocal: (t, deleted) => (deleted ? `"${t}" foi mantido e reenviado para a nuvem.` : `"${t}" desta máquina foi enviado.`),
  keepCloud: (t, deleted) => (deleted ? `"${t}" foi excluído daqui também.` : `"${t}" foi baixado da nuvem.`),
};

export function UnitCard({ unit }: { unit: UnitView }) {
  const phase = useSync((s) => s.unitPhases[unit.id]);
  const resolve = useSync((s) => s.resolve);
  const deleteSave = useSync((s) => s.deleteSave);
  const [resolving, setResolving] = useState<Resolution | null>(null);
  const [coverFailed, setCoverFailed] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const [deleting, setDeleting] = useState(false);

  const hue = hueFrom(unit.subtitle ?? unit.title);
  const cover = unit.coverPath && !coverFailed ? convertFileSrc(unit.coverPath) : null;
  const busy = phase === "uploading" || phase === "downloading" || phase === "deleting";

  const onResolve = async (resolution: Resolution) => {
    setResolving(resolution);
    try {
      await resolve(unit.id, resolution);
      toast.success("Conflito resolvido", RESOLVED_MESSAGE[resolution](unit.title, unit.remoteDeleted));
    } catch (e) {
      toast.error("Não foi possível resolver", errorMessage(e));
    } finally {
      setResolving(null);
    }
  };

  const onDelete = async () => {
    setDeleting(true);
    try {
      await deleteSave(unit.id);
      setConfirmingDelete(false);
      toast.success("Save excluído", `"${unit.title}" foi removido${unit.existsLocally ? " daqui e" : ""} da nuvem.`);
    } catch (e) {
      toast.error("Não foi possível excluir", errorMessage(e));
    } finally {
      setDeleting(false);
    }
  };

  return (
    <article
      className={cn(
        "group flex flex-col overflow-hidden rounded-xl border bg-surface transition-all duration-200",
        "hover:-translate-y-0.5 hover:border-line-strong hover:shadow-[0_16px_40px_-20px_rgb(0_0_0/0.8)]",
        unit.status === "conflict" ? "border-warning/30" : "border-line",
      )}
    >
      <div
        className="relative h-28 overflow-hidden"
        style={
          cover
            ? { background: "#15171d" }
            : {
                // "Capa" gerada a partir do nome do save, quando não há arte do emulador.
                background: `radial-gradient(120% 120% at 0% 0%, hsl(${hue} 70% 45% / 0.55), transparent 60%),
                             radial-gradient(120% 120% at 100% 100%, hsl(${(hue + 60) % 360} 70% 40% / 0.45), transparent 60%),
                             #15171d`,
              }
        }
      >
        {cover ? (
          <>
            <img
              src={cover}
              alt=""
              aria-hidden
              className="absolute inset-0 size-full scale-125 object-cover opacity-50 blur-xl"
              onError={() => setCoverFailed(true)}
            />
            <div className="absolute inset-0 bg-gradient-to-r from-black/50 via-black/20 to-black/40" />
            <img
              src={cover}
              alt={`Capa de ${unit.title}`}
              className="absolute top-3 bottom-3 left-3 aspect-[3/4] h-[calc(100%-24px)] rounded-md object-cover shadow-[0_8px_20px_-6px_rgb(0_0_0/0.8)] ring-1 ring-white/15 transition-transform duration-300 group-hover:scale-[1.04]"
            />
          </>
        ) : (
          <>
            <div className="absolute inset-0 bg-grid opacity-40" />
            <div className="absolute top-3 left-3 grid size-10 place-items-center rounded-lg bg-black/35 text-white/90 backdrop-blur">
              {unit.kind === "directory" ? <Gamepad2 className="size-5" /> : <MemoryStick className="size-5" />}
            </div>
          </>
        )}
        <div className="absolute top-3 right-3">
          <StatusBadge status={unit.status} />
        </div>
        {unit.subtitle && (
          <p
            className={cn(
              "absolute bottom-2.5 font-mono text-[11px] font-medium tracking-wide text-white/60",
              cover ? "left-[88px]" : "left-3",
            )}
          >
            {unit.subtitle}
          </p>
        )}
        {busy && (
          <div className="absolute inset-0 grid place-items-center bg-black/50 backdrop-blur-[2px]">
            <div className="flex items-center gap-2 text-xs font-medium text-fg">
              {phase === "uploading" ? <CloudUpload className="size-4 text-accent" /> : <CloudDownload className="size-4 text-accent" />}
              {PHASE_LABEL[phase]}
            </div>
            <div className="absolute right-0 bottom-0 left-0 h-0.5 animate-shimmer bg-[linear-gradient(90deg,transparent,var(--color-accent),transparent)] bg-[length:200%_100%]" />
          </div>
        )}
        {phase === "checking" && (
          <Loader2 className="absolute right-3 bottom-2.5 size-3.5 animate-spin text-white/70" />
        )}
      </div>

      <div className="flex flex-1 flex-col gap-2 p-4">
        <h3 className="line-clamp-2 text-[13px] leading-snug font-semibold text-fg" title={unit.title}>
          {unit.title}
        </h3>

        <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px] text-muted">
          {unit.existsLocally ? (
            <>
              <span className="inline-flex items-center gap-1">
                <HardDrive className="size-3" />
                {formatBytes(unit.size)}
              </span>
              <span title={unit.modifiedAt ?? ""}>Alterado {formatRelative(unit.modifiedAt)}</span>
            </>
          ) : (
            <span className="inline-flex items-center gap-1 text-accent">
              <CloudDownload className="size-3" /> Somente na nuvem
            </span>
          )}
        </div>

        <div className="flex items-center gap-2">
          <p className="flex min-w-0 flex-1 flex-wrap items-center gap-x-1.5 text-[11px] text-subtle">
            <span>Sync {formatRelative(unit.lastSyncedAt)}</span>
            {unit.remoteMachine && (
              <>
                <span>·</span>
                <span className="inline-flex items-center gap-1">
                  <Monitor className="size-3" /> {unit.remoteMachine}
                </span>
              </>
            )}
          </p>
          <button
            type="button"
            title="Excluir save"
            aria-label={`Excluir ${unit.title}`}
            disabled={busy}
            onClick={() => setConfirmingDelete(true)}
            className="grid size-7 shrink-0 place-items-center rounded-md text-subtle opacity-0 transition-all group-hover:opacity-100 hover:bg-danger/10 hover:text-danger focus-visible:opacity-100 disabled:hidden"
          >
            <Trash2 className="size-3.5" />
          </button>
        </div>

        {unit.message && unit.status !== "synced" && (
          <p
            className={cn(
              "rounded-md px-2.5 py-1.5 text-[11px] leading-relaxed",
              unit.status === "error" ? "bg-danger/8 text-danger" : "bg-white/4 text-muted",
            )}
          >
            {unit.message}
          </p>
        )}

        {unit.status === "conflict" && (
          <div className="mt-auto grid grid-cols-2 gap-2 pt-2">
            <Button
              size="sm"
              loading={resolving === "keepLocal"}
              disabled={!!resolving || !unit.existsLocally}
              onClick={() => onResolve("keepLocal")}
              title={
                unit.remoteDeleted
                  ? "Desfaz a exclusão: reenvia este save para a nuvem"
                  : "Envia a versão deste computador e substitui a da nuvem"
              }
            >
              {unit.remoteDeleted ? "Manter" : "Manter local"}
            </Button>
            <Button
              size="sm"
              loading={resolving === "keepCloud"}
              disabled={!!resolving}
              onClick={() => onResolve("keepCloud")}
              variant={unit.remoteDeleted ? "danger" : "secondary"}
              title={
                unit.remoteDeleted
                  ? "Apaga daqui também (uma cópia fica nos backups)"
                  : "Baixa a versão da nuvem (a local vai para o backup)"
              }
            >
              {unit.remoteDeleted ? "Excluir" : "Manter nuvem"}
            </Button>
          </div>
        )}
      </div>

      <ConfirmDialog
        open={confirmingDelete}
        danger
        icon={<Trash2 className="size-5" />}
        title={`Excluir "${unit.title}"?`}
        confirmLabel="Excluir save"
        loading={deleting}
        onConfirm={onDelete}
        onCancel={() => setConfirmingDelete(false)}
      >
        <p>
          {unit.existsLocally
            ? "O save será apagado deste computador e da nuvem."
            : "O save será apagado da nuvem."}{" "}
          As outras máquinas apagam a cópia delas no próximo sync. Se alguma tiver alterações, ela vai perguntar antes.
        </p>
        {unit.kind === "file" && (
          <p className="mt-2 text-warning">
            Este é um memory card inteiro: todos os jogos salvos nele serão excluídos.
          </p>
        )}
        {unit.existsLocally && <p className="mt-2 text-subtle">Uma cópia fica na pasta de backups deste computador.</p>}
      </ConfirmDialog>
    </article>
  );
}
