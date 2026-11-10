<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/logo-dark.svg" />
  <img src="docs/assets/logo-light.svg" width="360" alt="Altar Save" />
</picture>

**Seus saves de emulador sincronizados na nuvem, no seu próprio bucket.**

App desktop leve para Windows e Linux que mantém os saves dos seus emuladores sincronizados com um bucket compatível com S3: Cloudflare R2, Amazon S3, MinIO, Backblaze B2, Wasabi…

`Tauri 2` · `Rust` · `React 19` · `TypeScript` · `Tailwind CSS 4`

</div>

---

> **Status: protótipo.** Funciona localmente, sem contas nem servidor próprio. As credenciais da nuvem ficam em texto puro no computador (veja [Limitações](#limitações-conhecidas)).

## Funcionalidades

- **Seu próprio armazenamento**: Cloudflare R2 (10 GB grátis), Amazon S3 ou qualquer serviço compatível com S3, com teste de conexão.
- **PCSX2**: memory cards em arquivo (`.ps2`) e em pasta (sincronizados jogo por jogo), instalações padrão e portáteis.
- **Sync automático e em 3 vias**: envia segundos depois que o emulador grava, detecta conflitos e espera o emulador fechar para baixar.
- **Seguro**: backup antes de qualquer sobrescrita, uploads atômicos e exclusões que se propagam para as outras máquinas.
- **Biblioteca com capas** dos jogos, histórico de atividade e tema claro ou escuro.

## Como funciona

```
┌──────────────┐   watcher / manual   ┌──────────────────┐     S3 API     ┌──────────────────┐
│  PCSX2       │ ───────────────────▶ │   Altar Save     │ ─────────────▶ │  Seu bucket      │
│  memcards/   │ ◀─────────────────── │  (sync 3 vias)   │ ◀───────────── │  (R2 / S3 / …)   │
└──────────────┘   download + backup  └──────────────────┘                └──────────────────┘
                                              ▲
                          outra máquina ──────┘  (mesmo bucket, mesmo fluxo)
```

Cada save vira uma unidade com um `manifest.json` na nuvem. O app compara a versão local, a da nuvem e a última sincronizada nesta máquina para saber qual lado mudou. Detalhes em [Como o sync funciona](docs/sincronizacao.md).

## Começando

Pré-requisitos: **Node.js 20+**, **Rust** e as dependências do Tauri para o seu sistema (Visual Studio Build Tools no Windows, WebKitGTK no Linux). O passo a passo está em [Instalação e build](docs/instalacao.md).

```bash
npm install
npm run tauri dev     # desenvolvimento
npm run tauri build   # instalador (.exe no Windows; .deb, .rpm e .AppImage no Linux)
```

Com o app aberto:

1. **Nuvem**: escolha o provedor, preencha as chaves e teste a conexão ([guia](docs/nuvem.md)).
2. **Automações**: confira a pasta do PCSX2. No Linux, informe a pasta manualmente ([guia](docs/pcsx2.md)).
3. **Biblioteca**: clique em **Sincronizar agora**. Daí em diante, o sync é automático.

## Documentação

| Guia | Conteúdo |
|---|---|
| [Instalação e build](docs/instalacao.md) | pré-requisitos, desenvolvimento e pacotes no Windows e no Linux, problemas comuns |
| [Configurando a nuvem](docs/nuvem.md) | Cloudflare R2, Amazon S3, S3 compatível, erros do teste de conexão |
| [Configurando o PCSX2](docs/pcsx2.md) | pasta do emulador, instalações portáteis, cartão em arquivo × em pasta, Linux |
| [Usando o app](docs/uso.md) | biblioteca, status, conflitos, exclusão, atividade |
| [Como o sync funciona](docs/sincronizacao.md) | layout no bucket, upload e download atômicos, decisão em 3 vias, lápides, gatilhos |
| [Dados locais, backups e reset](docs/dados-locais.md) | onde ficam configurações, estado e backups; como zerar tudo |
| [Desenvolvimento](docs/desenvolvimento.md) | arquitetura, como adicionar um emulador, tema e logo, testes |

## Limitações conhecidas

- **Credenciais em texto puro** em `config.json`. O próximo passo é o Windows Credential Manager.
- **No Linux**, a pasta do PCSX2 precisa ser informada manualmente e não há detecção de arquivo em uso ([detalhes](docs/pcsx2.md#pcsx2-no-linux)).
- Um cartão em pasta só recebe jogos da nuvem se **existir localmente como pasta**.
- Sem contas, sem servidor próprio, sem criptografia dos saves no bucket.

## Roadmap

- [ ] Credenciais no Windows Credential Manager
- [ ] "Parar de sincronizar este save" (lista de ignorados)
- [ ] Ícone na bandeja e iniciar com o sistema
- [ ] Outros emuladores: DuckStation, RPCS3, Dolphin…
- [ ] Histórico de versões na nuvem (restaurar um save antigo)
- [ ] Criptografia opcional dos saves
