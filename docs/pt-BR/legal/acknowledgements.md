# Créditos e licenças de terceiros

[English](../../en/legal/acknowledgements.md)

O Spark é construído sobre o trabalho de muitos projetos e serviços de código aberto. Esta página credita os que o Spark usa diretamente e explica os termos de licença que se aplicam a eles. O código do próprio Spark é licenciado sob a [Apache License 2.0](../../../LICENSE).

## Dados e serviços

| Projeto | Para que o Spark usa | Licença e termos |
| --- | --- | --- |
| [Open-Meteo](https://open-meteo.com/) | Previsão do tempo e busca de cidades no widget de clima da nova aba | Dados sob [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/); a atribuição aparece no widget. O endpoint gratuito é para uso não comercial; veja a [página de preços](https://open-meteo.com/en/pricing). |
| [EasyList e EasyPrivacy](https://easylist.to/) | Listas de filtros incorporadas para o escudo de anúncios e rastreadores | GPL-3.0-or-later e CC BY-SA 3.0. Mantidas sem modificação, com os cabeçalhos originais. Veja o [aviso das listas de filtros](../../../src-tauri/resources/filterlists/README.md). |
| [NoCoin Filter List](https://github.com/hoshsadiq/adblock-nocoin-list), de Hosh | Lista incorporada que bloqueia mineradores de criptomoedas no navegador | MIT, veja [`NOCOIN-LICENSE.txt`](../../../src-tauri/resources/filterlists/NOCOIN-LICENSE.txt). |

## Recursos visuais

- **[Simple Icons](https://github.com/simple-icons/simple-icons)** (CC0 1.0): os logotipos monocromáticos dos buscadores. Versões e detalhes estão em [Mecanismos de busca](../features/search-engines.md).
- Nomes e logotipos de marcas pertencem aos seus donos. A presença deles no Spark não indica afiliação nem endosso.

## Código incluído no repositório

- **[deepmerge](https://github.com/TehShrike/deepmerge)** 4.3.1 (MIT, copyright James Halliday, Josh Duff e outros colaboradores): uma cópia privada e corrigida fica em `tools/vendor/deepmerge`, com a [licença](../../../tools/vendor/deepmerge/LICENSE) original. Ela afeta apenas as ferramentas de build, e o motivo da correção está nas [notas de dependências](../security/dependency-notes.md).

## Tecnologias principais

| Projeto | Papel no Spark |
| --- | --- |
| [Tauri](https://tauri.app/) (com Wry e Tao) | Framework do aplicativo desktop, janelas e web views |
| [Svelte](https://svelte.dev/), [Vite](https://vitejs.dev/) e [TypeScript](https://www.typescriptlang.org/) | Interface do navegador e ferramentas de build |
| [adblock-rust](https://github.com/brave/adblock-rust), da Brave | Motor de regras de rede por trás do escudo (MPL-2.0) |
| [SQLite](https://www.sqlite.org/) via [rusqlite](https://github.com/rusqlite/rusqlite) | Armazenamento local |
| [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) e [WebKitGTK](https://webkitgtk.org/) | Motores de conteúdo web no Windows e no Linux |
| [windows-rs](https://github.com/microsoft/windows-rs) e [webview2-com](https://github.com/wravery/webview2-rs) | Integração nativa com o Windows |
| [gtk-rs](https://gtk-rs.org/) e [webkit2gtk-rs](https://github.com/tauri-apps/webkit2gtk-rs) | Integração nativa com o Linux |
| [RustCrypto](https://github.com/RustCrypto) (`aes-gcm`, `argon2`, `sha2`) e [rustls](https://github.com/rustls/rustls) | Criptografia, hashing e TLS |
| [tokio](https://tokio.rs/), [reqwest](https://github.com/seanmonstar/reqwest), [serde](https://serde.rs/) e [url](https://github.com/servo/rust-url) | Trabalho assíncrono, HTTP e tratamento de dados |
| [window-vibrancy](https://github.com/tauri-apps/window-vibrancy) | Efeitos nativos de janela |
| [qrcode](https://github.com/soldair/node-qrcode) | Geração de QR codes |

Esta lista mostra só as dependências diretas. As dependências de cada projeto são creditadas pelos próprios autores nos seus repositórios.

## Licenças das dependências

Todas as dependências, diretas e indiretas, são de código aberto. As dependências Rust vêm do `Cargo.lock` e as de JavaScript do `package-lock.json`.

- A grande maioria usa MIT, Apache-2.0 ou ambas. As demais usam ISC, BSD, Zlib, Unicode ou termos permissivos semelhantes.
- Algumas crates usam MPL-2.0, como `adblock`, `cssparser` e `selectors`. A MPL-2.0 vale por arquivo: ela é cumprida mantendo os fontes e avisos dessas crates intactos, o que o Cargo faz, e não se estende ao código do próprio Spark.
- Nenhuma dependência exige os termos da GPL, LGPL ou AGPL. Quando uma crate oferece uma escolha que inclui licença copyleft (por exemplo `r-efi`), o Spark a usa sob a opção permissiva.

Você pode listar as licenças exatas do seu build com `cargo metadata --manifest-path src-tauri/Cargo.toml` e `npm ls --all`.

## Independência

O Spark é um projeto independente. Ele não é afiliado, endossado nem patrocinado por Google, Microsoft, Brave, Mozilla ou qualquer outra empresa ou projeto citado nesta página. Nomes de produtos e empresas são marcas registradas dos seus respectivos donos.
