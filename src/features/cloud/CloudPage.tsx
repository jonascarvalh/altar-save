import { useEffect, useMemo, useState } from "react";
import { Globe, KeyRound, Plug, Save, Trash2 } from "lucide-react";
import { Page, PageHeader } from "@/components/ui/PageHeader";
import { Card, CardHeader } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/TextField";
import { Badge } from "@/components/ui/Badge";
import { toast } from "@/components/ui/Toaster";
import { api } from "@/lib/api/commands";
import { errorMessage } from "@/lib/api/errors";
import type { CloudConfig, ConnectionReport } from "@/lib/api/types";
import { useSettings } from "@/stores/settingsStore";
import { useSync } from "@/stores/syncStore";
import { EMPTY_CONFIG, PROVIDERS, missingFields, type CloudField } from "./providers";
import { ProviderSelector } from "./ProviderSelector";
import { ConnectionResult } from "./ConnectionResult";
import { SetupGuide } from "./SetupGuide";

export function CloudPage() {
  const saved = useSettings((s) => s.settings?.cloud ?? null);
  const { saveCloud, clearCloud } = useSettings.getState();

  const [draft, setDraft] = useState<CloudConfig>(saved ?? EMPTY_CONFIG);
  const [report, setReport] = useState<ConnectionReport | null>(null);
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);

  // Sincroniza o formulário quando as configurações chegam do backend.
  useEffect(() => {
    if (saved) setDraft(saved);
  }, [saved]);

  const provider = PROVIDERS[draft.provider];
  const missing = missingFields(draft);
  const dirty = useMemo(() => JSON.stringify(draft) !== JSON.stringify(saved ?? EMPTY_CONFIG), [draft, saved]);

  const update = (key: CloudField, value: string) => {
    setDraft((d) => ({ ...d, [key]: value }));
    setReport(null);
  };

  const test = async () => {
    setTesting(true);
    setReport(null);
    try {
      const result = await api.testConnection(draft);
      setReport(result);
      if (result.ok) toast.success("Conexão funcionando", `Bucket "${result.bucket}" acessível.`);
    } catch (e) {
      toast.error("Não foi possível testar", errorMessage(e));
    } finally {
      setTesting(false);
    }
  };

  const save = async () => {
    setSaving(true);
    try {
      await saveCloud(draft);
      await useSync.getState().loadWatchStatus();
      toast.success("Nuvem salva", "As automações já vão usar esta configuração.");
    } catch (e) {
      toast.error("Não foi possível salvar", errorMessage(e));
    } finally {
      setSaving(false);
    }
  };

  const remove = async () => {
    try {
      await clearCloud();
      await useSync.getState().loadWatchStatus();
      setDraft(EMPTY_CONFIG);
      setReport(null);
      toast.info("Configuração removida");
    } catch (e) {
      toast.error("Não foi possível remover", errorMessage(e));
    }
  };

  return (
    <Page>
      <PageHeader
        title="Nuvem"
        description="Conecte um bucket compatível com S3. Seus saves ficam no seu próprio armazenamento: nada passa por servidores de terceiros."
        actions={
          saved ? (
            <Badge tone="success">Configurada</Badge>
          ) : (
            <Badge tone="warning">Não configurada</Badge>
          )
        }
      />

      <ProviderSelector
        value={draft.provider}
        onChange={(kind) => {
          setDraft((d) => ({ ...d, provider: kind }));
          setReport(null);
        }}
      />

      <div className="grid grid-cols-1 gap-6 xl:grid-cols-[1fr_320px]">
        <div className="flex flex-col gap-6">
          <Card>
            <CardHeader
              icon={<KeyRound className="size-4" />}
              title={`Credenciais · ${provider.name}`}
              description="As chaves ficam salvas apenas neste computador."
              actions={dirty && <Badge tone="info">Alterações não salvas</Badge>}
            />
            <form
              className="grid grid-cols-1 gap-4 p-5 md:grid-cols-2"
              onSubmit={(e) => {
                e.preventDefault();
                void save();
              }}
            >
              {provider.fields.map((field) => (
                <TextField
                  key={`${draft.provider}-${field.key}`}
                  label={field.label}
                  placeholder={field.placeholder}
                  hint={field.hint}
                  required={field.required}
                  secret={field.secret}
                  mono={field.mono}
                  value={draft[field.key]}
                  onValueChange={(v) => update(field.key, v)}
                  className={field.key === "secretAccessKey" || field.key === "endpoint" ? "md:col-span-2" : undefined}
                />
              ))}

              <div className="flex items-center gap-2 rounded-lg border border-line bg-surface-2 px-3 py-2.5 md:col-span-2">
                <Globe className="size-4 shrink-0 text-subtle" />
                <span className="text-xs text-muted">Endpoint:</span>
                <code className="truncate font-mono text-xs text-fg">{provider.endpoint(draft)}</code>
              </div>

              <div className="flex flex-wrap items-center gap-2 pt-1 md:col-span-2">
                <Button
                  type="button"
                  icon={<Plug className="size-4" />}
                  loading={testing}
                  disabled={missing.length > 0}
                  onClick={test}
                >
                  Testar conexão
                </Button>
                <Button
                  type="submit"
                  variant="primary"
                  icon={<Save className="size-4" />}
                  loading={saving}
                  disabled={missing.length > 0 || !dirty}
                >
                  Salvar
                </Button>
                {missing.length > 0 && <span className="text-xs text-subtle">Falta: {missing.join(", ")}</span>}
                {saved && (
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="ml-auto text-danger hover:text-danger"
                    icon={<Trash2 className="size-3.5" />}
                    onClick={remove}
                  >
                    Remover
                  </Button>
                )}
              </div>
            </form>
          </Card>

          {(testing || report) && <ConnectionResult report={report} testing={testing} />}
        </div>

        <SetupGuide provider={draft.provider} />
      </div>
    </Page>
  );
}
