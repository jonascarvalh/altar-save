import type { CloudConfig, ProviderKind } from "@/lib/api/types";

export type CloudField = keyof Omit<CloudConfig, "provider">;

export interface FieldSpec {
  key: CloudField;
  label: string;
  placeholder?: string;
  hint?: string;
  required?: boolean;
  secret?: boolean;
  mono?: boolean;
}

export interface ProviderSpec {
  kind: ProviderKind;
  name: string;
  tagline: string;
  badge?: string;
  fields: FieldSpec[];
  /** Endpoint exibido como pré-visualização. */
  endpoint: (config: CloudConfig) => string;
}

const credentials: FieldSpec[] = [
  { key: "accessKeyId", label: "Access Key ID", required: true, mono: true },
  { key: "secretAccessKey", label: "Secret Access Key", required: true, secret: true, mono: true },
];

const location: FieldSpec[] = [
  { key: "bucket", label: "Bucket", placeholder: "meus-saves", required: true },
  {
    key: "prefix",
    label: "Pasta no bucket",
    placeholder: "save-nuvem",
    hint: "Opcional. Permite usar o mesmo bucket para outras coisas.",
  },
];

export const PROVIDERS: Record<ProviderKind, ProviderSpec> = {
  r2: {
    kind: "r2",
    name: "Cloudflare R2",
    tagline: "10 GB grátis, sem taxa de saída",
    badge: "Recomendado",
    fields: [
      {
        key: "accountId",
        label: "Account ID",
        placeholder: "a1b2c3d4e5f6…",
        hint: "Painel da Cloudflare → R2 → Account ID (lado direito).",
        required: true,
        mono: true,
      },
      ...location,
      ...credentials,
    ],
    endpoint: (c) => `https://${c.accountId.trim() || "<account-id>"}.r2.cloudflarestorage.com`,
  },
  s3: {
    kind: "s3",
    name: "Amazon S3",
    tagline: "Armazenamento da AWS",
    fields: [
      { key: "region", label: "Região", placeholder: "sa-east-1", hint: "Ex.: us-east-1, sa-east-1 (São Paulo).", required: true, mono: true },
      ...location,
      ...credentials,
    ],
    endpoint: (c) => `https://s3.${c.region.trim() || "<regiao>"}.amazonaws.com`,
  },
  custom: {
    kind: "custom",
    name: "S3 compatível",
    tagline: "MinIO, Backblaze B2, Wasabi…",
    fields: [
      { key: "endpoint", label: "Endpoint", placeholder: "https://s3.exemplo.com", required: true, mono: true },
      { key: "region", label: "Região", placeholder: "us-east-1", hint: "Opcional. Padrão: us-east-1.", mono: true },
      ...location,
      ...credentials,
    ],
    endpoint: (c) => c.endpoint.trim() || "<endpoint>",
  },
};

export const EMPTY_CONFIG: CloudConfig = {
  provider: "r2",
  accountId: "",
  region: "",
  endpoint: "",
  bucket: "",
  prefix: "save-nuvem",
  accessKeyId: "",
  secretAccessKey: "",
};

/** Campos obrigatórios que ainda faltam para o provedor escolhido. */
export function missingFields(config: CloudConfig): string[] {
  return PROVIDERS[config.provider].fields
    .filter((f) => f.required && !config[f.key].trim())
    .map((f) => f.label);
}
