# ✨ Funcionalidades Principais e Experiência Visual

> 🌍 **Idioma:** [English (Main)](../../en/features/core-features.md) | **Português (pt-BR)**

## 1. ⚡ Command Bar Flutuante (`Ctrl + T` / `Ctrl + K`)
- Barra central flutuante estilo *Spotlight* animada com física de mola.
- Permite abrir URLs rapidamente, pesquisar na web, alternar entre abas abertas e executar comandos rápidos do navegador.

## 2. 💤 Hibernação Inteligente de Abas (Economia de RAM)
- Gerenciada nativamente em Rust.
- Suspende automaticamente abas em segundo plano após um tempo configurável de inatividade, liberando memória RAM do sistema enquanto mantém a aba visível e pronta para uso na barra lateral.

## 3. 🛡️ AdBlock e Privacidade Nativa em Rust
- Baseado no `adblock-rust` para bloquear anúncios intrusivos, pop-ups e scripts de rastreamento diretamente na camada de rede.
- Melhora drasticamente a velocidade de carregamento dos sites e reduz o consumo de memória e internet.

## 4. 🎨 Workspaces (Espaços de Trabalho) e Temas Dinâmicos
- Separação de abas por contexto (ex: *Trabalho*, *Estudo*, *Pessoal*, *Jogos*).
- Segure `Alt` por 0,4 segundo para abrir o seletor central de espaços de trabalho. Escolha um perfil enquanto mantém Alt pressionado ou solte Alt, pressione `Esc` ou clique fora para fechar sem trocar.
- `Alt + 1` até `Alt + 9` alternam imediatamente para os nove primeiros espaços, conforme a ordem exibida, e mostram uma confirmação breve.
- O seletor e a confirmação também aparecem temporariamente no Modo Zen; ao fechá-los, o Modo Zen continua ativo.
- Em Browser Settings, renomeie um espaço de trabalho, altere seu emoji e suas cores, crie novos ou exclua os existentes. Personal é protegido.
- Ao excluir um espaço de trabalho com abas abertas, transfira-as para outro espaço ou feche-as; as abas fechadas continuam no histórico de navegação.
- A paleta padrão do navegador usa tons de grafite, cinza e branco. Os espaços de trabalho predefinidos começam com destaques neutros e continuam personalizáveis; cada perfil mantém suas próprias cores e gradientes sutis na moldura translúcida da janela. Editá-lo nas configurações não o ativa.

## 5. 📐 Modo Zen / Foco
- Um único atalho recolhe todas as barras do navegador, entregando **100% da tela** para o site ativo.
- No layout Vertical, sair do Modo Zen restaura a barra lateral.

## 6. 🔀 Reordenação de Guias por Arrastar (Drag-and-Drop) e Movimentação entre Workspaces
- Reordene abas na barra horizontal ou na barra lateral usando arrastar e soltar (drag-and-drop).
- Navegação rápida entre abas: pressione `Ctrl + 1` até `Ctrl + 8` para ir diretamente para a aba correspondente, e `Ctrl + 9` para ir para a última aba.
- Atalhos de teclado globais: pressione `Alt + ↑` e `Alt + ↓` (ou `Ctrl + Shift + ↑` / `↓`) em qualquer lugar da aplicação para mover a guia ativa para cima ou para baixo imediatamente.
- No layout Vertical, arraste abas sobre as pílulas de Workspace (*Personal*, *Work*, *Study*, *Gaming*) para transferi-las entre contextos.
- Atualização em tempo real na interface com persistência automática no SQLite em Rust (`reorder_tabs`, `move_tab_to_workspace`).

## 7. 📑 Layouts Horizontal e Vertical do Navegador
- Escolha **Horizontal** ou **Vertical** nas Configurações (`Ctrl + ,`); a seleção é lembrada ao reabrir o aplicativo.
- **Horizontal:** As abas ficam no topo, os botões dos espaços de trabalho ficam ao lado da barra de pesquisa e os bookmarks salvos aparecem em uma barra horizontal abaixo dela.
- **Vertical:** Os espaços de trabalho, bookmarks salvos e abas verticais ficam juntos na barra lateral retrátil.

## 8. Configurações do Navegador Organizadas
- As configurações são divididas em **Aparência e Layout**, **Pesquisa e Navegação**, **Espaços de Trabalho e Tema**, **Desempenho e Privacidade** e **Atalhos de Teclado**.
- Use a barra fixa de categorias para alternar entre os painéis; somente o painel selecionado é exibido e rola quando seu conteúdo ultrapassa a altura da janela.
- A última categoria selecionada é restaurada ao reabrir o modal e também após reiniciar o aplicativo.
