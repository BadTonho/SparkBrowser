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

Cada limite é uma barra que você arrasta até o valor desejado, com o valor escolhido centralizado acima dela. Você também pode **clicar no número e digitar um valor exato** (memória em GB, processador em % e internet em Mbps) e apertar Enter. Os valores digitados ficam dentro do que o limite permite: memória de 0,25 a 128 GB, processador de 1 a 95%, internet de 0,1 a 1000 Mbps. Arraste a barra até o fim, à direita, ou apague o número, para **Sem limite**. O limite é aplicado quando você solta a barra ou confirma o número e fica salvo neste dispositivo.

| Limite | Faixa | O que faz |
| --- | --- | --- |
| **Memória** | 512 MB a 16 GB, de 512 em 512 MB | A aba em que você está nunca é mexida. Quando as abas abertas usam mais que o limite, as outras são hibernadas, as que você usou há mais tempo primeiro, quantas forem necessárias para caber (verificado a cada poucos segundos e na hora, quando você muda o limite ou troca de aba). Abas com formulário sendo editado, mídia tocando, download ou picture-in-picture, e abas fixadas ou protegidas não são hibernadas; enquanto estão em segundo plano, elas são orientadas a usar o mínimo de memória que o motor consegue devolver (só no Windows) e voltam ao normal quando você as abre. Uma aba hibernada volta quando você a abre. Se só a aba em que você está já usa mais que o limite, não há mais o que fazer: o limite nunca fecha nem deixa lenta a aba que você está usando. |
| **Processador** | 10 a 90%, de 5 em 5 | A parte do processador inteiro que o motor web pode usar, com as páginas juntas. A interface do próprio navegador fica fora do limite, então continua responsiva; as páginas só ficam mais lentas quando o alcançam. |
| **Internet** | 1 a 1000 Mbps: de 1 em 1 até 10, de 5 em 5 até 50, de 10 em 10 até 100, depois de 100 em 100 | A velocidade da rede das páginas (download e envio) e dos downloads do próprio navegador. |

### Por sistema

- **Windows**: o limite do processador é um teto rígido (um job object sobre os processos do motor, inclusive os que ele inicia depois), e o limite de internet desacelera as páginas pelo protocolo de desenvolvedor do motor. A medida da rede também vem dele.
- **Linux**: o limite do processador só **abaixa a prioridade** dos processos web, então faz diferença quando o computador está ocupado, e é desfeito ao reiniciar o navegador (voltar a prioridade exige privilégios). O motor não tem controle sobre a rede das páginas, então o limite de internet vale só para os **downloads do próprio navegador**, e a rede não é medida. O Linux ainda não foi testado.
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

### Quanto será limpo

O topo da página de limpeza mostra **quanto espaço os itens marcados ocupam** (e, portanto, quanto será limpo), e cada item mostra o seu tamanho. É medido no disco, não chutado: somam-se as pastas de cache e de dados de sites do motor, e o histórico e a lista de downloads têm o tamanho dos seus registros (os arquivos baixados não são tocados, então a lista é pequena). Os números seguem o período escolhido, contando os arquivos alterados pela última vez antes dele, e são medidos de novo quando você muda as escolhas e depois de uma limpeza. Uma pasta muito grande pode ser medida só em parte, e aí aparece "pelo menos". Onde um item não pode ser medido no período escolhido (cache e cookies por idade no Linux), aparece "—" e a página avisa.

### Limpar por idade

Em **Limpar o que tem mais de**, escolha **Tudo**, **5**, **10**, **20**, **30** ou **90 dias** para limpar só o que é mais antigo que isso e manter o mais novo. Vale para cada item marcado:

- **Histórico**: as visitas (e as abas fechadas recentemente) de antes disso.
- **Lista de downloads**: os downloads concluídos registrados antes disso. Um download concluído sem data conhecida é mantido.
- **Cache e cookies/dados dos sites**: o que o motor guardou antes disso. **Só no Windows**: no Linux o motor só consegue limpar o que mudou nos últimos dias, o oposto, então com uma idade escolhida esses dois avisam que não está disponível (escolha **Tudo** para limpá-los).

A página de limpeza tem o mesmo tamanho da página principal, então o painel não muda de tamanho ao ir de uma para a outra.

## Bom saber

- O cache e os dados dos sites são limpos em cada armazenamento de dados alcançável por uma página aberta. Workspaces com dados isolados têm um armazenamento próprio; o de um workspace cujas abas estão todas hibernadas só é alcançado quando uma delas acorda.
- As configurações do próprio navegador (tema, paleta de cores, botões da barra) ficam no mesmo armazenamento dos sites quando os dados são compartilhados. Elas são guardadas antes de limpar os dados dos sites e escritas de volta depois, então limpar dados dos sites não zera o navegador.
- O Windows usa o perfil do WebView2 (`ClearBrowsingData`) e o Linux usa o gerenciador de dados do WebKitGTK. O Linux ainda não foi testado.
