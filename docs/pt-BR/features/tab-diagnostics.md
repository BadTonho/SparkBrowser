# Diagnóstico das abas

Abra **Ferramentas → Diagnóstico das abas** ou pesquise pela função na Command Bar. Filtre pelo workspace atual ou todos e pesquise títulos e URLs. Os totais descrevem o escopo de workspaces escolhido; o limite configurado de residentes continua global e flexível.

Cada linha informa a residência real da WebView, tempo desde a ativação, workspace e todos os motivos de proteção: seleção, divisão, fixação, carregamento, formulário alterado, mídia e downloads. Downloads nativos pausados continuam protegendo a WebView da qual dependem. Os motivos usam a mesma política do backend da hibernação e do fechamento em lote revisado.

Selecione uma página ou hiberne uma residente sem proteção. O backend verifica novamente a proteção ao executar a ação. Os dados atualizam a cada dois segundos apenas enquanto o painel está montado; respostas obsoletas são descartadas. O preview informa que diagnósticos nativos exigem o aplicativo desktop. A contagem de WebViews não mede memória dos processos.

Validação: `node tools/testing/native-smoke.mjs --case tab-diagnostics` com a build nativa isolada.
