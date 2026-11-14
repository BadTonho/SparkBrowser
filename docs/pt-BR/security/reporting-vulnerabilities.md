# Política de segurança

[English](../../en/security/reporting-vulnerabilities.md)

O Spark é um navegador desktop, então relatos de segurança são levados a sério. Obrigado por ajudar a manter os usuários seguros.

## Como relatar uma vulnerabilidade

**Não abra issue, pull request ou discussão pública para um problema de segurança.** Use o relato privado de vulnerabilidades do GitHub:

1. Abra a [aba Security](https://github.com/BadTonho/SparkBrowser/security) deste repositório.
2. Escolha **Report a vulnerability**.
3. Descreva o problema da forma mais completa possível.

O relato fica privado entre você e os mantenedores até que uma correção esteja disponível.

### O que incluir

- A versão afetada (veja **Sobre** no aplicativo, ou a tag da release) e o sistema operacional (Windows ou Linux).
- Uma descrição clara do impacto: o que um atacante consegue ler, alterar ou executar.
- Passos detalhados ou uma prova de conceito mínima que reproduza o problema.
- Logs ou capturas de tela relevantes. **Remova dados pessoais, cookies, tokens e senhas antes de anexar.**

## O que esperar

- Buscamos confirmar o recebimento em até **7 dias**.
- Manteremos você informado durante a investigação e diremos se consideramos o relato válido.
- Quando a correção for publicada, daremos o crédito nas notas da release, se você quiser. Diga como prefere ser citado, ou peça para permanecer anônimo.

O Spark é mantido por voluntários, então esses prazos são metas, não garantias.

## Versões com suporte

O Spark está em desenvolvimento inicial. As correções de segurança são feitas apenas para a **release mais recente** e para a branch `main`. Reproduza o problema na release mais recente antes de relatar.

## Escopo

Dentro do escopo:

- O aplicativo Spark: o backend em Rust, a interface em Svelte, os comandos IPC e as permissões que os conectam (veja [Fronteiras de segurança do aplicativo](./application-boundaries.md)).
- O código de sincronização e backup, incluindo o armazenamento seguro de credenciais.
- A integridade de releases e atualizações: artefatos do atualizador assinados e o workflow de release.

Fora do escopo:

- Vulnerabilidades em sites que você visita ou nos motores WebView do sistema (WebView2 e WebKitGTK). Relate-as aos respectivos fabricantes.
- Vulnerabilidades em dependências de terceiros sem impacto prático no Spark. Os avisos conhecidos do upstream estão nas [notas de dependências](./dependency-notes.md). Se você encontrar uma forma de explorar um deles no Spark, relate.
- Ataques que exigem controle prévio da conta do sistema operacional do usuário ou de um dispositivo desbloqueado.
- Relatos de scanners automáticos sem impacto demonstrado.

## Pesquisa de boa-fé

Não tomaremos medidas legais contra pesquisas que sigam esta política: teste apenas na sua própria instalação e contas, evite acessar dados de outras pessoas, não degrade serviços e nos dê um prazo razoável para corrigir o problema antes de divulgá-lo.
