<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/logo-dark.svg" />
  <img src="docs/assets/logo-light.svg" width="360" alt="Altar Save" />
</picture>

**Seus saves de emulador sincronizados na nuvem, no seu próprio bucket.**

App desktop leve para Windows (e Linux) que mantém os saves dos seus emuladores sincronizados com um bucket compatível com S3: Cloudflare R2, Amazon S3, MinIO, Backblaze B2, Wasabi…

`Tauri 2` · `Rust` · `React 19` · `TypeScript` · `Tailwind CSS 4`

</div>

---

> **Status: protótipo.** Funciona localmente, sem contas nem servidor próprio. As credenciais da nuvem ficam em texto puro no computador (veja [Limitações](#limitações-conhecidas)).

## Sumário

- [Funcionalidades](#funcionalidades)
- [Como funciona (visão rápida)](#como-funciona-visão-rápida)
- [Instalação e execução](#instalação-e-execução)
- [Configurando a nuvem](#configurando-a-nuvem)
- [Configurando o PCSX2](#configurando-o-pcsx2)
- [Usando o app](#usando-o-app)
- [Como o sync funciona (detalhes)](#como-o-sync-funciona-detalhes)
- [Dados locais, backups e reset](#dados-locais-backups-e-reset)
- [Arquitetura](#arquitetura)
- [Testes](#testes)
- [Limitações conhecidas](#limitações-conhecidas)
- [Roadmap](#roadmap)

## Funcionalidades

- **Seu próprio armazenamento.** Funciona com Cloudflare R2 (10 GB grátis, sem taxa de saída), Amazon S3 ou qualquer serviço compatível com S3.
- **Teste de conexão** na tela de configuração: acesso ao bucket, gravação, leitura e remoção de um arquivo de teste, com o tempo de cada etapa.
- **Automação PCSX2**:
  - memory cards **em arquivo** (`.ps2`), sincronizados como cartão inteiro;
  - memory cards **em pasta** (folder memory cards), sincronizados **jogo por jogo**;
  - detecção da instalação padrão (`Documentos\PCSX2`) e de **instalações portáteis**;
  - leitura do `PCSX2.ini` (pasta de memory cards, slots, pasta de capas).
- **Sync automático**: monitora a pasta e envia segundos depois que o emulador grava. Também sincroniza ao abrir o app e tem botão de sync manual.
- **Sync bidirecional em 3 vias**, com detecção de **conflitos** ("Manter local" / "Manter nuvem").
- **Seguro com o emulador aberto**: downloads e saves em uso esperam o PCSX2 fechar e acontecem sozinhos depois.
- **Backups automáticos** antes de qualquer sobrescrita ou exclusão local.
- **Exclusão de saves** que se propaga para as outras máquinas (com lápide e proteção contra perda de alterações).
- **Capas dos jogos** lidas da pasta de capas do PCSX2 (somente leitura).
- **Histórico de atividade** com filtros (envios, downloads, exclusões, conflitos, erros).
- Interface **voltada para gamers**, com titlebar própria e **tema escuro ou claro** (segue o sistema até você escolher um pelo ícone de sol/lua na barra de título). O instalador tem ~3,6 MB.

## Como funciona (visão rápida)

```
┌──────────────┐   watcher / manual   ┌──────────────────┐     S3 API     ┌──────────────────┐
│  PCSX2       │ ───────────────────▶ │   Altar Save     │ ─────────────▶ │  Seu bucket      │
│  memcards/   │ ◀─────────────────── │  (sync 3 vias)   │ ◀───────────── │  (R2 / S3 / …)   │
└──────────────┘   download + backup  └──────────────────┘                └──────────────────┘
                                              ▲
                          outra máquina ──────┘  (mesmo bucket, mesmo fluxo)
```

Cada save (um cartão `.ps2` ou a pasta de um jogo) vira uma **unidade de sync**. Na nuvem, a unidade tem um `manifest.json`, que serve de índice, e os arquivos ficam guardados pelo hash SHA-256 do conteúdo. O app compara três versões: a **local**, a **da nuvem** e a **última sincronizada nesta máquina**. Assim ele sabe qual lado mudou.

## Instalação e execução

### Windows

#### Pré-requisitos

| Ferramenta | Como instalar |
|---|---|
| Node.js 20+ | <https://nodejs.org> |
| Rust (stable, MSVC) | `winget install Rustlang.Rustup` |
| Visual Studio Build Tools | carga **"Desktop development with C++"** |
| WebView2 | já vem no Windows 10/11 |

#### Rodando em desenvolvimento

```bash
npm install
npm run tauri dev
```

#### Gerando o instalador

```bash
npm run tauri build
```

Saída:

- executável: `src-tauri/target/release/altar-save.exe`
- instalador: `src-tauri/target/release/bundle/nsis/Altar Save_<versão>_x64-setup.exe`

### Linux

Testado no Manjaro (Arch) com Wayland. Deve funcionar em qualquer distro com WebKitGTK 4.1.

#### 1. Dependências do sistema

O Tauri usa o WebKitGTK como WebView. Instale-o junto com as ferramentas de compilação:

**Arch / Manjaro**

```bash
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl librsvg
```

**Debian / Ubuntu (22.04+)**

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Fedora**

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
  libappindicator-gtk3-devel librsvg2-devel libxdo-devel
sudo dnf group install "c-development"
```

Para outras distros, veja a lista oficial: <https://v2.tauri.app/start/prerequisites/#linux>.

#### 2. Rust

Instale pelo `rustup` (no diretório do usuário, sem `sudo`):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"   # ou abra um terminal novo
cargo --version
```

#### 3. Node.js 20+

Pelo gerenciador da distro (`sudo pacman -S nodejs npm`, `sudo apt install nodejs npm`…) ou pelo [nvm](https://github.com/nvm-sh/nvm). Confira com `node -v`.

#### 4. Rodando em desenvolvimento

```bash
npm install
npm run tauri dev
```

A primeira execução compila todas as dependências Rust e leva alguns minutos. As seguintes são incrementais.

#### 5. Gerando os pacotes

```bash
npm run tauri build
```

No Linux, o `src-tauri/tauri.linux.conf.json` troca o alvo NSIS (Windows) por pacotes Linux. Saída em `src-tauri/target/release/`:

| Arquivo | Uso |
|---|---|
| `altar-save` | executável, roda direto |
| `bundle/deb/Altar Save_<versão>_amd64.deb` | Debian / Ubuntu: `sudo apt install "./Altar Save_<versão>_amd64.deb"` |
| `bundle/rpm/Altar Save-<versão>-1.x86_64.rpm` | Fedora / openSUSE: `sudo dnf install "./Altar Save-<versão>-1.x86_64.rpm"` |
| `bundle/appimage/Altar Save_<versão>_amd64.AppImage` | qualquer distro: `chmod +x` e execute |

Para gerar só um formato: `npm run tauri build -- --bundles deb` (ou `rpm`, `appimage`).

> O AppImage baixa o `linuxdeploy` do GitHub na primeira vez, então precisa de internet. Ele também é bem maior (~100 MB), porque embute o WebKitGTK.

#### Problemas comuns no Linux

| Sintoma | Solução |
|---|---|
| `No space left on device` durante o build | A pasta `src-tauri/target` passa de 4 GB entre debug e release. Libere espaço ou aponte o build para outro disco: `export CARGO_TARGET_DIR=/caminho/com/espaco/target` |
| `cargo: command not found` | Rode `source "$HOME/.cargo/env"` ou abra um terminal novo |
| Erro de `pkg-config` / `webkit2gtk-4.1` não encontrado | Falta a dependência do passo 1 (no Debian/Ubuntu, os pacotes `-dev`) |
| Janela branca ou em branco (comum com NVIDIA + Wayland) | `WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev` (vale também para o executável) |

## Configurando a nuvem

Abra **Nuvem** no menu lateral, escolha o provedor, preencha os campos, clique em **Testar conexão** e depois em **Salvar**.

### Cloudflare R2 (recomendado)

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

### Amazon S3

Informe **região** (ex.: `sa-east-1`), bucket e as chaves de um usuário IAM com `s3:GetObject`, `s3:PutObject`, `s3:DeleteObject` e `s3:ListBucket` no bucket.

### S3 compatível (MinIO, Backblaze B2, Wasabi…)

Informe o **endpoint** completo (ex.: `https://s3.us-west-004.backblazeb2.com`), a região (opcional, padrão `us-east-1`), o bucket e as chaves.

### Erros comuns no teste de conexão

| Mensagem | Causa provável |
|---|---|
| Acesso negado | Chaves erradas, token revogado ou sem permissão no bucket |
| Bucket não encontrado | Nome do bucket errado ou em outra conta |
| Não foi possível conectar ao endpoint | Account ID ou endpoint errado, ou sem internet |
| Região incorreta | (S3) A região não é a do bucket |

## Configurando o PCSX2

Abra **Automações**. Por padrão, o app usa `Documentos\PCSX2`.

**Instalação portátil** (pasta com `portable.txt` ao lado do `pcsx2-qt.exe`):

- com o PCSX2 aberto, a instalação aparece em **Instalações encontradas**. Clique em **Usar esta**;
- ou informe a pasta manualmente. Funciona apontando para a pasta do PCSX2, para a pasta `memcards` ou para o próprio `pcsx2-qt.exe`.

O app lê o `inis\PCSX2.ini` para descobrir:

| Chave | Uso |
|---|---|
| `[Folders] MemoryCards` | pasta dos memory cards (padrão `memcards`) |
| `[Folders] Covers` | pasta das capas (padrão `covers`) |
| `[MemoryCards] SlotN_Filename` / `SlotN_Enable` | quais cartões estão nos slots (exibido como "Slot 1/2") |

### Memory card em arquivo ou em pasta?

| | Arquivo (`.ps2`) | Pasta (folder memory card) |
|---|---|---|
| Unidade de sync | o cartão inteiro (8 MB) | cada jogo (subpasta) |
| Conflitos | qualquer mudança em dois PCs conflita | só se o **mesmo jogo** mudar nos dois |
| Capa na biblioteca | não (o cartão mistura jogos) | sim |
| Envio | o cartão todo a cada mudança | só os arquivos do jogo que mudaram |

Para converter um cartão no PCSX2: **Configurações → Memory Cards → botão direito no cartão → Converter**. O original é mantido.

### PCSX2 no Linux

O app procura por padrão em `~/Documents/PCSX2` (ou `~/Documentos/PCSX2`, conforme o idioma do sistema), que é o caminho do Windows. No Linux o PCSX2 guarda os dados em outro lugar, então **informe a pasta manualmente** em **Automações**:

| Instalação do PCSX2 | Pasta de dados |
|---|---|
| Pacote da distro / AppImage | `~/.config/PCSX2` |
| Flatpak | `~/.var/app/net.pcsx2.PCSX2/config/PCSX2` |
| Portátil (`portable.txt` ao lado do executável) | a pasta do executável (detectada automaticamente com o PCSX2 aberto) |

A detecção do emulador aberto funciona igual (procura `pcsx2` no nome do processo). Como o Linux não tem trava exclusiva de arquivo, saves em uso não ficam "Aguardando"; downloads e exclusões continuam esperando o PCSX2 fechar.

## Usando o app

- **Biblioteca**: os saves agrupados por memory card, com status (Em dia, Pendente, Conflito, Aguardando, Erro), tamanho, última alteração e a máquina que enviou por último.
  - **Sincronizar agora** força um sync completo.
  - **Conflito**: escolha **Manter local** (envia a versão deste PC) ou **Manter nuvem** (baixa; a local vai para o backup).
  - **Excluir**: passe o mouse no card e clique na lixeira. Há uma confirmação antes.
- **Automações**: liga e desliga o PCSX2 e o sync automático, define a pasta e mostra o que foi detectado.
- **Nuvem**: provedor, credenciais e teste de conexão.
- **Atividade**: histórico com filtros e o botão **Abrir backups**.
- **Barra inferior**: progresso do sync em andamento, resultado do último e se o monitoramento está ativo.

## Como o sync funciona (detalhes)

### Layout no bucket

```
<pasta no bucket>/
└─ pcsx2/
   ├─ memcards/
   │  └─ Mcd002/                        ← memory card em arquivo
   │     ├─ manifest.json
   │     └─ blobs/<sha256>
   └─ folders/
      └─ Mcd001_converted/              ← memory card em pasta
         └─ BASCUS-97399GodOfWar/       ← um jogo = uma unidade
            ├─ manifest.json
            └─ blobs/<sha256>           ← um por conteúdo distinto (data0.bin, icon.sys…)
```

O `manifest.json` lista cada arquivo (`caminho`, `sha256`, `tamanho`), o hash da unidade, a data e a máquina que enviou. Os **blobs** são endereçados por conteúdo: arquivos idênticos (ex.: slots vazios) viram um blob só.

### Upload atômico

1. Envia só os blobs que a nuvem ainda não tem.
2. Grava o `manifest.json` **por último**. Esse é o commit: se a conexão cair antes, a nuvem continua com a versão anterior completa.
3. Remove os blobs que nenhum manifest usa mais.

### Download seguro

1. Baixa os blobs e **confere o SHA-256** de cada um.
2. Copia a versão local atual para os **backups**.
3. Monta a versão nova numa pasta temporária e troca de lugar com renames. Não existe estado "meio baixado".

### Decisão em 3 vias

`base` = hash da última versão sincronizada **nesta máquina** (em `sync-state.json`).

| Local | Nuvem | Resultado |
|---|---|---|
| existe | não existe | **envia** |
| não existe | existe | **baixa** |
| igual à nuvem | — | em dia |
| mudou (nuvem = base) | — | **envia** |
| igual à base | mudou | **baixa** |
| mudou | mudou | **conflito** |
| igual à base | **excluído** (lápide) | **apaga localmente** (com backup) |
| mudou | **excluído** (lápide) | **conflito** ("Manter" ou "Excluir") |

> Apagar um save **só no PC** faz o app baixá-lo de novo: arquivo ausente é tratado como "precisa baixar", para nunca perder save por engano. Para apagar de verdade, use **Excluir** no app.

### Exclusão com lápide

Ao excluir, o app grava no lugar do `manifest.json` uma **lápide** (`"deleted": true`, com a data e a máquina), remove os blobs e apaga a cópia local, com backup. As outras máquinas veem a lápide no próximo sync:

- apagam a cópia delas, se não houve alteração desde o último sync;
- abrem conflito, se houve ("Manter" reenvia e desfaz a exclusão).

### Emulador aberto e arquivos em uso

- **Downloads e exclusões** esperam o PCSX2 fechar. O status fica **Aguardando**.
- Quando o jogo está rodando, o PCSX2 abre os arquivos do cartão com exclusividade. Esses saves também ficam **Aguardando** (não é erro).
- A cada 30 s o app verifica se o emulador fechou e retoma o que estava pendente.

### Gatilhos

| Gatilho | Quando |
|---|---|
| Watcher | ~5 s após uma gravação (~10 s com cartões em pasta, que o PCSX2 grava com atraso) |
| Inicialização | ao abrir o app |
| Nova tentativa | a cada 30 s, se houver algo aguardando e o emulador estiver fechado |
| Manual | botão **Sincronizar agora** |

O arquivo `_pcsx2_index` (cartões em pasta) é transferido, mas fica **fora do hash**. O PCSX2 o reescreve com frequência, e isso não deve disparar sync.

## Dados locais, backups e reset

Tudo fica em `%APPDATA%\com.savenuvem.app`:

| Arquivo | Conteúdo |
|---|---|
| `config.json` | nuvem (inclusive as chaves) e automações |
| `sync-state.json` | base de cada save, usada nas decisões de sync |
| `activity.json` | histórico (últimos 300 eventos) |
| `backups\` | cópias feitas antes de downloads/exclusões (10 por save, por padrão) |

O cache do WebView fica em `%LOCALAPPDATA%\com.savenuvem.app`.

**Reset completo** (com o app fechado):

```bash
Remove-Item -Recurse -Force "$env:APPDATA\com.savenuvem.app", "$env:LOCALAPPDATA\com.savenuvem.app"
```

No **Linux**, os mesmos arquivos ficam em `~/.local/share/com.savenuvem.app` (com `backups/`), junto com o cache do WebView. Reset completo:

```bash
rm -rf ~/.local/share/com.savenuvem.app ~/.cache/com.savenuvem.app
```

O bucket não é afetado. Para zerar a nuvem também, apague a pasta configurada no painel do provedor.

> Apagar só o `sync-state.json` faz o app perder a base: saves diferentes entre PC e nuvem viram conflito, e nada é sobrescrito sem você escolher.

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
├─ stores/                 estado global (zustand)
└─ styles/                 tokens do tema (Tailwind 4)
```

### Adicionando outro emulador

1. Crie `src-tauri/src/infrastructure/sources/<emulador>.rs` implementando `SaveSource`: `scan`, `info`, `watch_paths`, `process_names` e, opcionalmente, `find_covers`.
2. Registre-o em `DefaultSourceProvider` (`sources/mod.rs`) e adicione as configurações em `AppSettings`.

O motor de sync, o storage, os backups e a interface da biblioteca funcionam sem alteração.

## Testes

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

## Limitações conhecidas

- **Credenciais em texto puro** em `config.json`. O próximo passo é o Windows Credential Manager.
- **Feito para Windows; Linux compila e roda**, mas a pasta padrão do PCSX2 ainda é a do Windows (informe a pasta manualmente) e não há detecção de arquivo em uso (veja [PCSX2 no Linux](#pcsx2-no-linux)).
- Um cartão em pasta só recebe jogos da nuvem se **existir localmente como pasta** (crie ou converta o cartão antes).
- Sem contas, sem servidor próprio, sem criptografia dos saves no bucket.
- As lápides (manifests de poucos bytes) permanecem no bucket.

## Roadmap

- [ ] Credenciais no Windows Credential Manager
- [ ] "Parar de sincronizar este save" (lista de ignorados)
- [ ] Ícone na bandeja e iniciar com o Windows
- [ ] Outros emuladores: DuckStation, RPCS3, Dolphin…
- [ ] Histórico de versões na nuvem (restaurar um save antigo)
- [ ] Criptografia opcional dos saves
