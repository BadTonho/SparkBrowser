# Proteção planejada contra sites e downloads suspeitos

Status: **planejado, não implementado**. Esta é a segunda opção discutida durante o reforço da segurança interna. A primeira entrega reforça as fronteiras do aplicativo e a identificação de atualizações do WebView2.

Uma entrega futura poderá incluir consultas de reputação contra phishing e sites maliciosos, avisos para downloads suspeitos e decisões explícitas do usuário antes de continuar. Antes de implementar, definir:

- Qual serviço de reputação ou listas locais utilizar, sua licença, cobertura, disponibilidade e custo.
- Quais URLs, hashes ou outras informações sairiam do dispositivo, políticas de retenção e consentimento.
- Como lidar com indisponibilidade, falsos positivos, contestação e exceções temporárias.
- Quais detecções exibem avisos, quais bloqueiam ações e como explicar cada decisão.
- Como atualizar, autenticar e testar resultados em cache e listas.

Nenhum serviço de reputação é consultado pela entrega atual de reforço da segurança. Manter esta proposta separada da proteção de privacidade e da verificação de checksums de arquivos já existentes.
