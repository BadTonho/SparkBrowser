# Abas Efêmeras (Containers Descartáveis)

O Veloce Browser oferece suporte a abas temporárias e descartáveis ("Abas Efêmeras"), projetadas para navegação rápida, testes ou acessos privados que não deixam nenhum rastro na sua máquina.

---

## Funcionalidades

* **Armazenamento Temporário e Isolado:** Cada aba efêmera opera em um diretório de dados dedicado e isolado em `webview-ephemeral/<tab_id>`. Cookies, localStorage, sessionStorage e cache HTTP ficam completamente separados das suas outras abas e workspaces.
* **Destruição Imediata de Dados:** Ao fechar uma aba efêmera, toda a pasta temporária de dados associada a ela é excluída permanentemente do disco em segundo plano.
* **Zero Persistência em Histórico e Sessão:**
  * Abas efêmeras são excluídas da tabela `session_tabs` no SQLite e nunca são restauradas após reiniciar o navegador.
  * Páginas acessadas em abas efêmeras nunca são registradas no histórico de navegação SQLite (`record_history_visit`).
  * Abas efêmeras fechadas não são salvas na fila de abas recentemente fechadas.
* **Distinção Visual Sofisticada:** Abas efêmeras exibem uma identificação roxa com ícone de discrição (`🕵️`), permitindo diferenciar claramente containers descartáveis tanto na barra horizontal quanto na barra vertical.

---

## Como Abrir uma Aba Efêmera

1. **Barra de Abas Horizontal:** Clique no botão circular roxo localizado ao lado do botão `+` (Nova Aba).
2. **Barra Lateral Vertical:** Clique no botão **Efêmera** no cabeçalho da lista de abas.
3. Ao terminar, basta fechar a aba: todo o conteúdo e cookies gerados são apagados imediatamente.

