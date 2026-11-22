# Mecanismos de busca

Configurações → Busca e navegação → **Buscador padrão** agora é uma grade de cartões em vez de uma lista suspensa: cada mecanismo mostra seu logotipo, o nome e uma descrição de uma linha, e o escolhido é contornado por uma borda (nada mais muda, então a grade não se mexe). A escolha vale para pesquisas digitadas na barra de endereço, para novas abas e para o botão de início.

## Mecanismos disponíveis

| Mecanismo | Observações |
| --- | --- |
| Google | Padrão; resultados mais completos |
| DuckDuckGo | Privado, sem perfil de rastreamento |
| Bing | Busca da Microsoft |
| Brave Search | Índice próprio e privado |
| Startpage | Resultados do Google, de forma anônima |
| Ecosia | Planta árvores com a receita dos anúncios |
| Qwant | Europeu e privado |
| Yahoo | Busca clássica (o parâmetro da pesquisa é `p`, não `q`) |
| Mojeek | Independente, sem rastreamento |
| Kagi | Sem anúncios, por assinatura; os resultados exigem login no Kagi |

As descrições são resumos curtos para ajudar a escolher, não garantias: vale a política de privacidade de cada serviço. O Google é o padrão em perfis novos, e também é o que qualquer valor desconhecido ou antigo usa como alternativa.

Os atalhos de busca por site (por exemplo `yt gatos`) continuam tendo prioridade sobre o buscador padrão; veja [Atalhos de busca por site](site-search.md).

## Logotipos

Os logotipos ficam sobre bases neutras que acompanham o tema atual. O G multicolorido e o pato Dax usam elementos do [kit de marca do Google](https://developers.google.com/identity/branding-guidelines#download_pre-approved_brand_icons) e do [kit de imprensa do DuckDuckGo](https://duckduckgo.com/press). Os demais usam vetores monocromáticos do [Simple Icons](https://github.com/simple-icons/simple-icons) (CC0 1.0). O Bing usa a versão 12, o Yahoo usa a versão 9 e os demais usam a versão 13. O Spark inclui os elementos para uso offline. Cada marca identifica seu mecanismo de busca e pertence ao respectivo titular.

## Compatibilidade

- Os ids dos mecanismos (`google`, `duckduckgo`, `bing`, `brave`, `startpage`, `ecosia`, `qwant`, `yahoo`, `mojeek`, `kagi`) são guardados nas configurações e nos backups, então nunca mudam.
- Um id desconhecido ou antigo volta para o Google em vez de falhar, tanto no lado nativo quanto na interface.
- Backups que contêm um dos novos mecanismos são recusados por versões antigas do aplicativo, que só conheciam `google` e `duckduckgo`. Importar nesta versão um backup que escolheu um mecanismo novo funciona, e aplicar as preferências o restaura.

## Adicionar um mecanismo

1. Inclua uma entrada em `src-tauri/src/search_engines.rs` (id, nome, página inicial, página de resultados e parâmetro da pesquisa).
2. Inclua a mesma entrada, na mesma ordem, em `src/lib/services/search-engines.js`, com uma `color`.
3. Adicione `settings.engine_<id>_desc` nos dois catálogos de configurações.
4. Adicione o caminho do logotipo em `src/lib/components/icons/searchEngineIcons.js` (24×24, somente dados de caminho).

`tests/search-engines.test.mjs` confere se as listas coincidem, se todos os mecanismos têm uma descrição e um ícone incluído e se todos usam HTTPS. Os testes Rust conferem se cada mecanismo monta uma URL de busca válida.

## Implementação

- Nativo: `search_engines.rs` é o registro único usado por `url_input.rs`, pelas preferências da nova aba e pela validação do backup.
- Interface: `search-engines.js` alimenta `searchEngineStore.ts`, a prévia offline em `site-search.js` e `SearchEnginePicker.svelte`. O seletor usa botões com `role="radio"`. Os estilos dos ícones e os atributos SVG de preenchimento funcionam nas duas formas de exibir as Configurações.
