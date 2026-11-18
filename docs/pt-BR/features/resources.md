# Painel de recursos

[English](../../en/features/resources.md)

O botão **Recursos** (um medidor) vem **desligado por padrão**. Ligue em **Configurações → Botões da barra** (a engrenagem na barra superior). Ele abre um painel sob o botão com os recursos do navegador e as ferramentas de limpeza.

## O que o painel mostra

- **Memória**: a soma do que as abas abertas usam, com quantas abas estão abertas e quantas estão hibernadas, e um **limite de memória**.
- **Processador**: a parte do processador inteiro que as páginas estão usando, e um **limite do processador**.
- **Internet**: a velocidade com que as páginas estão recebendo, e um **limite de velocidade**.
- **Limpeza**: abre a lista do que pode ser limpo.

Os números se atualizam a cada dois segundos enquanto o painel está aberto. Eles só são medidos nesse momento, porque medir a rede custa um pouco.

## Limites

Cada limite tem uma fileira de opções; **Sem** desliga. As escolhas ficam salvas neste dispositivo.

| Limite | Opções | O que faz |
| --- | --- | --- |
| **Memória** | 1, 2, 4 ou 8 GB | Quando as abas abertas usam mais que o limite, a aba que você usou há mais tempo é hibernada, uma de cada vez, até caberem. A aba ativa, abas com formulário sendo editado, mídia tocando, download ou picture-in-picture, e abas protegidas nunca são hibernadas. Uma aba hibernada volta quando você a abre. |
| **Processador** | 25, 50 ou 75% | A parte do processador inteiro que o motor web pode usar, com as páginas juntas. A interface do próprio navegador fica fora do limite, então continua responsiva; as páginas só ficam mais lentas quando o alcançam. |
| **Internet** | 1, 5, 10 ou 25 Mbps | A velocidade da rede das páginas (download e envio) e dos downloads do próprio navegador. |

### Por sistema

- **Windows**: o limite do processador é um teto rígido (um job object sobre os processos do motor, inclusive os que ele inicia depois), e o limite de internet desacelera as páginas pelo protocolo de desenvolvedor do motor. A medida da rede também vem dele.
- **Linux**: o limite do processador só **abaixa a prioridade** dos processos web, então faz diferença quando o computador está ocupado, e é desfeito ao reiniciar o navegador (voltar a prioridade exige privilégios). O motor não tem controle sobre a rede das páginas, então o limite de internet vale só para os **downloads do próprio navegador**, e a rede não é medida. O painel avisa isso. O Linux ainda não foi testado.
- Downloads iniciados por uma página (e não pela caixa de downloads do navegador) seguem o limite de internet no Windows só até onde a emulação de rede do motor os alcança.

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
