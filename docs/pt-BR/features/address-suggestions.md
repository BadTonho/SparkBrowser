# Sugestões da barra de endereço

Clicar na barra de endereço, ou digitar nela, mostra uma lista de sugestões logo abaixo.

## O que a lista mostra

- **Ao focar (nada digitado):** os sites visitados mais recentemente, uma linha por endereço, para voltar com um clique.
- **Ao digitar:** a primeira linha é sempre o que o **Enter** faz com o texto como foi digitado: *Pesquisar com* o seu mecanismo padrão para palavras, ou *Ir para o endereço* quando o texto parece um (`example.com`, `localhost:1420`, `https://…`). Abaixo vêm os seus **favoritos** que combinam e depois as **páginas visitadas** do histórico que combinam, as melhores primeiro.
- Uma página aparece uma só vez, qualquer que seja o esquema, o `www.` ou a barra final. Cada palavra digitada precisa aparecer no título ou no endereço.
- Páginas que começam com o que você digitou e páginas que você visita muito aparecem primeiro; visitas recentes pesam um pouco mais.
- **Sugestões de pesquisa do mecanismo de busca** (opcionais, veja abaixo) entram por último, até quatro.

## Campo de pesquisa da página de nova aba

O campo no meio da [página de nova aba](new-tab.md) mostra a mesma lista: as mesmas linhas, a mesma ordem, as mesmas teclas e a mesma opção **Sugestões de pesquisa**. A diferença está em quando ela abre: ao clicar no campo, digitar nele ou apertar a seta para baixo. O foco automático de uma nova aba não abre a lista, para a página aparecer limpa. A lista fica por cima dos cartões de sites recentes, sem movê-los.

A página é isolada do restante do aplicativo, então pede os dados ao lado nativo pela ponte autenticada da nova aba (`/suggest` para histórico e favoritos, `/terms` para os termos do mecanismo, recusado em janela privada). A ordenação é o código da barra de endereço: `public/new-tab/suggestions-core.js` é gerado de `src/lib/services/address-suggestions.js` com `npm run build:suggestions`, e um teste falha quando ele fica desatualizado.

## Como usar

| Tecla / ação | Resultado |
| --- | --- |
| Seta para cima / baixo | Move o destaque; passando de uma ponta, volta a "nada destacado" |
| Enter | Abre a linha destacada, ou faz o que o texto digitado pede quando nenhuma está destacada |
| Esc | Fecha a lista e restaura a barra de endereço |
| Clique numa linha | Abre; a barra de endereço mantém o foco até a página começar a carregar |

## Como ela é desenhada

A lista é um popover sobre a página, logo abaixo da barra de endereço e com a mesma largura. A página não é redimensionada nem movida, e a barra de endereço e a barra de ferramentas continuam clicáveis.

- **A digitação continua na barra de endereço.** O popover nunca toma o teclado; as setas, o Enter e o Esc são tratados pela barra de endereço. Clicar numa linha leva o foco ao popover só para aquele clique.
- **Clicar em qualquer outro lugar da página fecha a lista**, assim como o Esc, mover o foco para outra parte da interface, abrir outro painel ou trocar de janela.
- Ele é criado com antecedência quando o ponteiro chega à barra de endereço, para a primeira lista não esperar por isso.
- Fora do aplicativo (a interface vista num navegador comum) não há camada nativa, então a lista fica no layout da página e empurra a página para baixo.

## Sugestões de pesquisa do mecanismo de busca

**Configurações → Busca e navegação → Sugestões de pesquisa** acrescenta os termos que o seu mecanismo de busca propõe para o que você digita. Vem **ligado por padrão** e pode ser desligado ali. É o único ponto em que um texto sai do computador antes de você apertar Enter, por isso a configuração diz o que faz.

- Funciona com **Google, DuckDuckGo, Bing e Brave Search**. Os outros mecanismos não têm serviço público de sugestões, então nada é pedido para eles e a página de configurações avisa.
- Só o texto digitado na barra de endereço é enviado, com o nome genérico de cliente `Spark`; sem cookies, sem conta, sem seguir redirecionamentos, e o pedido desiste após 3 segundos.
- **Nunca é usado em janela privada**, nem quando o texto parece um endereço (`example.com`), nem acima de 100 caracteres ou com caracteres de controle.
- O pedido é feito após uma pequena pausa na digitação (150 ms), não a cada tecla, e o mesmo texto reaproveita a resposta anterior pelo resto daquela sessão de foco.
- Se o pedido falhar, a lista simplesmente fica sem os termos do mecanismo; nada é mostrado ao usuário.
- Os pedidos são feitos pelo lado nativo, não pela página: o aplicativo continua bloqueando conexões diretas da sua interface.

## Observações

- Com a opção desligada, tudo vem do histórico e dos favoritos locais e nada é enviado para fora. Veja as [configurações de Privacidade e segurança](privacy-security-settings.md) para a retenção do histórico.
- O histórico é pesquisado com a mesma busca do painel de Histórico, mas agrupado por endereço: as 2.000 visitas mais recentes que combinam viram uma linha por endereço (a visita mais recente e quantas daquelas visitas ele teve), e os 60 endereços mais recentes são ordenados. Um site que você visita muito não enche mais a lista, e o número de visitas entra na ordem.
