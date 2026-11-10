# Como o sync funciona

Cada save (um cartão `.ps2` ou a pasta de um jogo) vira uma **unidade de sync**. Na nuvem, a unidade tem um `manifest.json`, que serve de índice, e os arquivos ficam guardados pelo hash SHA-256 do conteúdo. O app compara três versões: a **local**, a **da nuvem** e a **última sincronizada nesta máquina**. Assim ele sabe qual lado mudou.

- [Layout no bucket](#layout-no-bucket)
- [Upload atômico](#upload-atômico)
- [Download seguro](#download-seguro)
- [Decisão em 3 vias](#decisão-em-3-vias)
- [Exclusão com lápide](#exclusão-com-lápide)
- [Emulador aberto e arquivos em uso](#emulador-aberto-e-arquivos-em-uso)
- [Gatilhos](#gatilhos)

## Layout no bucket

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

## Upload atômico

1. Envia só os blobs que a nuvem ainda não tem.
2. Grava o `manifest.json` **por último**. Esse é o commit: se a conexão cair antes, a nuvem continua com a versão anterior completa.
3. Remove os blobs que nenhum manifest usa mais.

## Download seguro

1. Baixa os blobs e **confere o SHA-256** de cada um.
2. Copia a versão local atual para os **backups**.
3. Monta a versão nova numa pasta temporária e troca de lugar com renames. Não existe estado "meio baixado".

## Decisão em 3 vias

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

## Exclusão com lápide

Ao excluir, o app grava no lugar do `manifest.json` uma **lápide** (`"deleted": true`, com a data e a máquina), remove os blobs e apaga a cópia local, com backup. As outras máquinas veem a lápide no próximo sync:

- apagam a cópia delas, se não houve alteração desde o último sync;
- abrem conflito, se houve ("Manter" reenvia e desfaz a exclusão).

As lápides (manifests de poucos bytes) permanecem no bucket.

## Emulador aberto e arquivos em uso

- **Downloads e exclusões** esperam o PCSX2 fechar. O status fica **Aguardando**.
- Quando o jogo está rodando, o PCSX2 abre os arquivos do cartão com exclusividade (no Windows). Esses saves também ficam **Aguardando** (não é erro).
- A cada 30 s o app verifica se o emulador fechou e retoma o que estava pendente.

## Gatilhos

| Gatilho | Quando |
|---|---|
| Watcher | ~5 s após uma gravação (~10 s com cartões em pasta, que o PCSX2 grava com atraso) |
| Inicialização | ao abrir o app |
| Nova tentativa | a cada 30 s, se houver algo aguardando e o emulador estiver fechado |
| Manual | botão **Sincronizar agora** |

O arquivo `_pcsx2_index` (cartões em pasta) é transferido, mas fica **fora do hash**. O PCSX2 o reescreve com frequência, e isso não deve disparar sync.
