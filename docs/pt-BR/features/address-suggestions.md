# Sugestões da barra de endereço

Clicar na barra de endereço, ou digitar nela, mostra uma lista de sugestões logo abaixo.

## O que a lista mostra

- **Ao focar (nada digitado):** os sites visitados mais recentemente, uma linha por endereço, para voltar com um clique.
- **Ao digitar:** a primeira linha é sempre o que o **Enter** faz com o texto como foi digitado: *Pesquisar com* o seu mecanismo padrão para palavras, ou *Ir para o endereço* quando o texto parece um (`example.com`, `localhost:1420`, `https://…`). Abaixo vêm os seus **favoritos** que combinam e depois as **páginas visitadas** do histórico que combinam, as melhores primeiro.
- Uma página aparece uma só vez, qualquer que seja o esquema, o `www.` ou a barra final. Cada palavra digitada precisa aparecer no título ou no endereço.
- Páginas que começam com o que você digitou e páginas que você visita muito aparecem primeiro; visitas recentes pesam um pouco mais.
- **Sugestões de pesquisa do mecanismo de busca** (opcionais, veja abaixo) entram por último, até quatro.

## Como usar

| Tecla / ação | Resultado |
| --- | --- |
| Seta para cima / baixo | Move o destaque; passando de uma ponta, volta a "nada destacado" |
| Enter | Abre a linha destacada, ou faz o que o texto digitado pede quando nenhuma está destacada |
| Esc | Fecha a lista e restaura a barra de endereço |
| Clique numa linha | Abre; a barra de endereço mantém o foco até a página começar a carregar |

A lista se alinha à barra de endereço. Ela fica no layout da página, então a página abaixo desce enquanto a lista está aberta; a lista só cresce durante uma sessão, para a página não pular a cada tecla.

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
- O histórico é pesquisado com a mesma consulta do painel de Histórico, nas 60 visitas mais recentes que combinam.
