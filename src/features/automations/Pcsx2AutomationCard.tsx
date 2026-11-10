import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { CheckCircle2, FolderOpen, Gamepad2, HardDrive, RotateCcw, Save, Search, TriangleAlert } from "lucide-react";
import { cn } from "@/lib/format";
import { Card, CardHeader } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { Switch } from "@/components/ui/Switch";
import { TextField } from "@/components/ui/TextField";
import { Badge } from "@/components/ui/Badge";
import { toast } from "@/components/ui/Toaster";
import { api } from "@/lib/api/commands";
import { errorMessage } from "@/lib/api/errors";
import type { Pcsx2Settings, SourceInfo } from "@/lib/api/types";
import { useSettings } from "@/stores/settingsStore";
import { useSync } from "@/stores/syncStore";

const DEFAULTS: Pcsx2Settings = { enabled: true, autoSync: true, dataDir: null };

export function Pcsx2AutomationCard() {
  const saved = useSettings((s) => s.settings?.pcsx2 ?? null);
  const hasCloud = useSettings((s) => !!s.settings?.cloud);
  const watching = useSync((s) => s.watching.pcsx2 ?? false);

  const [draft, setDraft] = useState<Pcsx2Settings>(saved ?? DEFAULTS);
  const [detection, setDetection] = useState<SourceInfo | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (saved) setDraft(saved);
  }, [saved]);

  // Pré-visualiza a detecção sempre que a pasta muda.
  useEffect(() => {
    let cancelled = false;
    const timer = setTimeout(() => {
      api
        .detectPcsx2(draft.dataDir)
        .then((info) => !cancelled && setDetection(info))
        .catch(() => {});
    }, 250);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [draft.dataDir]);

  const [candidates, setCandidates] = useState<SourceInfo[]>([]);
  const [scanning, setScanning] = useState(false);

  const findInstalls = async () => {
    setScanning(true);
    try {
      setCandidates(await api.pcsx2Candidates());
    } catch (e) {
      toast.error("Falha ao procurar instalações", errorMessage(e));
    } finally {
      setScanning(false);
    }
  };

  useEffect(() => {
    void findInstalls();
  }, []);

  const dirty = JSON.stringify(draft) !== JSON.stringify(saved ?? DEFAULTS);

  const browse = async () => {
    const dir = await open({ directory: true, title: "Pasta de dados do PCSX2 (contém inis e memcards)" });
    if (typeof dir === "string") setDraft((d) => ({ ...d, dataDir: dir }));
  };

  const save = async () => {
    setSaving(true);
    try {
      await useSettings.getState().updatePcsx2(draft);
      await Promise.all([useSync.getState().loadLibrary(), useSync.getState().loadWatchStatus()]);
      toast.success("Automação do PCSX2 salva");
    } catch (e) {
      toast.error("Não foi possível salvar", errorMessage(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Card>
      <CardHeader
        icon={<Gamepad2 className="size-4" />}
        title="PCSX2 · Memory cards"
        description="Sincroniza memory cards em arquivo (.ps2, o cartão inteiro) e em pasta (jogo por jogo)."
        actions={
          watching ? (
            <Badge tone="success">Monitorando</Badge>
          ) : draft.enabled && draft.autoSync && !hasCloud ? (
            <Badge tone="warning">Aguardando nuvem</Badge>
          ) : (
            <Badge>{draft.enabled ? "Manual" : "Desativado"}</Badge>
          )
        }
      />

      <div className="flex flex-col gap-5 p-5">
        <Switch
          label="Ativar automação"
          description="Lista os memory cards do PCSX2 na biblioteca e permite sincronizá-los."
          checked={draft.enabled}
          onChange={(enabled) => setDraft((d) => ({ ...d, enabled }))}
        />
        <Switch
          label="Sincronizar automaticamente"
          description="Envia alguns segundos depois que o PCSX2 grava um save, sincroniza ao abrir o app e baixa novidades quando o emulador é fechado."
          checked={draft.autoSync}
          disabled={!draft.enabled}
          onChange={(autoSync) => setDraft((d) => ({ ...d, autoSync }))}
        />

        <div className="h-px bg-line" />

        <div className="flex items-end gap-2">
          <TextField
            className="flex-1"
            label="Pasta de dados do PCSX2"
            placeholder="Detectar automaticamente (Documentos\PCSX2)"
            hint="Portátil: pode apontar para a pasta do PCSX2, para a pasta memcards ou para o pcsx2-qt.exe."
            mono
            value={draft.dataDir ?? ""}
            onValueChange={(v) => setDraft((d) => ({ ...d, dataDir: v || null }))}
          />
          <div className="mb-[22px] flex gap-2">
            <Button icon={<FolderOpen className="size-4" />} onClick={browse}>
              Procurar
            </Button>
            {draft.dataDir && (
              <Button
                variant="ghost"
                icon={<RotateCcw className="size-4" />}
                onClick={() => setDraft((d) => ({ ...d, dataDir: null }))}
                title="Voltar para a detecção automática"
              />
            )}
          </div>
        </div>

        <InstallCandidates
          candidates={candidates}
          current={detection?.dataDir ?? null}
          scanning={scanning}
          onRefresh={findInstalls}
          onUse={(dir) => setDraft((d) => ({ ...d, dataDir: dir }))}
        />

        {detection && <DetectionPanel info={detection} />}

        <div className="flex justify-end">
          <Button variant="primary" icon={<Save className="size-4" />} loading={saving} disabled={!dirty} onClick={save}>
            Salvar
          </Button>
        </div>
      </div>
    </Card>
  );
}

const samePath = (a: string | null, b: string | null) => !!a && !!b && a.toLowerCase() === b.toLowerCase();

function InstallCandidates({
  candidates,
  current,
  scanning,
  onRefresh,
  onUse,
}: {
  candidates: SourceInfo[];
  current: string | null;
  scanning: boolean;
  onRefresh: () => void;
  onUse: (dataDir: string) => void;
}) {
  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <p className="text-xs font-medium text-muted">Instalações encontradas</p>
        <Button size="sm" variant="ghost" icon={<Search className="size-3.5" />} loading={scanning} onClick={onRefresh}>
          Procurar novamente
        </Button>
      </div>
      {candidates.length === 0 ? (
        <p className="text-xs text-subtle">
          Nenhuma encontrada. Para detectar uma instalação portátil, abra o PCSX2 e clique em "Procurar novamente", ou
          escolha a pasta manualmente.
        </p>
      ) : (
        candidates.map((c) => {
          const active = samePath(c.dataDir, current);
          return (
            <div
              key={c.dataDir}
              className={cn(
                "flex items-center gap-3 rounded-lg border px-3 py-2.5",
                active ? "border-accent/40 bg-accent/5" : "border-line bg-surface-2",
              )}
            >
              <HardDrive className="size-4 shrink-0 text-subtle" />
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <p className="truncate font-mono text-xs text-fg">{c.dataDir}</p>
                  <Badge tone={c.portable ? "accent" : "neutral"}>{c.portable ? "Portátil" : "Documentos"}</Badge>
                </div>
                <p className="mt-0.5 truncate text-[11px] text-subtle">
                  {c.cards.length > 0 ? c.cards.join(" · ") : "Nenhum memory card"}
                </p>
              </div>
              {active ? (
                <Badge tone="success">Em uso</Badge>
              ) : (
                <Button size="sm" onClick={() => c.dataDir && onUse(c.dataDir)}>
                  Usar esta
                </Button>
              )}
            </div>
          );
        })
      )}
    </div>
  );
}

function DetectionPanel({ info }: { info: SourceInfo }) {
  return (
    <div className="rounded-lg border border-line bg-surface-2 p-4">
      <div className="mb-3 flex items-center gap-2">
        {info.detected ? (
          <CheckCircle2 className="size-4 text-success" />
        ) : (
          <TriangleAlert className="size-4 text-warning" />
        )}
        <p className="text-[13px] font-medium text-fg">{info.detected ? "PCSX2 encontrado" : "PCSX2 não encontrado"}</p>
      </div>
      <dl className="grid grid-cols-[140px_1fr] gap-x-4 gap-y-1.5 text-xs">
        <dt className="text-subtle">Pasta de dados</dt>
        <dd className="truncate font-mono text-muted">{info.dataDir ?? "—"}</dd>
        <dt className="text-subtle">Memory cards</dt>
        <dd className="truncate font-mono text-muted">{info.savesDir ?? "—"}</dd>
        <dt className="text-subtle">PCSX2.ini</dt>
        <dd className="text-muted">
          {info.configFound ? "Encontrado" : "Não encontrado"}
          {info.portable && " · instalação portátil"}
        </dd>
        <dt className="text-subtle">Cartões</dt>
        <dd className="flex flex-wrap gap-1.5">
          {info.cards.length === 0 ? (
            <span className="text-muted">Nenhum</span>
          ) : (
            info.cards.map((card) => (
              <span key={card} className="rounded bg-tint/5 px-1.5 py-0.5 font-mono text-[11px] text-fg">
                {card}
              </span>
            ))
          )}
        </dd>
      </dl>
      {info.message && <p className="mt-3 text-xs text-warning">{info.message}</p>}
    </div>
  );
}
