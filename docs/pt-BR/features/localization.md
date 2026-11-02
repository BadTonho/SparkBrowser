# Idiomas da interface

Configurações → Aparência e disposição → Idioma da interface oferece Sistema, Português e English. Sistema usa português quando o primeiro idioma preferido do sistema começa com `pt`, e inglês nos demais casos. Uma escolha explícita salva tem prioridade. As alterações são imediatas, incluindo `lang` do documento, controles, barra de comandos, datas, números, plurais e rótulos dos overlays nativos.

Conteúdo das páginas, títulos, endereços, notas, notas de versão e nomes definidos pelo usuário são preservados. Diálogos do sistema operacional seguem o idioma do sistema. A preferência de idioma permanece local ao perfil do navegador.

Os catálogos de referência em inglês e as traduções em português ficam em `src/lib/i18n/catalogs/`, divididos por domínio da interface. `MessageKey` é inferido das chaves em inglês. Os testes Node exigem arquivos de domínio, chaves e parâmetros correspondentes; a verificação Svelte valida as referências. Adicione a tradução nos dois catálogos ao introduzir textos na interface. Use parâmetros explícitos nas mensagens em vez de traduzir valores do usuário.

Erros usam códigos estáveis independentes do texto de diagnóstico. Falhas da bridge nativa carregam o código do comando; validações locais podem fornecer um código `UserError` específico. Mensagens ao usuário usam o catálogo; detalhes técnicos expansíveis mantêm o diagnóstico original. Não traduza erros comparando textos em inglês.

Execute `npm test`, `npm run check` e `npm run test:native -- --case localization`. O cenário nativo troca os idiomas pelo controle real das configurações, verifica rótulos de Ferramentas, reinicia com o mesmo perfil isolado e confirma que as abas mantêm suas identidades e endereços.
