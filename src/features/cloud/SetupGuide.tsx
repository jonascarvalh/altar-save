import { BookOpen } from "lucide-react";
import type { ProviderKind } from "@/lib/api/types";
import { Card, CardHeader } from "@/components/ui/Card";

const STEPS: Record<ProviderKind, string[]> = {
  r2: [
    "No painel da Cloudflare, abra R2 Object Storage e crie um bucket.",
    "Copie o Account ID exibido na página do R2.",
    "Em \"Manage API tokens\", crie um token com permissão Object Read & Write para o bucket.",
    "Copie o Access Key ID e o Secret Access Key. O secret só aparece uma vez.",
  ],
  s3: [
    "No console da AWS, crie um bucket no S3 e anote a região.",
    "No IAM, crie um usuário com permissão s3:GetObject, s3:PutObject, s3:DeleteObject e s3:ListBucket no bucket.",
    "Gere uma Access Key para esse usuário.",
  ],
  custom: [
    "Use o endpoint S3 do seu provedor (ex.: https://s3.us-west-004.backblazeb2.com).",
    "Crie um bucket e uma chave de aplicação com leitura e escrita.",
    "Informe a região, se o provedor exigir.",
  ],
};

export function SetupGuide({ provider }: { provider: ProviderKind }) {
  return (
    <Card className="h-fit">
      <CardHeader icon={<BookOpen className="size-4" />} title="Como obter as chaves" />
      <ol className="flex flex-col gap-3 p-5">
        {STEPS[provider].map((step, i) => (
          <li key={step} className="flex gap-3 text-xs leading-relaxed text-muted">
            <span className="grid size-5 shrink-0 place-items-center rounded-full bg-surface-3 text-[10px] font-semibold text-fg">
              {i + 1}
            </span>
            <span>{step}</span>
          </li>
        ))}
      </ol>
      <p className="border-t border-line px-5 py-3 text-[11px] text-subtle">
        Protótipo: as chaves ficam em texto puro em %APPDATA%\com.savenuvem.app\config.json.
      </p>
    </Card>
  );
}
