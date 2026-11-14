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

Os logotipos são caminhos vetoriais de uma cor do projeto [Simple Icons](https://github.com/simple-icons/simple-icons) (CC0 1.0), guardados como texto em `src/lib/components/icons/searchEngineIcons.js` e desenhados na cor de cada marca sobre um cartão claro, para que cores escuras continuem legíveis no tema escuro. Google, DuckDuckGo, Brave, Startpage, Ecosia, Qwant, Mojeek e Kagi vêm da versão 13, o Bing da versão 12 e o Yahoo da versão 9, porque as versões seguintes removeram esses dois. Nada é baixado em tempo de execução, então abrir as configurações não contata nenhum mecanismo de busca. As marcas continuam sendo propriedade de seus donos e só identificam o mecanismo.

## Compatibilidade

- Os ids dos mecanismos (`google`, `duckduckgo`, `bing`, `brave`, `startpage`, `ecosia`, `qwant`, `yahoo`, `mojeek`, `kagi`) são guardados nas configurações e nos backups, então nunca mudam.
- Um id desconhecido ou antigo volta para o Google em vez de falhar, tanto no lado nativo quanto na interface.
- Backups que contêm um dos novos mecanismos são recusados por versões antigas do aplicativo, que só conheciam `google` e `duckduckgo`. Importar nesta versão um backup que escolheu um mecanismo novo funciona, e aplicar as preferências o restaura.

## Adicionar um mecanismo

1. Inclua uma entrada em `src-tauri/src/search_engines.rs` (id, nome, página inicial, página de resultados e parâmetro da pesquisa).
2. Inclua a mesma entrada, na mesma ordem, em `src/lib/services/search-engines.js`, com uma `color`.
3. Adicione `settings.engine_<id>_desc` nos dois catálogos de configurações.
4. Adicione o caminho do logotipo em `src/lib/components/icons/searchEngineIcons.js` (24×24, somente dados de caminho).

`tests/search-engines.test.mjs` falha se as duas listas divergirem, se faltar uma descrição ou um logotipo, se um logotipo tiver algo além de dados de caminho ou se um mecanismo usar endereço que não seja https; os testes Rust conferem que cada mecanismo monta uma URL de busca válida.

## Implementação

- Nativo: `search_engines.rs` é o registro único usado por `url_input.rs`, pelas preferências da nova aba e pela validação do backup.
- Interface: `search-engines.js` alimenta `searchEngineStore.ts`, a prévia offline em `site-search.js` e `SearchEnginePicker.svelte`. O seletor usa botões simples com `role="radio"`, que também funcionam na superfície nativa de configurações. Essa superfície não copia estilos inline, então as cores das marcas vão no atributo `fill` do ícone.
