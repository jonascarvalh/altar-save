# Configurando a nuvem

O Altar Save guarda os saves no **seu** bucket compatível com S3. Abra **Nuvem** no menu lateral, escolha o provedor, preencha os campos, clique em **Testar conexão** e depois em **Salvar**.

O teste de conexão verifica o acesso ao bucket e grava, lê e remove um arquivo de teste, mostrando o tempo de cada etapa.

- [Cloudflare R2 (recomendado)](#cloudflare-r2-recomendado)
- [Amazon S3](#amazon-s3)
- [S3 compatível (MinIO, Backblaze B2, Wasabi…)](#s3-compatível-minio-backblaze-b2-wasabi)
- [Várias máquinas no mesmo bucket](#várias-máquinas-no-mesmo-bucket)
- [Erros comuns no teste de conexão](#erros-comuns-no-teste-de-conexão)

## Cloudflare R2 (recomendado)

10 GB grátis e sem taxa de saída.

1. No painel da Cloudflare, abra **R2 Object Storage** e crie um bucket.
2. Copie o **Account ID** exibido na página do R2.
3. Em **Manage API tokens**, crie um token com permissão **Object Read & Write** para o bucket.
4. Copie o **Access Key ID** e o **Secret Access Key**. O secret só aparece uma vez.

| Campo | Exemplo |
|---|---|
| Account ID | `a1b2c3d4e5f6…` (32 caracteres) |
| Bucket | `altar-save` |
| Pasta no bucket | `PCSX2` (opcional) |
| Access Key ID / Secret | do token criado acima |

O endpoint `https://<account-id>.r2.cloudflarestorage.com` é montado automaticamente.

## Amazon S3

Informe **região** (ex.: `sa-east-1`), bucket e as chaves de um usuário IAM com `s3:GetObject`, `s3:PutObject`, `s3:DeleteObject` e `s3:ListBucket` no bucket.

## S3 compatível (MinIO, Backblaze B2, Wasabi…)

Informe o **endpoint** completo (ex.: `https://s3.us-west-004.backblazeb2.com`), a região (opcional, padrão `us-east-1`), o bucket e as chaves.

## Várias máquinas no mesmo bucket

Configure cada máquina com o **mesmo bucket e a mesma pasta no bucket**. É assim que elas enxergam os saves umas das outras. Veja [Como o sync funciona](sync.md) para entender conflitos e exclusões entre máquinas.

> As chaves ficam em texto puro no computador (veja [Dados locais](local-data.md)). Prefira um token restrito a esse bucket.

## Erros comuns no teste de conexão

| Mensagem | Causa provável |
|---|---|
| Acesso negado | Chaves erradas, token revogado ou sem permissão no bucket |
| Bucket não encontrado | Nome do bucket errado ou em outra conta |
| Não foi possível conectar ao endpoint | Account ID ou endpoint errado, ou sem internet |
| Região incorreta | (S3) A região não é a do bucket |
