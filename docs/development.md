# Desenvolvimento

Para preparar o ambiente, veja [Instalação e build](installation.md).

- [Arquitetura](#arquitetura)
- [Adicionando outro emulador](#adicionando-outro-emulador)
- [Tema e identidade visual](#tema-e-identidade-visual)
- [Testes](#testes)

Para publicar uma versão, veja [Publicando uma release](release.md).

## Arquitetura

Arquitetura hexagonal (ports & adapters). O domínio não conhece Tauri, AWS nem sistema de arquivos.

```
src-tauri/src/
├─ domain/                 regras puras e contratos
│  ├─ models.rs            CloudConfig, SyncUnit, RemoteManifest, estados, eventos
│  ├─ ports.rs             traits: CloudStorage, SaveSource, LocalUnitStore, BackupStore…
│  ├─ sync_policy.rs       decisão em 3 vias (função pura)
│  └─ hashing.rs           SHA-256 e hash de conteúdo da unidade
├─ application/            casos de uso
│  ├─ sync_service.rs      sync, conflitos, exclusão, backups, eventos
│  ├─ library.rs           visão da biblioteca (sem rede)
│  ├─ test_connection.rs   teste de conexão em etapas
│  └─ settings_service.rs  configurações
├─ infrastructure/         implementações das portas
│  ├─ storage/             S3/R2 (aws-sdk-s3) e um storage em memória para testes
│  ├─ sources/             PCSX2 (layout, scan, capas, icon.sys)
│  ├─ persistence.rs       JSON com escrita atômica
│  ├─ fs_store.rs          leitura e substituição atômica de arquivos
│  ├─ backups.rs           backups com rotação
│  ├─ watcher.rs           notify + debounce
│  ├─ process.rs           detecção do emulador (sysinfo)
│  └─ events.rs            eventos para a interface
├─ commands.rs             comandos Tauri finos (só delegam)
├─ state.rs                composição e injeção de dependências
└─ runtime.rs              automações em segundo plano

src/
├─ app/                    router e layout (titlebar, sidebar, statusbar)
├─ components/ui/          design system (Button, Card, TextField, ConfirmDialog…)
├─ features/               library · cloud · automations · activity
├─ lib/api/                wrappers tipados de invoke() e eventos
├─ stores/                 estado global (zustand), inclusive o tema
└─ styles/                 tokens do tema (Tailwind 4)
```

## Adicionando outro emulador

1. Crie `src-tauri/src/infrastructure/sources/<emulador>.rs` implementando `SaveSource`: `scan`, `info`, `watch_paths`, `process_names` e, opcionalmente, `find_covers`.
2. Registre-o em `DefaultSourceProvider` (`sources/mod.rs`) e adicione as configurações em `AppSettings`.

O motor de sync, o storage, os backups e a interface da biblioteca funcionam sem alteração.

## Tema e identidade visual

- **Cores**: tokens em `src/styles/globals.css`. O tema escuro é o padrão (`@theme`); o claro redefine os mesmos tokens em `:root[data-theme="light"]`. Use os tokens (`bg-surface`, `text-muted`, `text-accent`…) em vez de cores fixas. Para realces translúcidos, use `tint` (`bg-tint/5`, `border-tint/8`), que é branco no escuro e preto no claro.
- **Troca de tema**: `src/stores/themeStore.ts`. Um script em `index.html` aplica o tema salvo antes do primeiro paint, para não piscar.
- **Logo**: a arte vem do figcraft (projeto "Logo Altar Save").
  - `public/logo.svg`: símbolo usado na barra de título e no favicon;
  - `docs/assets/logo-dark.svg` e `logo-light.svg`: logo horizontal do README;
  - `src-tauri/app-icon.png`: ícone de 1024 px de onde saem os ícones do app. Para regenerá-los: `npx tauri icon src-tauri/app-icon.png` (apague depois as pastas `android/` e `ios/` geradas, o app é só desktop).

## Testes

Backend:

```bash
cd src-tauri
cargo test
```

São 46 testes no Windows e 45 no Linux (o de arquivo travado com exclusividade só existe no Windows), incluindo cenários com **duas máquinas compartilhando o mesmo bucket** (em memória):

- envio e download entre máquinas, conflito e as duas resoluções;
- cartões em pasta sincronizando jogo a jogo sem conflito;
- download adiado com o emulador aberto;
- arquivos travados com exclusividade (como o PCSX2 faz) ficando "Aguardando";
- exclusão com lápide: propagação, conflito com alteração local e espera pelo emulador;
- layout portátil, leitura do `PCSX2.ini`, serial → capa, `icon.sys` em Shift-JIS e proteção contra path traversal.

Frontend:

```bash
npx tsc --noEmit
```

No Linux, se `cargo` não for encontrado, rode `source "$HOME/.cargo/env"` antes.
