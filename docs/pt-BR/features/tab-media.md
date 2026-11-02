# Controles de mídia das abas

No Windows, abas que emitem áudio mostram um botão de alto-falante nos dois layouts. Clique para mutar toda a aba, incluindo frames e Web Audio, sem selecioná-la. Abas mutadas mantêm o botão visível. O ajuste acompanha a aba na navegação e hibernação durante a execução; abas novas, reabertas e restauradas na inicialização começam sem mute. Preview e motores incompatíveis não exibem um controle de mute funcional.

## Volume da página

O botão de mídia junto ao endereço abre um painel acessível por teclado para a aba focada, inclusive na divisão. Ajuste de 0 a 200% em passos de 1% ou restaure 100%. O mute preserva o nível escolhido. Abas novas, reabertas e restauradas na inicialização começam em 100%; navegação, hibernação e troca de workspace mantêm os ajustes durante a execução.

100% preserva o nível próprio do player. Mídias HTML compatíveis podem ser amplificadas localmente com Web Audio e captura do elemento, sem gravar ou transmitir conteúdo. Players incompatíveis preservam o áudio original e limitam a amplificação a 100%; fontes sem suporte ao volume mantêm o mute nativo disponível. Restrições de autoplay podem exigir interação com a página. O percentual exibido é o aplicado, com indicação de operações indisponíveis ou pendentes.

O backend Rust acompanha contextos padrão de execução, incluindo frames, e vincula operações fixas à geração da WebView e ao documento. Ajustes rápidos do slider são agrupados e documentos antigos não recebem alterações destinadas à página anterior. Nenhuma preferência de mídia é gravada no SQLite ou em backups.

Execute `--case media-volume` na suíte Windows isolada para atenuação, medição da onda amplificada, independência do volume do player, agrupamento, navegação antiga, fallback entre origens e valores iniciais após reiniciar.
