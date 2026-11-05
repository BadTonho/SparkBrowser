# Extensões (Suporte a WebExtensions do Chrome)

O Veloce Browser suporta o carregamento e execução de extensões padrão do Chrome (`manifest.json` v2/v3).

---

## Funcionalidades

* **Extensões Descompactadas:** Carregue pastas de extensões descompactadas diretamente do disco ou de arquivos `.zip`/`.crx` extraídos.
* **Instalação Persistente:** Extensões instaladas são salvas na pasta de dados do navegador e registradas no banco SQLite local, mantendo-se ativas após reinicializações.
* **Ações na Barra de Ferramentas:** Extensões que declaram ações (`action` ou `browser_action`) exibem seu ícone na barra de ferramentas e abrem seus popups ao serem clicadas.
* **Ativar / Desativar / Remover:** Controle total do ciclo de vida para ligar/desligar extensões ou desinstalá-las completamente.
* **Prontidão Multiplataforma:** Integração nativa com o WebView2 no Windows e arquitetura desacoplada e extensível para Linux (WebKitGTK).

---

## Como Instalar uma Extensão

1. Clique no ícone de **Extensões** (peça de quebra-cabeça) na barra de ferramentas superior ou abra **Configurações -> Extensões**.
2. Clique em **Carregar Descompactada**.
3. Selecione a pasta que contém o arquivo `manifest.json` da extensão.
4. A extensão aparecerá na lista de instaladas e seu ícone de ação ficará fixado na barra de ferramentas.

---

## Arquitetura Técnica Multiplataforma

* **Windows:** Utiliza a interface nativa `ICoreWebView2Profile7` (`AddBrowserExtension`) do Microsoft Edge WebView2, que executa background workers, injeção de content scripts e suporte a APIs de WebExtensions do Chromium.
* **Linux:** Utiliza um modelo abstraído de driver com documentação arquitetural completa para WebKitGTK em [linux-extensions-guide.md](../architecture/linux-extensions-guide.md).

