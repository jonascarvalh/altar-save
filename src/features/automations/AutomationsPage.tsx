import { Disc3, Gamepad, MonitorPlay } from "lucide-react";
import type { ReactNode } from "react";
import { Page, PageHeader } from "@/components/ui/PageHeader";
import { Card } from "@/components/ui/Card";
import { Badge } from "@/components/ui/Badge";
import { Pcsx2AutomationCard } from "./Pcsx2AutomationCard";

const UPCOMING: { name: string; description: string; icon: ReactNode }[] = [
  { name: "DuckStation", description: "Memory cards de PS1", icon: <Disc3 className="size-5" /> },
  { name: "RPCS3", description: "Saves de PS3", icon: <MonitorPlay className="size-5" /> },
  { name: "Dolphin", description: "GameCube e Wii", icon: <Gamepad className="size-5" /> },
];

export function AutomationsPage() {
  return (
    <Page>
      <PageHeader
        title="Automações"
        description="Cada automação sabe onde um emulador guarda os saves e os mantém sincronizados com a nuvem."
      />

      <Pcsx2AutomationCard />

      <div>
        <h2 className="mb-3 text-[11px] font-semibold tracking-[0.12em] text-subtle uppercase">Em breve</h2>
        <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
          {UPCOMING.map((item) => (
            <Card key={item.name} className="flex items-center gap-3 p-4 opacity-60">
              <div className="grid size-10 place-items-center rounded-lg bg-surface-3 text-muted">{item.icon}</div>
              <div className="min-w-0 flex-1">
                <p className="text-[13px] font-semibold text-fg">{item.name}</p>
                <p className="text-xs text-muted">{item.description}</p>
              </div>
              <Badge>Em breve</Badge>
            </Card>
          ))}
        </div>
      </div>
    </Page>
  );
}
