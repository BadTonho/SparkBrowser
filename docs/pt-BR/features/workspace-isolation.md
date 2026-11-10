# Isolamento de conteúdo por workspace

Estado: **implementado, opt-in e desligado por padrão**. O modelo de sites/cookies apoiado nas fronteiras do aplicativo mantém um único perfil de navegação; este recurso adiciona separação opcional por workspace.

## O que ele muda

Configurações → Workspaces expõe **“Isolar conteúdo por workspace”**. Estando ativo, cada WebView de conteúdo recebe a própria pasta de dados do WebView2, derivada do identificador do workspace responsável:

- Cookies, `localStorage`/`IndexedDB` e caches HTTP tornam-se específicos do workspace; o mesmo site aberto em dois workspaces mantém duas sessões independentes.
- O runtime do WebView2 mantém um processo de navegador separado por pasta, o que também adiciona uma fronteira de processo entre workspaces.
- A interface, os painéis internos e as janelas de PiP continuam no perfil padrão do aplicativo; somente o conteúdo das abas é segregado.

## Ciclo de vida

- A preferência é guardada de forma durável na tabela SQLite `app_preferences` e é carregada antes da restauração da sessão, garantindo que a restauração já ocorra com as pastas corretas.
- Ativar ou desativar a configuração recria as WebViews de conteúdo cujo perfil resolvido muda; essas páginas perdem o estado vivo. Containers de contas e abas efêmeras mantêm seus próprios perfis e documentos. A interface anuncia a alteração.
- Mover uma aba confirma os metadados do workspace antes de reconciliar as WebViews. O documento só é recriado quando o perfil resolvido muda; containers de contas e abas efêmeras mantêm seus próprios perfis e documentos residentes ao mudar de workspace. Documentos ocultos também são verificados.
- Excluir um workspace remove a pasta de dados dele após o encerramento dos documentos (tentativa única; arquivos travados são ignorados).

## Nomenclatura das pastas

Somente letras ASCII minúsculas, dígitos, `-` e `_` até 48 bytes, sem `-` final ou nome de dispositivo do Windows, preservam o nome de pasta anterior. Os demais IDs, inclusive os que contêm maiúsculas, são integralmente transformados por SHA-256 em uma pasta `h-…` com 64 dígitos hexadecimais. Assim, IDs distintos por maiúsculas e minúsculas no banco permanecem separados no Windows e no Linux. Pastas existentes de workspaces com IDs minúsculos não mudam; IDs antigos incomuns com maiúsculas ou nomes de dispositivos recebem um perfil separado e podem exigir novo login. As pastas ficam no diretório local de dados do aplicativo em `webview-workspaces/<nome>`.

## Custos e limites

- Cada workspace com abas visíveis mantém mais um processo de navegador do WebView2; avalie memória em dispositivos com pouca RAM antes de ativar.
- Quando a movimentação muda o perfil, o documento recriado perde seu histórico local e estado vivo; o histórico global, as sessões e o escudo de privacidade não são afetados.
- Um exploit de renderer dentro de um workspace ainda pode ler os cookies desse workspace. Contenção completa diante de exploit de renderer é tratada separadamente em [isolamento AppContainer](../security/app-container-isolation.md).

Os textos da interface e da documentação permanecem sincronizados; consulte `tests/security-hardening.test.mjs` para os invariantes garantidos.
