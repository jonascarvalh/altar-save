# Publicando uma release

As releases são geradas pelo GitHub Actions ([`.github/workflows/release.yml`](../.github/workflows/release.yml)). Cada tag `v*` enviada ao GitHub dispara o build no Windows e no Linux, e os instaladores são anexados a um **rascunho de release**. Nada é publicado sem você revisar.

| Sistema | Arquivos gerados |
|---|---|
| Windows | `Altar Save_<versão>_x64-setup.exe` (NSIS) |
| Linux | `.deb`, `.rpm` e `.AppImage` |

## Passo a passo

1. Esteja na `main`, atualizada, com tudo o que vai na versão já mesclado.

2. Atualize a versão em todos os arquivos de uma vez:

   ```bash
   npm run version:set -- 0.2.0
   ```

   O script altera `package.json`, `package-lock.json`, `src-tauri/Cargo.toml` e `src-tauri/Cargo.lock`. O `tauri.conf.json` e a versão exibida na barra inferior do app leem do `package.json`.

3. Faça commit, crie a tag e envie as duas:

   ```bash
   git commit -am "chore: versão 0.2.0"
   git tag v0.2.0
   git push && git push origin v0.2.0
   ```

4. Acompanhe o build em **Actions** no GitHub (uns 10 a 15 minutos; as próximas versões são mais rápidas por causa do cache).

5. Em **Releases**, abra o rascunho **Altar Save v0.2.0**, escreva as novidades e clique em **Publish release**.

## Versionamento

Use [versionamento semântico](https://semver.org/lang/pt-BR/):

| Mudança | Exemplo |
|---|---|
| correções | `0.2.0` → `0.2.1` |
| funcionalidades novas | `0.2.1` → `0.3.0` |
| mudança incompatível (ex.: formato do bucket) | `0.3.0` → `1.0.0` |

Para versões de teste, use um sufixo: `0.3.0-beta.1`. Marque o rascunho como **pre-release** antes de publicar.

## Se algo der errado

| Problema | Solução |
|---|---|
| O workflow falha em **Conferir a versão da tag** | A tag não bate com a versão dos arquivos. Apague a tag (`git tag -d v0.2.0 && git push origin :refs/tags/v0.2.0`), rode `npm run version:set -- 0.2.0`, faça commit e crie a tag de novo |
| O build falhou em um dos sistemas | Corrija na `main`, apague o rascunho e a tag (comando acima) e crie a tag de novo no commit corrigido |
| Quero testar o build sem publicar | Crie uma tag de teste (ex.: `v0.2.0-beta.1`); o resultado fica só como rascunho, que pode ser apagado |

## Limitações atuais

- **Instalador do Windows sem assinatura**: o SmartScreen avisa "O Windows protegeu o computador" na primeira execução. O usuário precisa clicar em **Mais informações → Executar assim mesmo**. A assinatura exige um certificado de code signing.
- **Sem atualização automática**: quem já instalou precisa baixar a versão nova manualmente. Dá para adicionar depois com o [`tauri-plugin-updater`](https://v2.tauri.app/plugin/updater/), que exige um par de chaves de assinatura guardado como secret no GitHub.
- **Sem build para macOS**: dá para incluir `macos-latest` na matriz do workflow, mas o app nunca foi testado lá.
