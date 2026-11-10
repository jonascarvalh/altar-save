import { Check, Cloud, Database, Server } from "lucide-react";
import type { ReactNode } from "react";
import type { ProviderKind } from "@/lib/api/types";
import { cn } from "@/lib/format";
import { PROVIDERS } from "./providers";

const ICONS: Record<ProviderKind, ReactNode> = {
  r2: <Cloud className="size-5" />,
  s3: <Database className="size-5" />,
  custom: <Server className="size-5" />,
};

export function ProviderSelector({ value, onChange }: { value: ProviderKind; onChange: (kind: ProviderKind) => void }) {
  return (
    <div role="radiogroup" className="grid grid-cols-1 gap-3 md:grid-cols-3">
      {Object.values(PROVIDERS).map((p) => {
        const active = p.kind === value;
        return (
          <button
            key={p.kind}
            type="button"
            role="radio"
            aria-checked={active}
            onClick={() => onChange(p.kind)}
            className={cn(
              "group relative flex items-center gap-3 overflow-hidden rounded-xl border p-4 text-left transition-all",
              active
                ? "border-accent/50 bg-surface-2 shadow-[0_0_0_1px_rgb(61_214_245/0.25),0_12px_32px_-16px_rgb(61_214_245/0.5)]"
                : "border-line bg-surface hover:border-line-strong hover:bg-surface-2",
            )}
          >
            {active && <div className="pointer-events-none absolute -top-10 -right-10 size-28 rounded-full bg-accent/10 blur-2xl" />}
            <div
              className={cn(
                "grid size-10 shrink-0 place-items-center rounded-lg transition-colors",
                active ? "bg-accent-gradient text-[#06121a]" : "bg-surface-3 text-muted group-hover:text-fg",
              )}
            >
              {ICONS[p.kind]}
            </div>
            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2">
                <p className="text-[13px] font-semibold text-fg">{p.name}</p>
                {p.badge && (
                  <span className="rounded bg-accent/12 px-1.5 py-0.5 text-[10px] font-semibold text-accent">{p.badge}</span>
                )}
              </div>
              <p className="mt-0.5 truncate text-xs text-muted">{p.tagline}</p>
            </div>
            {active && <Check className="size-4 shrink-0 text-accent" />}
          </button>
        );
      })}
    </div>
  );
}
