# Isolamento AppContainer (planejado)

Estado: **planejado, não implementado**. Identificado como a ideia de proteção de maior risco durante a revisão da camada de isolamento; a decisão atual é documentar o desenho primeiro e colocá-lo no roadmap, em vez de entregar um launcher experimental.

## Objetivo

Executar todo o host do navegador dentro de um AppContainer do Windows, o mesmo modelo de contenção de nível de sistema usado pelo Edge e pelo Chrome nos próprios processos. Um renderer malicioso ficaria contido duas vezes: pelo sandbox de renderer do WebView2 e pelo token de processo do host.

## Arquitetura proposta

1. Um pequeno launcher cria um token de segurança AppContainer (integridade não confiável, um SID de perfil mais as capacidades de Internet e mídia de que o WebView2 precisa) e depois inicie o executável real do navegador com esse token, descartando os próprios privilégios.
2. O launcher pré-instala entradas ACL para o SID do AppContainer no diretório de perfil e no diretório de downloads, porque processos AppContainer não podem ler caminhos arbitrários do usuário.
3. O atualizador ou reinicia pelo launcher (para que a contenção não desapareça silenciosamente após a atualização) ou reaproveita o token antes do reinício; ambos os caminhos precisam ser validados.
4. Desligado por padrão atrás de uma opção nas configurações e de um teste de fumaça nativo, para que instalações comuns mantenham exatamente o comportamento atual.

## Conflitos conhecidos que precisam de solução primeiro

- **Atualizador**: o `tauri-plugin-updater` reabre o executável atual diretamente; esse caminho precisa passar pelo launcher ou a contenção some após a primeira atualização.
- **Acesso a arquivos**: SQLite, downloads, destinos de arrastar-e-soltar e o estado do runtime do WebView2 exigem ACLs explícitas; falhas aparecem como erros de permissão difíceis depurar.
- **Fluxos auxiliares**: registro de handler de protocolo, integração com o shell e assinaturas do atualizador (ver `src-tauri/tests/updater_signatures.rs`) precisam ser revalidados sob o token restrito.
- **Depuração**: ferramentas de desenvolvedor da interface e o harness de testes nativos precisam continuar utilizáveis em builds de desenvolvimento, onde a opção fica desligada.

## Critérios de aceitação antes de implementar

- A suíte de testes nativos (`npm run test:native`) aprovada duas vezes seguidas com o isolamento ativo: início a frio, reinício de atualização, download e salvamento, arrastar-e-soltar, isolamento de workspaces ligado e desligado.
- Encerrar o launcher, encerrar processos do WebView2 e alternar a opção no meio da sessão nunca deixam o navegador executando fora do container.
- O recurso tem opt-in explícito, uma explicação no produto dos seus custos e uma reversão de um comando.

Até lá, nenhum launcher é entregue. As fronteiras de aplicativo já mantêm as WebViews de conteúdo sem comandos do aplicativo, sem objetos nativos e sem superfície persistente entre abas; o AppContainer blindaria a última camada entre um processo de renderer comprometido e o sistema operacional.
