# Extensões (Suporte a WebExtensions do Chrome)

O Veloce Browser suporta o carregamento e execução de extensões padrão do Chrome (`manifest.json` v2/v3).

---

## Funcionalidades

* **Instalador da Chrome Web Store:** Cole qualquer URL da Chrome Web Store ou o ID de 32 caracteres da extensão para baixar, descompactar e instalar diretamente.
* **Instalador de Arquivos (.zip / .crx):** Instale extensões empacotadas diretamente a partir de arquivos `.zip` ou `.crx` locais (formatos CRX2 e CRX3).
* **Extensões Descompactadas:** Carregue pastas de extensões descompactadas diretamente do disco.
* **Instalação Persistente:** Extensões instaladas são salvas na pasta de dados do navegador e registradas no banco SQLite local, mantendo-se ativas após reinicializações.
* **Ações na Barra de Ferramentas:** Extensões que declaram ações (`action` ou `browser_action`) exibem seu ícone na barra de ferramentas e abrem seus popups ao serem clicadas.
* **Ativar / Desativar / Remover:** Controle total do ciclo de vida para ligar/desligar extensões ou desinstalá-las completamente.
* **Prontidão Multiplataforma:** Integração nativa com o WebView2 no Windows e arquitetura desacoplada e extensível para Linux (WebKitGTK).

---

## Como Instalar uma Extensão

### 1. Pela Chrome Web Store
1. Clique no ícone de **Extensões** (peça de quebra-cabeça) na barra superior.
2. No campo de texto, cole a URL da Chrome Web Store (ex: `https://chromewebstore.google.com/detail/...`) ou o ID de 32 caracteres.
3. Clique em **Instalar**. A extensão será baixada, validada, extraída e ativada automaticamente.

### 2. A Partir de Arquivo .zip ou .crx
1. Clique em **Carregar .zip / .crx** na janela de Extensões.
2. Selecione o arquivo no gerenciador de arquivos.
3. O Veloce extrairá o pacote em uma pasta gerenciada e ativará a extensão.

### 3. Carregar Descompactada
1. Clique em **Carregar Descompactada**.
2. Selecione a pasta que contém o arquivo `manifest.json`.
3. A extensão aparecerá na lista de instaladas.

---

## Arquitetura Técnica Multiplataforma

* **Windows:** Utiliza a interface nativa `ICoreWebView2Profile7` (`AddBrowserExtension`) do Microsoft Edge WebView2, que executa background workers, injeção de content scripts e suporte a APIs de WebExtensions do Chromium.
* **Linux:** Utiliza um modelo abstraído de driver com documentação arquitetural completa para WebKitGTK em [linux-extensions-guide.md](../architecture/linux-extensions-guide.md).

