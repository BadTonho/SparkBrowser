# Abas Efêmeras (Containers Descartáveis)

O Spark Browser oferece abas temporárias e descartáveis ("Abas Efêmeras") para navegação temporária e privada.

---

## Funcionalidades

* **Armazenamento Temporário e Isolado:** Cada aba efêmera opera em um diretório de dados dedicado e isolado em `webview-ephemeral/<tab_id>`. Cookies, localStorage, sessionStorage e cache HTTP ficam completamente separados das suas outras abas e workspaces.
* **Limpeza dos dados temporários:** Ao fechar uma aba, a exclusão do diretório é agendada, com novas tentativas se o mecanismo ainda mantiver arquivos abertos. Os dados restantes são removidos de forma síncrona após obter o bloqueio do perfil na próxima inicialização. Se a limpeza falhar na inicialização, o navegador informa o erro em vez de reutilizar o diretório.
* **Zero Persistência em Histórico e Sessão:**
  * Abas efêmeras são excluídas da tabela `session_tabs` no SQLite e nunca são restauradas após reiniciar o navegador.
  * Páginas acessadas em abas efêmeras nunca são registradas no histórico de navegação SQLite (`record_history_visit`).
  * Abas efêmeras fechadas não são salvas na fila de abas recentemente fechadas.
* **Distinção Visual Sofisticada:** Abas efêmeras exibem uma identificação roxa com ícone de discrição (`🕵️`), permitindo diferenciar claramente containers descartáveis tanto na barra horizontal quanto na barra vertical.

---

## Como Abrir uma Aba Efêmera

1. **Barra de Abas Horizontal:** Clique no botão circular roxo localizado ao lado do botão `+` (Nova Aba).
2. **Barra Lateral Vertical:** Clique no botão **Efêmera** no cabeçalho da lista de abas.
3. Ao fechar a aba, a limpeza dos dados temporários da WebView é agendada. Downloads e dados salvos fora desse diretório não são apagados.

