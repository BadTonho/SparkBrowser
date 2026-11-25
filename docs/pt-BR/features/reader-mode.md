# Modo Leitura Imersivo (Reader View)

O Spark Browser conta com um Modo Leitura personalizável e livre de distrações, que remove anúncios, barras de navegação, menus laterais e elementos visuais pesados para apresentar artigos com tipografia elegante e foco total no conteúdo.

---

## Funcionalidades

* **Extração Inteligente de Conteúdo:** Analisa o DOM da página para isolar títulos, subtítulos, autoria/byline, parágrafos, citações, blocos de código e imagens relevantes, eliminando banners publicitários, scripts de rastreamento e barras desnecessárias.
* **Temas Confortáveis para os Olhos:**
  * **Papel (Paper):** Paleta suave para leitura diurna (`#FBFBF9`).
  * **Sépia:** Paleta estilo pergaminho quente e de baixo contraste, ideal para leitura prolongada (`#F6EFE2`).
  * **Escuro (Dark):** Fundo grafite moderno que reduz a fadiga visual à noite (`#1E1E24`).
  * **Preto OLED:** Fundo preto absoluto que economiza energia em telas OLED (`#000000`).
* **Tipografia e Dimensões Personalizadas:**
  * **Serifa:** Tipografia editorial clássica (Merriweather / Georgia).
  * **Sem Serifa:** Tipografia moderna e limpa (Inter / System).
  * **Monoespaçada:** Ideal para tutoriais e artigos técnicos de programação.
  * **Tamanho de Fonte Dinâmico:** Ajuste fino de 14px a 28px através dos botões A- / A+.
  * **Largura de Coluna:** Estreita (640px), Média (760px) e Larga (940px).
* **Segurança do Conteúdo:** Scripts, iframes, formulários, objetos incorporados e manipuladores de evento inline são removidos dos artigos extraídos. Links no texto do artigo aparecem como texto simples.
* **Imagens do Artigo:** As imagens são lidas do cache da própria aba (a cópia que a página já carregou) e exibidas como imagens `data:` incorporadas, porque a interface do navegador não carrega imagens remotas. Nada é baixado de novo. Imagens que a página ainda não tinha carregado (por exemplo, imagens com carregamento tardio mais abaixo), imagens vetoriais e imagens acima de 4 MB (ou além de 24 MB no total, até 40 por artigo) ficam de fora.
* **Plataformas:** A extração de artigos está disponível no Windows. No Linux, abrir o Modo Leitura mostra uma mensagem, no idioma da interface, dizendo que ele está disponível apenas no Windows.
* **Métricas de Leitura:** Informa o tempo estimado de leitura ("~4 min de leitura") e contagem total de palavras.
* **Integração com a Janela Nativa:** Oculta temporariamente o WebView nativo subjacente com `setOverlayVisibility(true, "reader-mode")`, garantindo que o leitor nunca sofra interferência da página web de fundo.

---

## Como Usar o Modo Leitura

1. Acesse qualquer artigo de notícias, post de blog ou página editorial.
2. Na barra de endereço (Omnibox), o botão **Modo Leitura (ícone de livro)** surgirá automaticamente.
3. Clique no botão para abrir o leitor imersivo.
4. Ajuste tema, tipografia, largura e tamanho de fonte na barra superior flutuante.
5. Pressione **Escape (`Esc`)** ou clique em **Fechar Modo Leitura** para retornar à página web original.

