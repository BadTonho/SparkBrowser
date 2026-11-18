# ✨ Funcionalidades Principais e Experiência Visual

> 🌍 **Idioma:** [English (Main)](../../en/features/core-features.md) | **Português (pt-BR)**

## 1. ⚡ Command Bar Flutuante (`Ctrl + K` por padrão)
- Barra central flutuante estilo *Spotlight* animada com física de mola.
- Permite abrir URLs rapidamente, pesquisar na web, alternar entre abas abertas e executar comandos rápidos do navegador.

## 2. 💤 Hibernação Inteligente de Abas e Inspeção de Memória (Economia de RAM)
- Gerenciada nativamente em Rust.
- Ao passar o mouse sobre qualquer guia, o uso de memória física (RAM) em tempo real é exibido logo abaixo do nome da guia e do domínio na dica de contexto. Abas hibernadas indicam 0 MB para evidenciar a economia de memória.
- Suspende automaticamente abas em segundo plano após 5, 10 ou 30 minutos, ou usa o modo adaptativo **Auto**. No Auto, abas comuns hibernam após 10 minutos; abas ativadas pelo menos 10 vezes nos últimos 7 dias hibernam após 5 horas sem uso.
- **Off** desativa a hibernação automática, mantendo a hibernação manual disponível. Os horários locais de ativação são mantidos no SQLite para a janela adaptativa de 7 dias, sem armazenar URLs.

## 3. Privacy Shield

O escudo da barra abre um [pop-up de Proteção](auto-skip-ads.md) ancorado ao botão, com controles independentes de Adblock e Auto skip ads. O auto skip é um auxiliar opcional para o botão de pular do YouTube desktop; não altera posição nem velocidade da reprodução.

- O Rust rejeita navegações para hosts conhecidos de rastreamento e seus subdomínios exatos.
- Um script de página filtra fetch (incluindo objetos URL/Request), XHR e sendBeacon, e adiciona CSS. A cobertura não equivale a um bloqueador completo da rede: scripts/imagens do parser, service workers e alterações feitas pela página podem contornar esses hooks.
- No Windows, alterar a proteção atualiza os interceptadores de requisições ativos e substitui um script de política para documentos futuros sem recarregar WebViews, preservando o estado das páginas e o histórico nativo de voltar/avançar. Outros motores recriam as WebViews: a página ativa recarrega e as abas em segundo plano hibernam. Estado e histórico nativo são reiniciados nesse fallback; a mudança é recusada enquanto formulários, mídia ou downloads detectados precisam de proteção.
- O contador mede navegações rejeitadas, não todos os recursos filtrados.
- Se a gravação da sessão falhar, a alteração do shield preserva a proteção atual e os documentos.

## 4. 🎨 Workspaces (Espaços de Trabalho) e Temas Dinâmicos
- Separação de abas por contexto (ex: *Trabalho*, *Estudo*, *Pessoal*, *Jogos*).
- Em **Aparência e Layout**, escolha entre botões permanentes na barra horizontal e um seletor flutuante aberto ao manter a tecla modificadora configurada (Alt por padrão) pressionada por 0,4 segundo. No modo Popup, solte a tecla, pressione `Esc` ou clique fora para fechar o seletor.
- `Alt + 1` até `Alt + 9` alternam imediatamente para os nove primeiros espaços, conforme a ordem exibida, e mostram uma confirmação breve.
- Esses atalhos de workspace podem ser alterados em Browser Settings; o seletor usa uma tecla modificadora configurável que deve ser mantida pressionada.
- O seletor e a confirmação também aparecem temporariamente no Modo Zen; ao fechá-los, o Modo Zen continua ativo.
- Em Browser Settings, renomeie um espaço de trabalho, altere seu emoji e suas cores, crie novos ou exclua os existentes. Personal é protegido.
- Ao excluir um espaço de trabalho com abas abertas, transfira-as para outro espaço ou feche-as; as abas fechadas continuam no histórico de navegação.
- Uma falha de gravação preserva o workspace e a aba ativa. Voltar para um destino dividido restaura seu painel focado; transferir abas para ele substitui somente esse painel.
- A paleta padrão do navegador usa tons de grafite, cinza e branco. Os espaços de trabalho predefinidos começam com destaques neutros e continuam personalizáveis; cada perfil mantém suas próprias cores e gradientes sutis na moldura translúcida da janela. Editá-lo nas configurações não o ativa.

## 5. 📐 Modo Zen / Foco
- Um único atalho recolhe todas as barras do navegador, entregando **100% da tela** para o site ativo.
- No layout Vertical, sair do Modo Zen restaura a barra lateral.

## 6. 🔀 Reordenação de Guias por Arrastar (Drag-and-Drop) e Movimentação entre Workspaces
- Reordene abas na barra horizontal ou na barra lateral usando arrastar e soltar (drag-and-drop).
- Navegação rápida entre abas: pressione `Ctrl + 1` até `Ctrl + 8` para ir diretamente para a aba correspondente, e `Ctrl + 9` para ir para a última aba.
- Por padrão, pressione `Alt + ↑` ou `Alt + ↓` (ou `Ctrl + Shift + ↑` / `↓`) para mover a aba ativa. Cada atalho pode ser personalizado em Browser Settings.
- No layout Vertical, arraste abas sobre as pílulas de Workspace (*Personal*, *Work*, *Study*, *Gaming*) para transferi-las entre contextos.
- Atualização em tempo real na interface com persistência automática no SQLite em Rust (`reorder_tabs`, `move_tab_to_workspace`).

