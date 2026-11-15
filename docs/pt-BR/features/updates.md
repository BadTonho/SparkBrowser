# Canais de versão e atualizações

Abra **Settings → About & Updates** para consultar a versão em execução, escolher **Stable** ou **Beta**, verificar atualizações e ler as notas. Instalações de release Windows x64 consultam uma vez ao abrir. Falhas na inicialização aparecem nesse painel sem interromper a navegação. Builds nativos de desenvolvimento e teste isolados e outras plataformas podem salvar a preferência de canal, mas não podem verificar nem instalar atualizações. A prévia no navegador não pode salvar a preferência.

Stable aceita tags publicadas como `v1.0.0`. Beta também aceita `v1.1.0-beta.1`, marcada como pré-lançamento no GitHub, e versões estáveis mais novas. A ordenação SemVer coloca `beta.10` depois de `beta.2`, e `1.1.0` depois de `1.1.0-beta.10`. Rascunhos, tags não suportadas e marcações de pré-lançamento inconsistentes são excluídos. Erros de paginação e limites da API são informados em vez de indicar que a instalação está atualizada.

Os canais compartilham a instalação e os dados locais. A primeira execução adota o canal da versão instalada; depois, a preferência é preservada. Trocar de uma beta mais nova para Stable aguarda uma estável superior e nunca rebaixa o aplicativo nem o banco. Não há sincronização em nuvem nem login no GitHub.

Uma atualização disponível exibe um aviso que pode ser adiado. **Download update** baixa e verifica sem instalar. **Install and restart** exige confirmação. Formulários alterados e mídia em reprodução exigem confirmação adicional; downloads ativos ou na fila precisam terminar ou ser cancelados. Notas pendentes precisam salvar com sucesso, gravações de navegação são concluídas e a sessão é persistida antes de instalar. Falhas preservam a instalação em execução e permitem tentar novamente. Formulários, rolagem e mídia são perdidos ao reiniciar.

O Rust consulta releases públicas de `BadTonho/SparkBrowser`, valida a tag, o manifesto e a URL do instalador e usa o atualizador oficial do Tauri. Os pacotes precisam de assinatura válida vinculada à versão anunciada. Os bytes do instalador ficam em memória até instalar, mudar de canal ou sair; reiniciar exige baixar novamente. Somente a WebView local `main` pode executar comandos de atualização do aplicativo e receber seus eventos. Não são concedidas permissões IPC diretas do plugin updater.

Consulte [releases assinadas localmente](../contributing/releases.md). Versões sem esse atualizador precisam de uma instalação manual inicial.
