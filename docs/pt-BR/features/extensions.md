# Extensões (Suporte a WebExtensions do Chrome)

No Windows, o Spark Browser pode carregar extensões do Chrome pelo WebView2; a compatibilidade depende do mecanismo e da extensão. No Linux, a instalação e o gerenciamento ainda não estão disponíveis e retornam um erro.

---

## Funcionalidades

* **Instalação em 1-Clique na Chrome Web Store:** Ao navegar em qualquer extensão na `chromewebstore.google.com`, um botão interativo **`[🧩 Instalar no Spark]`** aparece automaticamente no Omnibox. Um clique baixa, valida e ativa a extensão na hora.
* **Interceptador de Download de CRX:** O clique no botão "Usar no Chrome" da Google é interceptado nativamente antes da barreira de downloads do WebView2, direcionando o pacote diretamente para o instalador em segundo plano do Spark.
* **Instalador da Chrome Web Store (Manual):** Cole qualquer URL da Chrome Web Store ou o ID de 32 caracteres da extensão no modal para baixar, descompactar e instalar diretamente.
* **Instalador de Arquivos (.zip / .crx):** Instale extensões empacotadas diretamente a partir de arquivos `.zip` ou `.crx` locais (formato CRX3).
* **Só pacotes assinados:** Todo `.crx` precisa ser um pacote CRX3 com assinaturas válidas e cujo ID corresponda à chave do desenvolvedor, a mesma verificação que o Chromium faz. Pacotes da Chrome Web Store também precisam ser a extensão pedida, e a cópia instalada só é substituída depois que o novo pacote passa na verificação. Arquivos CRX2 antigos são recusados. Um `.zip` não tem assinatura e é tratado como uma pasta descompactada que você escolheu.
* **Extensões Descompactadas:** Carregue pastas de extensões descompactadas diretamente do disco.
* **Instalação Persistente:** Extensões instaladas são salvas na pasta de dados do navegador e registradas no banco SQLite local, mantendo-se ativas após reinicializações.
* **Ações na Barra de Ferramentas:** Extensões que declaram ações (`action` ou `browser_action`) exibem seu ícone na barra de ferramentas e abrem seus popups ao serem clicadas.
* **Ativar / Desativar / Remover:** Controle total do ciclo de vida para ligar/desligar extensões ou desinstalá-las completamente.
* **Disponibilidade por plataforma:** A integração nativa usa WebView2 no Windows. O suporte a Linux ainda está planejado.

---

## Como Instalar uma Extensão

### 1. Em 1-Clique pela Chrome Web Store (Recomendado)
1. Navegue até a página de qualquer extensão na Chrome Web Store (ex: `https://chromewebstore.google.com/detail/...`).
2. O botão **`[🧩 Instalar no Spark]`** aparecerá diretamente dentro da barra de endereços (Omnibox).
3. Dê 1 clique: o Spark baixa e ativa a extensão imediatamente, e uma notificação toast confirma a instalação.

### 2. Pela Janela de Extensões
1. Clique no ícone de **Extensões** (peça de quebra-cabeça) na barra superior.
2. No campo de texto, cole a URL da Chrome Web Store ou o ID de 32 caracteres.
3. Clique em **Instalar**. A extensão será baixada, validada, extraída e ativada automaticamente.

### 3. A Partir de Arquivo .zip ou .crx
1. Clique em **Carregar .zip / .crx** na janela de Extensões.
2. Selecione o arquivo no gerenciador de arquivos.
3. O Spark extrairá o pacote em uma pasta gerenciada e ativará a extensão.

### 4. Carregar Descompactada
1. Clique em **Carregar Descompactada**.
2. Selecione a pasta que contém o arquivo `manifest.json`.
3. A extensão aparecerá na lista de instaladas.

---

## Arquitetura Técnica Multiplataforma

* **Windows:** Utiliza a interface nativa `ICoreWebView2Profile7` (`AddBrowserExtension`) do Microsoft Edge WebView2, que executa background workers, injeção de content scripts e suporte a APIs de WebExtensions do Chromium.
* **Linux:** A instalação e o gerenciamento de extensões não estão disponíveis. A proposta para WebKitGTK está descrita em [linux-extensions-guide.md](../architecture/linux-extensions-guide.md).


Importações de arquivo são limitadas a 64 MiB compactados, 32 MiB por entrada, 128 MiB extraídos no total e 2.048 entradas.