## 7. 📑 Layouts Horizontal e Vertical do Navegador
- Escolha **Horizontal** ou **Vertical** nas Configurações (`Ctrl + ,`); a seleção é lembrada ao reabrir o aplicativo.
- No layout Horizontal, arraste a janela pela área vazia da barra superior de abas. A barra de endereço/pesquisa não move a janela. No layout Vertical, a barra superior continua sendo a área de arraste da janela.
- **Horizontal:** As abas ficam no topo e os bookmarks salvos aparecem em uma barra horizontal abaixo da pesquisa. A barra fica oculta quando não há bookmarks nem pastas. Escolha se os workspaces ficam como botões ao lado da pesquisa ou em um seletor flutuante aberto ao manter Alt pressionado. Oculte ou mostre a barra de bookmarks em Aparência e Layout ou pelo botão na barra superior; ocultá-la não remove os bookmarks.
- **Vertical:** Os espaços de trabalho, bookmarks salvos e abas verticais ficam juntos na barra lateral retrátil; a preferência de exibição dos workspaces altera somente o layout horizontal.
- Salve uma página pela estrela da barra de endereço e escolha o título e a pasta. Ao selecionar um bookmark existente, o mesmo editor permite renomeá-lo, movê-lo para outra pasta ou removê-lo.
- Crie, renomeie e exclua pastas de bookmarks de um nível. Elas aparecem como menus na barra horizontal e seções expansíveis na barra lateral vertical; excluir uma pasta move seus bookmarks para a raiz.
- Clique com o botão direito em um bookmark para editá-lo ou apagá-lo, ou em uma área vazia da barra visível para criar e gerenciar pastas. Se a barra estiver vazia e oculta, crie uma pasta pelo editor de bookmark da barra de endereço. Clique com o botão direito em uma pasta para renomeá-la ou excluí-la; exclusões pedem confirmação.
- Instalações novas começam sem bookmarks de exemplo. Bancos existentes removem uma única vez somente as entradas padrão originais do DuckDuckGo, GitHub, Rust Lang e Wikipedia; os demais bookmarks são preservados.

## 8. Configurações do Navegador Organizadas
- Em **Aparência e Layout**, escolha se as configurações abrem em pop-up ou em uma guia. A mudança transfere a visualização atual imediatamente; ao abrir novamente, a guia existente no workspace atual é reutilizada.
- As configurações são divididas em **Aparência e Layout**, **Pesquisa e Navegação**, **Espaços de Trabalho e Tema**, **Desempenho e Privacidade** e **Atalhos de Teclado**.
- Pesquise em todas as categorias, configurações e atalhos pelo campo ao lado do título Browser Settings; selecionar um resultado abre a categoria e destaca o destino.
- Escolha Google ou DuckDuckGo como mecanismo de busca padrão em **Pesquisa e Navegação**. Google vem selecionado por padrão; a escolha vale para buscas na barra de endereço, na Command Bar e para novas guias.
- Personalize a superfície do fundo das janelas entre **Fosco**, **Vidro** e **Gelo** (um desfoque forte), regule a opacidade das janelas de 30% a 100% e ajuste o desfoque das janelas de 0 a 32 px. Consulte [Aparência das Janelas](window-appearance.md).
- Ajuste em Aparência e Layout o desfoque de fundo compartilhado pelos overlays temporários, de **0 a 24 px**; a preferência é aplicada imediatamente e lembrada após reiniciar.
- Escolha entre a aparência **Escura** e **Clara** em Aparência e Layout. O modo escuro vem ativado por padrão; as mudanças se aplicam imediatamente à interface, configurações e overlays, sendo lembradas entre reinicializações.
- Mostre ou oculte a barra horizontal de bookmarks em Aparência e Layout ou pelo botão na barra superior; a escolha é lembrada após reiniciar e não afeta os bookmarks da barra lateral vertical. A barra só aparece quando há pelo menos um bookmark ou pasta.
- Use a barra fixa de categorias para alternar entre os painéis; somente o painel selecionado é exibido e rola quando seu conteúdo ultrapassa a altura da janela.
- A última categoria selecionada é restaurada ao reabrir o modal e também após reiniciar o aplicativo.
- Em **Keyboard Shortcuts**, clique em um atalho e pressione uma combinação de teclas ou Mouse 4/5 para gravá-lo. O seletor de workspaces pode usar uma tecla modificadora mantida pressionada (Alt, Ctrl, Shift ou Meta); combinações duplicadas são rejeitadas. As alterações são salvas entre reinicializações e funcionam com foco na interface do navegador ou em uma página.
- Mouse 4 e Mouse 5 voltam e avançam por padrão e podem ser reconfigurados em **Keyboard Shortcuts**. Use `Ctrl + E` para focar e selecionar o endereço atual na barra combinada de endereço e busca.

## 9. Downloads diretos

O gerenciador transfere recursos HTTP/HTTPS em segundo plano. Ele grava um arquivo temporário, verifica a conclusão e salva sem substituir arquivos existentes antes de registrar **Completed**. O painel mostra erros e impede envios duplicados durante a transferência.

O cliente HTTP é independente da sessão autenticada da página e salva os bytes da resposta, sem criar um arquivo completo da página para uso offline. Downloads exigem o aplicativo desktop; o preview no navegador informa um erro em vez de simular sucesso.
