import { useMemo } from "react";
import { useNavigate } from "react-router";
import { FolderSearch, Gamepad2, Loader2, Settings2 } from "lucide-react";
import { Page } from "@/components/ui/PageHeader";
import { Button } from "@/components/ui/Button";
import { EmptyState } from "@/components/ui/EmptyState";
import { useSync } from "@/stores/syncStore";
import type { SourceOverview, UnitView } from "@/lib/api/types";
import { SourceHero } from "./SourceHero";
import { UnitCard } from "./UnitCard";

export function LibraryPage() {
  const library = useSync((s) => s.library);
  const loaded = useSync((s) => s.libraryLoaded);

  if (!loaded) {
    return (
      <div className="grid h-full place-items-center text-muted">
        <Loader2 className="size-6 animate-spin" />
      </div>
    );
  }

  return (
    <Page>
      {library.map((source) => (
        <SourceSection key={source.info.id} source={source} />
      ))}
    </Page>
  );
}

function SourceSection({ source }: { source: SourceOverview }) {
  const navigate = useNavigate();
  const groups = useMemo(() => groupUnits(source.units), [source.units]);

  return (
    <section className="flex flex-col gap-6">
      <SourceHero source={source} />

      {!source.enabled ? (
        <EmptyState
          icon={<Settings2 className="size-5" />}
          title="Automação do PCSX2 desativada"
          description="Ative a automação para listar e sincronizar seus memory cards."
          action={<Button onClick={() => navigate("/automations")}>Abrir automações</Button>}
        />
      ) : !source.info.detected ? (
        <EmptyState
          icon={<FolderSearch className="size-5" />}
          title="PCSX2 não encontrado"
          description={source.info.message ?? "Informe a pasta de dados do PCSX2 nas automações."}
          action={<Button onClick={() => navigate("/automations")}>Configurar pasta</Button>}
        />
      ) : source.units.length === 0 ? (
        <EmptyState
          icon={<Gamepad2 className="size-5" />}
          title="Nenhum memory card encontrado"
          description={`Nenhum memory card em ${source.info.savesDir ?? "memcards"}. Jogue algo no PCSX2 ou sincronize para baixar da nuvem.`}
        />
      ) : (
        groups.map(([group, units]) => <UnitGroup key={group} name={group} units={units} />)
      )}
    </section>
  );
}

function UnitGroup({ name, units }: { name: string; units: UnitView[] }) {
  const isFolder = units.some((u) => u.kind === "directory");
  const tags = [...new Set(units.flatMap((u) => u.tags))];

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-3">
        <h2 className="text-sm font-semibold text-fg">{name}</h2>
        <span className="text-xs text-subtle">
          {isFolder ? `Memory card em pasta · ${units.length} ${units.length === 1 ? "jogo" : "jogos"}` : "Memory card em arquivo"}
        </span>
        {tags.map((t) => (
          <span key={t} className="rounded bg-accent/10 px-1.5 py-0.5 text-[10px] font-semibold text-accent">
            {t}
          </span>
        ))}
        <div className="h-px flex-1 bg-line" />
      </div>
      <div className="grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-4">
        {units.map((unit) => (
          <UnitCard key={unit.id} unit={unit} />
        ))}
      </div>
    </div>
  );
}

function groupUnits(units: UnitView[]): [string, UnitView[]][] {
  const map = new Map<string, UnitView[]>();
  for (const unit of units) {
    const list = map.get(unit.group) ?? [];
    list.push(unit);
    map.set(unit.group, list);
  }
  return [...map.entries()];
}
