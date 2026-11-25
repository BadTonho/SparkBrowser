# Dependências e plataformas

[English](../../en/security/dependency-notes.md)

## Correção de segurança JavaScript

O `@sveltejs/vite-plugin-svelte` depende de `deepmerge 4.3.1`, afetado por [GHSA-gcqx-w48m-w2h4](https://github.com/advisories/GHSA-gcqx-w48m-w2h4). No momento desta alteração, o aviso não informa uma versão corrigida pelo projeto original, e o plugin Svelte mais recente ainda depende do pacote afetado.

O override npm resolve `deepmerge` para `tools/vendor/deepmerge`, um fork privado chamado `@spark/deepmerge`. Ele contém a implementação CommonJS empacotada da versão 4.3.1 e sua licença MIT. A única mudança de comportamento usa `Object.defineProperty` nas escritas do destino originadas **tanto do target quanto do source**, impedindo que um `__proto__` enumerável invoque o setter herdado do protótipo. O fork possui uma versão explicitamente corrigida e não é publicado.

O `tests/deepmerge-security.test.mjs` resolve a dependência a partir do próprio plugin Svelte e verifica a regressão, arrays, símbolos, opções aninhadas e mesclagem personalizada. Substituir esse fork exige verificações equivalentes e uma nova auditoria npm; ele não deve ser removido apenas para reduzir os arquivos de terceiros.

## Avisos Rust e dependências Linux

O Spark inclui o SQLite por meio do `rusqlite 0.41`. A versão fixada do `libsqlite3-sys`, 0.39.1, inclui a correção da [CVE-2026-119375](https://rustsec.org/advisories/RUSTSEC-2026-0459.html) no Windows e no Linux.

A versão fixada `url 2.5.9` corrige a [RUSTSEC-2026-0592](https://rustsec.org/advisories/RUSTSEC-2026-0592.html) (estouro de pilha em `Url::origin()` com endereços `blob:` aninhados, alcançável porque o Spark lê a origem dos endereços das páginas), e `cookie 0.18.3` corrige a [RUSTSEC-2026-0594](https://rustsec.org/advisories/RUSTSEC-2026-0594.html) (nonces de cookies privados repetidos após `fork`, algo que o Spark não faz).

O `cargo audit --no-fetch` usa a base local de avisos e não certifica que ela esteja atualizada. A árvore atual do Tauri ainda inclui crates GTK3 para Linux marcados como sem manutenção e `glib 0.18.5`, afetado por [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html), além da dependência sem manutenção `proc-macro-error`.

O iterador afetado do glib foi corrigido em `glib >=0.20`, fora da faixa compatível com as dependências GTK3 usadas. Forçar essa versão na árvore existente não é uma atualização suportada. Os avisos continuam visíveis, sem exceções de auditoria. Uma migração suportada pelo Tauri/GTK ainda é necessária para considerar os avisos de dependências Linux resolvidos.

## Validação por plataforma

A validação automatizada local é feita no Windows, incluindo uma suíte WebDriver de release isolada. O fluxo de release também compila pacotes Linux e executa testes Rust; o comportamento nativo das WebViews e os instaladores exigem verificações separadas. A API de múltiplas WebViews usa a feature `unstable` do Tauri e os motores variam conforme o sistema. Não há garantia de execução em todas as plataformas nem promessa de tamanho fixo do executável.

## Dados locais

Os bancos do usuário ficam no diretório de dados do aplicativo no sistema operacional. Bancos, arquivos auxiliares WAL/SHM do SQLite, credenciais, builds e `.cache/` são excluídos do Git. Antes de commitar, confira o status e os diffs staged e unstaged; regras de ignore não protegem arquivos já rastreados.
