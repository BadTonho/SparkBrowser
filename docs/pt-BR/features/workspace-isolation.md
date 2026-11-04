# Isolamento de conteúdo por workspace

Estado: **implementado, opt-in e desligado por padrão**. O modelo de sites/cookies apoiado nas fronteiras do aplicativo mantém um único perfil de navegação; este recurso adiciona separação opcional por workspace.

## O que ele muda

Configurações → Workspaces expõe **“Isolar conteúdo por workspace”**. Estando ativo, cada WebView de conteúdo recebe a própria pasta de dados do WebView2, derivada do identificador do workspace responsável:

- Cookies, `localStorage`/`IndexedDB` e caches HTTP tornam-se específicos do workspace; o mesmo site aberto em dois workspaces mantém duas sessões independentes.
- O runtime do WebView2 mantém um processo de navegador separado por pasta, o que também adiciona uma fronteira de processo entre workspaces.
- A interface, os painéis internos e as janelas de PiP continuam no perfil padrão do aplicativo; somente o conteúdo das abas é segregado.

## Ciclo de vida

- A preferência é guardada de forma durável na tabela SQLite `app_preferences` e é carregada antes da restauração da sessão, garantindo que a restauração já ocorra com as pastas corretas.
- Ativar ou desativar a configuração reconstrói as WebViews de conteúdo; as páginas perdem o estado vivo. Isso espelha o contrato do escudo de privacidade e é anunciado pela interface.
- Mover uma aba para outro workspace descarta a WebView primeiro, para que nenhum documento continue escrevendo na pasta do workspace anterior; a próxima sincronização a recria dentro da pasta de destino.
- Excluir um workspace remove a pasta de dados dele após o encerramento dos documentos (tentativa única; arquivos travados são ignorados).

## Nomenclatura das pastas

Identificadores de workspace jamais são usados como componentes de caminho. Ids com letras ASCII, dígitos, `-` e `_` até 48 bytes são usados literalmente em minúsculas; qualquer outro id vira uma pasta `h-…` de 64 dígitos hexadecimais (SHA-256), de modo que sequências de travessia, separadores e ids não-ASCII nem escapem do diretório-base, nem colidam entre si. As pastas ficam no diretório local de dados do aplicativo, em `webview-workspaces/<nome>`.

## Custos e limites

- Cada workspace com abas visíveis mantém mais um processo de navegador do WebView2; avalie memória em dispositivos com pouca RAM antes de ativar.
- Restaurar uma sessão de workspace não restaura o histórico local da página de abas movidas entre workspaces; o histórico global, as sessões e o escudo de privacidade não são afetados.
- Um exploit de renderer dentro de um workspace ainda pode ler os cookies desse workspace. Contenção completa diante de exploit de renderer é tratada separadamente em [isolamento AppContainer](../security/app-container-isolation.md).

Os textos da interface e da documentação permanecem sincronizados; consulte `tests/security-hardening.test.mjs` para os invariantes garantidos.
