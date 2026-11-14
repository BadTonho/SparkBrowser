# Exportar favoritos para outros navegadores

Configurações → Dados e backup tem um cartão **Favoritos** com dois botões que gravam todos os favoritos em um arquivo que outros navegadores sabem ler. Isso é separado do backup `.veloce-backup.json`, que só este aplicativo restaura.

## Formatos

- **HTML** (`.html`, `.htm`) é o formato Netscape de favoritos. Chrome, Edge, Firefox, Safari, Brave, Vivaldi e a maioria dos gerenciadores de favoritos o importam (por exemplo, Chrome → Favoritos → Importar favoritos, Firefox → Favoritos → Gerenciar → Importar).
- **CSV** (`.csv`) tem as colunas `folder,title,url`, um favorito por linha, para planilhas e scripts. É UTF-8 com marca de ordem de bytes para o Excel mostrar títulos acentuados corretamente, com quebras de linha CRLF e aspas conforme a RFC 4180.

As pastas são exportadas em um único nível, na ordem das pastas, seguidas dos favoritos sem pasta. Pastas vazias ficam de fora. Um favorito cuja pasta não existe mais é exportado como avulso. Ícones e datas de criação não são exportados.

## Segurança

- Só endereços `http` e `https` simples são exportados. Qualquer outro (`javascript:`, `data:` e semelhantes) é ignorado e contado no aviso exibido após a exportação, para que um favorito malicioso nunca chegue à barra de outro navegador.
- Títulos, nomes de pastas e endereços do HTML são escapados, e caracteres de controle são removidos.
- Células do CSV que começam com `=`, `+`, `-`, `@`, tabulação ou retorno de carro recebem um apóstrofo no início, para que a planilha não execute o título de uma página como fórmula.
- O nome de arquivo confirmado no diálogo de salvar nunca é alterado depois. Um nome que não combina com o formato (`.html`/`.htm` ou `.csv`) é recusado em vez de renomeado em silêncio. O arquivo é gravado em um temporário e movido para o lugar, então uma falha preserva qualquer arquivo anterior.
- Sem favoritos para exportar, o comando avisa em vez de gravar um arquivo vazio. Cancelar o diálogo não faz nada.

Favoritos não são dados secretos, mas o arquivo contém sua lista de leitura em texto simples. Nada sai do computador.

## Ainda não incluído

- Importar favoritos de um arquivo HTML de outro navegador.
- Exportar sessões salvas, notas ou abas nesses formatos.

## Implementação

- A formatação é código puro em `src-tauri/src/backup/bookmark_export.rs` (com testes).
- O comando `export_bookmarks` em `src-tauri/src/commands/bookmark_export.rs` lê os favoritos, abre o diálogo nativo de salvar pelo Rust e grava de forma atômica. Os comandos JavaScript do plugin de diálogos do Tauri continuam sem permissão.
- A interface é `BookmarkExportCard.svelte`, apoiada por `exportBookmarksAs` em `backupStore.ts`.
