# Dados locais, backups e reset

## Onde ficam os dados

| Sistema | Pasta |
|---|---|
| Windows | `%APPDATA%\com.savenuvem.app` (cache do WebView em `%LOCALAPPDATA%\com.savenuvem.app`) |
| Linux | `~/.local/share/com.savenuvem.app` (junto com o cache do WebView) |

> O identificador `com.savenuvem.app` vem do nome antigo do projeto e foi mantido para não perder dados de instalações existentes.

| Arquivo | Conteúdo |
|---|---|
| `config.json` | nuvem (**inclusive as chaves, em texto puro**) e automações |
| `sync-state.json` | base de cada save, usada nas [decisões de sync](sincronizacao.md#decisão-em-3-vias) |
| `activity.json` | histórico (últimos 300 eventos) |
| `backups/` | cópias feitas antes de downloads e exclusões (10 por save, por padrão) |

Os backups podem ser abertos pelo botão **Abrir backups**, na tela **Atividade**.

## Reset completo

Com o app fechado:

**Windows (PowerShell)**

```powershell
Remove-Item -Recurse -Force "$env:APPDATA\com.savenuvem.app", "$env:LOCALAPPDATA\com.savenuvem.app"
```

**Linux**

```bash
rm -rf ~/.local/share/com.savenuvem.app ~/.cache/com.savenuvem.app
```

O bucket não é afetado. Para zerar a nuvem também, apague a pasta configurada no painel do provedor.

> Apagar só o `sync-state.json` faz o app perder a base: saves diferentes entre PC e nuvem viram conflito, e nada é sobrescrito sem você escolher.
