# 🛡️ Regras e Padrões do Projeto

> 🌍 **Idioma:** [English (Main)](../../en/contributing/rules-and-standards.md) | **Português (pt-BR)**

Para manter um projeto open-source seguro, escalável e impecável, todos os contribuidores e agentes devem seguir estas cinco regras fundamentais:

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
