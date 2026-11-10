import { CheckCircle2, Loader2, XCircle } from "lucide-react";
import type { ConnectionReport } from "@/lib/api/types";
import { Card } from "@/components/ui/Card";
import { cn } from "@/lib/format";

export function ConnectionResult({ report, testing }: { report: ConnectionReport | null; testing: boolean }) {
  if (testing || !report) {
    return (
      <Card className="flex animate-fade-in items-center gap-3 p-5">
        <Loader2 className="size-5 animate-spin text-accent" />
        <div>
          <p className="text-[13px] font-medium text-fg">Testando conexão…</p>
          <p className="text-xs text-muted">Acessando o bucket e gravando um arquivo de teste.</p>
        </div>
      </Card>
    );
  }

  return (
    <Card
      className={cn(
        "animate-slide-up overflow-hidden",
        report.ok ? "border-success/25" : "border-danger/25",
      )}
    >
      <div className={cn("flex items-center gap-3 px-5 py-4", report.ok ? "bg-success/6" : "bg-danger/6")}>
        {report.ok ? <CheckCircle2 className="size-5 text-success" /> : <XCircle className="size-5 text-danger" />}
        <div className="min-w-0">
          <p className="text-[13px] font-semibold text-fg">
            {report.ok ? "Tudo certo! A conexão está funcionando." : "A conexão falhou."}
          </p>
          <p className="truncate font-mono text-[11px] text-muted">
            {report.endpoint} / {report.bucket}
          </p>
        </div>
      </div>
      <ul className="divide-y divide-line">
        {report.steps.map((step) => (
          <li key={step.name} className="flex items-start gap-3 px-5 py-3">
            {step.ok ? (
              <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-success" />
            ) : (
              <XCircle className="mt-0.5 size-4 shrink-0 text-danger" />
            )}
            <div className="min-w-0 flex-1">
              <p className="text-[13px] text-fg">{step.name}</p>
              {step.error && <p className="mt-0.5 text-xs break-words text-danger/90">{step.error}</p>}
            </div>
            <span className="font-mono text-[11px] text-subtle">{step.durationMs} ms</span>
          </li>
        ))}
      </ul>
    </Card>
  );
}
