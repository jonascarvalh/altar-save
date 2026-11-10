# Configurando o PCSX2

Abra **Automações**. Ali você liga e desliga a automação do PCSX2 e o sync automático, define a pasta do emulador e vê o que foi detectado.

- [Pasta do PCSX2](#pasta-do-pcsx2)
- [O que o app lê do PCSX2.ini](#o-que-o-app-lê-do-pcsx2ini)
- [Memory card em arquivo ou em pasta?](#memory-card-em-arquivo-ou-em-pasta)
- [PCSX2 no Linux](#pcsx2-no-linux)

## Pasta do PCSX2

Por padrão, o app usa `Documentos\PCSX2`, a pasta da instalação padrão no Windows.

**Instalação portátil** (pasta com `portable.txt` ao lado do `pcsx2-qt.exe`):

- com o PCSX2 aberto, a instalação aparece em **Instalações encontradas**. Clique em **Usar esta**;
- ou informe a pasta manualmente. Funciona apontando para a pasta do PCSX2, para a pasta `memcards` ou para o próprio `pcsx2-qt.exe`.

## O que o app lê do PCSX2.ini

O app lê o `inis\PCSX2.ini` para descobrir:

| Chave | Uso |
|---|---|
| `[Folders] MemoryCards` | pasta dos memory cards (padrão `memcards`) |
| `[Folders] Covers` | pasta das capas (padrão `covers`), usada só para leitura na biblioteca |
| `[MemoryCards] SlotN_Filename` / `SlotN_Enable` | quais cartões estão nos slots (exibido como "Slot 1/2") |

## Memory card em arquivo ou em pasta?

| | Arquivo (`.ps2`) | Pasta (folder memory card) |
|---|---|---|
| Unidade de sync | o cartão inteiro (8 MB) | cada jogo (subpasta) |
| Conflitos | qualquer mudança em dois PCs conflita | só se o **mesmo jogo** mudar nos dois |
| Capa na biblioteca | não (o cartão mistura jogos) | sim |
| Envio | o cartão todo a cada mudança | só os arquivos do jogo que mudaram |

Para quem joga em mais de uma máquina, o cartão em pasta é bem melhor. Para converter um cartão no PCSX2: **Configurações → Memory Cards → botão direito no cartão → Converter**. O original é mantido.

> Um cartão em pasta só recebe jogos da nuvem se **existir localmente como pasta**. Crie ou converta o cartão antes na outra máquina.

## PCSX2 no Linux

O app procura por padrão em `~/Documents/PCSX2` (ou `~/Documentos/PCSX2`, conforme o idioma do sistema), que é o caminho do Windows. No Linux o PCSX2 guarda os dados em outro lugar, então **informe a pasta manualmente** em **Automações**:

| Instalação do PCSX2 | Pasta de dados |
|---|---|
| Pacote da distro / AppImage | `~/.config/PCSX2` |
| Flatpak | `~/.var/app/net.pcsx2.PCSX2/config/PCSX2` |
| Portátil (`portable.txt` ao lado do executável) | a pasta do executável (detectada automaticamente com o PCSX2 aberto) |

A detecção do emulador aberto funciona igual (procura `pcsx2` no nome do processo). Como o Linux não tem trava exclusiva de arquivo, saves em uso não ficam "Aguardando"; downloads e exclusões continuam esperando o PCSX2 fechar.
