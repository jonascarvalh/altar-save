import type { ReactNode } from "react";
import { NavLink, useNavigate } from "react-router";
import { Activity, Cloud, CloudOff, Gamepad2, LibraryBig, Workflow } from "lucide-react";
import { useSettings } from "@/stores/settingsStore";
import { useSync } from "@/stores/syncStore";
import { cn } from "@/lib/format";
import { PROVIDERS } from "@/features/cloud/providers";

const NAV = [
  { to: "/", label: "Biblioteca", icon: <LibraryBig className="size-4" />, end: true },
  { to: "/automations", label: "Automações", icon: <Workflow className="size-4" /> },
  { to: "/cloud", label: "Nuvem", icon: <Cloud className="size-4" /> },
  { to: "/activity", label: "Atividade", icon: <Activity className="size-4" /> },
];

export function Sidebar() {
  const library = useSync((s) => s.library);
  const running = useSync((s) => s.running);

  return (
    <aside className="flex w-60 shrink-0 flex-col border-r border-line bg-surface">
      <nav className="flex flex-col gap-0.5 p-3">
        {NAV.map((item) => (
          <NavItem key={item.to} {...item} />
        ))}
      </nav>

      <div className="px-5 pt-4 pb-2 text-[10px] font-semibold tracking-[0.12em] text-subtle uppercase">Emuladores</div>
      <div className="flex flex-col gap-0.5 px-3">
        {library.map((source) => {
          const conflicts = source.units.filter((u) => u.status === "conflict").length;
          return (
            <NavLink
              key={source.info.id}
              to="/"
              className="group flex items-center gap-3 rounded-lg px-2 py-2 hover:bg-white/4"
            >
              <div className="grid size-8 place-items-center rounded-md bg-gradient-to-br from-[#1f6feb]/30 to-accent-2/30 text-fg">
                <Gamepad2 className="size-4" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="truncate text-[13px] font-medium text-fg">{source.info.name}</p>
                <p className="truncate text-[11px] text-subtle">
                  {!source.enabled
                    ? "Desativado"
                    : running[source.info.id]
                      ? "Sincronizando…"
                      : `${source.units.length} ${source.units.length === 1 ? "save" : "saves"}`}
                </p>
              </div>
              {conflicts > 0 && (
                <span className="grid size-5 place-items-center rounded-full bg-warning/15 text-[10px] font-bold text-warning">
                  {conflicts}
                </span>
              )}
              {running[source.info.id] && <span className="size-2 animate-pulse rounded-full bg-accent" />}
            </NavLink>
          );
        })}
      </div>

      <div className="mt-auto p-3">
        <CloudStatusCard />
      </div>
    </aside>
  );
}

function NavItem({ to, label, icon, end }: { to: string; label: string; icon: ReactNode; end?: boolean }) {
  return (
    <NavLink
      to={to}
      end={end}
      className={({ isActive }) =>
        cn(
          "relative flex h-9 items-center gap-3 rounded-lg px-3 text-[13px] font-medium transition-colors",
          isActive ? "bg-white/6 text-fg" : "text-muted hover:bg-white/4 hover:text-fg",
        )
      }
    >
      {({ isActive }) => (
        <>
          {isActive && <span className="absolute top-2 bottom-2 left-0 w-0.5 rounded-full bg-accent-gradient" />}
          <span className={isActive ? "text-accent" : undefined}>{icon}</span>
          {label}
        </>
      )}
    </NavLink>
  );
}

function CloudStatusCard() {
  const cloud = useSettings((s) => s.settings?.cloud);
  const navigate = useNavigate();

  return (
    <button
      type="button"
      onClick={() => navigate("/cloud")}
      className="flex w-full items-center gap-3 rounded-xl border border-line bg-surface-2 p-3 text-left transition-colors hover:border-line-strong"
    >
      <div
        className={cn(
          "grid size-9 place-items-center rounded-lg",
          cloud ? "bg-success/10 text-success" : "bg-warning/10 text-warning",
        )}
      >
        {cloud ? <Cloud className="size-4" /> : <CloudOff className="size-4" />}
      </div>
      <div className="min-w-0">
        <p className="truncate text-xs font-medium text-fg">
          {cloud ? PROVIDERS[cloud.provider].name : "Nuvem não configurada"}
        </p>
        <p className="truncate text-[11px] text-subtle">{cloud ? cloud.bucket : "Clique para configurar"}</p>
      </div>
    </button>
  );
}
