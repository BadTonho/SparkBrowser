# Painel de recursos

[English](../../en/features/resources.md)

O botão **Recursos** (um medidor) vem **desligado por padrão**. Ligue em **Configurações → Botões da barra** (a engrenagem na barra superior). Ele abre um painel sob o botão com os recursos do navegador e as ferramentas de limpeza.

## O que o painel mostra

- **Memória em uso**: a soma do que as abas abertas usam, com quantas abas estão abertas e quantas estão hibernadas. Atualiza enquanto o painel está aberto.
- **Limpeza**: abre a lista do que pode ser limpo.

Os limites de memória, processador e internet são a próxima etapa e ainda não estão nesta versão do painel.

## Limpeza

Escolha o que limpar e aperte **Limpar selecionados**; o botão pede um segundo clique para confirmar, porque nada disso pode ser desfeito.

| Item | O que limpa |
| --- | --- |
| **Cache** | Cópias salvas de páginas e arquivos. Os sites podem carregar um pouco mais devagar uma vez. |
| **Cookies e dados dos sites** | Cookies, armazenamento local, bancos de dados e service workers que os sites guardam. Você sai das contas dos sites em que estiver logado. |
| **Histórico de navegação** | O histórico de páginas visitadas do navegador. |
| **Lista de downloads** | Os downloads concluídos saem da lista; os arquivos continuam no disco. |

Só o cache vem marcado ao abrir o painel. O resultado aparece no painel: o que foi limpo, ou o que não deu para limpar.

## Bom saber

- O cache e os dados dos sites são limpos em cada armazenamento de dados alcançável por uma página aberta. Workspaces com dados isolados têm um armazenamento próprio; o de um workspace cujas abas estão todas hibernadas só é alcançado quando uma delas acorda.
- As configurações do próprio navegador (tema, paleta de cores, botões da barra) ficam no mesmo armazenamento dos sites quando os dados são compartilhados. Elas são guardadas antes de limpar os dados dos sites e escritas de volta depois, então limpar dados dos sites não zera o navegador.
- O Windows usa o perfil do WebView2 (`ClearBrowsingData`) e o Linux usa o gerenciador de dados do WebKitGTK. O Linux ainda não foi testado.
