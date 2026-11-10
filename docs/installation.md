# Instalação e build

Como preparar o ambiente, rodar em desenvolvimento e gerar o instalador no Windows e no Linux.

- [Windows](#windows)
- [Linux](#linux)
- [Problemas comuns](#problemas-comuns)

## Windows

### Pré-requisitos

| Ferramenta | Como instalar |
|---|---|
| Node.js 20+ | <https://nodejs.org> |
| Rust (stable, MSVC) | `winget install Rustlang.Rustup` |
| Visual Studio Build Tools | carga **"Desktop development with C++"** |
| WebView2 | já vem no Windows 10/11 |

### Rodando em desenvolvimento

```bash
npm install
npm run tauri dev
```

### Gerando o instalador

```bash
npm run tauri build
```

Saída em `src-tauri/target/release/`:

| Arquivo | Uso |
|---|---|
| `altar-save.exe` | executável, roda direto |
| `bundle/nsis/Altar Save_<versão>_x64-setup.exe` | instalador (~3,6 MB) |

## Linux

Testado no Manjaro (Arch) com Wayland. Deve funcionar em qualquer distro com WebKitGTK 4.1.

### 1. Dependências do sistema

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

### 2. Rust

Instale pelo `rustup` (no diretório do usuário, sem `sudo`):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"   # ou abra um terminal novo
cargo --version
```

### 3. Node.js 20+

Pelo gerenciador da distro (`sudo pacman -S nodejs npm`, `sudo apt install nodejs npm`…) ou pelo [nvm](https://github.com/nvm-sh/nvm). Confira com `node -v`.

### 4. Rodando em desenvolvimento

```bash
npm install
npm run tauri dev
```

A primeira execução compila todas as dependências Rust e leva alguns minutos. As seguintes são incrementais.

### 5. Gerando os pacotes

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

Depois de instalar, veja [PCSX2 no Linux](pcsx2.md#pcsx2-no-linux): a pasta do emulador precisa ser informada manualmente.

## Problemas comuns

| Sintoma | Solução |
|---|---|
| `No space left on device` durante o build | A pasta `src-tauri/target` ocupa uns 2 GB no desenvolvimento e mais alguns GB com o build de release. O cargo não apaga builds antigos sozinho: rode `cargo clean` em `src-tauri` de vez em quando. Ou libere espaço ou aponte o build para outro disco: `export CARGO_TARGET_DIR=/caminho/com/espaco/target` |
| `cargo: command not found` (Linux) | Rode `source "$HOME/.cargo/env"` ou abra um terminal novo |
| Erro de `pkg-config` / `webkit2gtk-4.1` não encontrado (Linux) | Falta a dependência do [passo 1](#1-dependências-do-sistema) (no Debian/Ubuntu, os pacotes `-dev`) |
| Janela branca ou em branco (comum com NVIDIA + Wayland) | `WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev` (vale também para o executável) |
