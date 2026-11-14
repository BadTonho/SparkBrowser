# 🛡️ Regras e Padrões do Projeto

> 🌍 **Idioma:** [English (Main)](../../en/contributing/rules-and-standards.md) | **Português (pt-BR)**

Para manter um projeto open-source seguro, escalável e impecável, todos os contribuidores e agentes devem seguir estas sete regras fundamentais:

---

## Regra 1: Zero Arquivos Sensíveis no GitHub
- **Nunca subir ou commitar arquivos sensíveis** no GitHub (incluindo `.env`, chaves de API, chaves privadas, certificados, credenciais ou bancos de dados locais `.db`/`.sqlite`).
- **Verificação Obrigatória no Final do Processo:** Ao final de toda tarefa ou antes de qualquer commit/push, verificar `git status`, `git diff` e o `.gitignore` para garantir que nenhum dado sensível seja exposto.

## Regra 2: Organização Impecável do Projeto
- Manter a árvore de diretórios limpa, modular e intuitiva o tempo todo.
- Sempre criar pastas dedicadas e dividir arquivos quando necessário para que cada módulo tenha uma responsabilidade única e clara.

## Regra 3: Documentação Modular e Bem Organizada
- Nunca concentrar toda a documentação em um único arquivo gigante.
- Dividir a documentação em subpastas lógicas (`architecture/`, `features/`, `roadmap/`, `contributing/`) mantendo cada tópico conciso e atualizado.

## Regra 4: Documentação Bilíngue (Inglês Principal + Tradução em Português)
- **Inglês (`docs/en/` e `README.md` da raiz)** é o idioma principal da documentação.
- **Português (`docs/pt-BR/`)** deve ser mantido sempre sincronizado em paralelo, espelhando exatamente a mesma estrutura de pastas e arquivos de `docs/en/`.

## Regra 5: Autorização Estrita do Usuário (Nunca Fazer Nada Sem Pedido Explícito)
- **Nunca comece a programar, modificar arquivos ou executar ações sem que o usuário peça explicitamente.**
- Quando o usuário fizer uma pergunta, tirar uma dúvida ou discutir uma ideia, **apenas responda ou discuta**. Jamais edite o código antecipadamente sem que o usuário ordene explicitamente a execução da mudança.

## Regra 6: Suporte Multiplataforma (Windows e Linux)
- O aplicativo deve funcionar perfeitamente tanto no **Windows** quanto no **Linux**.
- Implementações de funcionalidades e decisões de arquitetura devem preservar a compatibilidade multiplataforma em ambos os sistemas operacionais (usando compilação condicional `#[cfg(...)]` ou abstrações de plataforma quando necessário).
- **Flexibilidade nos Testes:** Não é obrigatório executar os testes automatizados/nativos em ambos os sistemas operacionais simultaneamente em cada ciclo de desenvolvimento ou ambiente.

## Regra 7: Changelog em Toda Versão
- A versão **0.1.0** é o lançamento inicial e **não tem changelog**. Toda versão posterior deve ter um.
- Os changelogs ficam na pasta `changelog/` na raiz, com **uma subpasta por versão**, nomeada exatamente como a versão (por exemplo `changelog/0.2.0/`). Cada subpasta contém um arquivo por idioma: `en.md` (inglês) e `pt-BR.md` (português), mantidos sincronizados.
- Todo changelog é dividido em três categorias, nesta ordem: **Novas funcionalidades**, **Melhorias** e **Correções** (`New features`, `Improvements` e `Fixes` em inglês). Omita a categoria que não tiver itens.
- **Mantenha o changelog atualizado durante o trabalho:** qualquer mudança que o usuário possa perceber (comportamento novo, comportamento melhor ou correção de bug) deve adicionar um item ao changelog da versão em andamento, nos dois idiomas, na mesma alteração. Changelogs de versões já lançadas nunca são reescritos.
- O formato, as orientações de escrita e um exemplo estão no [guia de changelog](./changelog.md).
