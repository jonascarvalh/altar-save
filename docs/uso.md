# Usando o app

## Biblioteca

Os saves agrupados por memory card, com status, tamanho, última alteração e a máquina que enviou por último.

| Status | Significado |
|---|---|
| Em dia | local e nuvem iguais |
| Pendente | há algo para enviar ou baixar |
| Conflito | o save mudou neste PC e na nuvem desde o último sync |
| Aguardando | esperando o emulador fechar (download, exclusão ou arquivo em uso) |
| Erro | a última tentativa falhou; a mensagem aparece no card |

- **Sincronizar agora** força um sync completo.
- **Conflito**: escolha **Manter local** (envia a versão deste PC) ou **Manter nuvem** (baixa; a local vai para o backup).
- **Excluir**: passe o mouse no card e clique na lixeira. Há uma confirmação antes, e a exclusão vale para todas as máquinas (veja [Exclusão com lápide](sincronizacao.md#exclusão-com-lápide)).

> Apagar um save **só no PC** faz o app baixá-lo de novo. Para apagar de verdade, use **Excluir** no app.

## Automações

Liga e desliga o PCSX2 e o sync automático, define a pasta e mostra o que foi detectado. Detalhes em [Configurando o PCSX2](pcsx2.md).

## Nuvem

Provedor, credenciais e teste de conexão. Detalhes em [Configurando a nuvem](nuvem.md).

## Atividade

Histórico de envios, downloads, exclusões, conflitos e erros, com filtros, e o botão **Abrir backups**.

## Barra de título e barra inferior

- **Barra de título**: o ícone de sol/lua alterna entre tema claro e escuro. Até a primeira escolha, o app segue o tema do sistema.
- **Barra inferior**: progresso do sync em andamento, resultado do último e se o monitoramento está ativo.
