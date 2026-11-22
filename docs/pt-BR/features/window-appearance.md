# 🪟 Customização do Fundo das Janelas e Superfícies

> 🌍 **Idioma:** [English](../../en/features/window-appearance.md) | **Português (pt-BR)**

O Spark Browser oferece controle abrangente sobre a superfície visual e transparência das janelas flutuantes, diálogos, popovers e gavetas (como **Configurações do Navegador**, **Configurações Rápidas**, **Personalizar Nova Guia**, **Gerenciador de Extensões** e as **Gavetas de Histórico e Downloads**).

---

## 1. Estilos Visuais

Os usuários podem escolher entre três estilos de superfície em **Configurações → Aparência e disposição**:

### ⬛ Fosco (Sólido)
- Oferece uma superfície completamente opaca e uniforme (`#18181b` no modo escuro; `#ffffff` no modo claro).
- Desativa o desfoque de fundo para usuários que preferem superfícies planas e de alto contraste sem transparência.

### 💧 Vidro
- Janelas flutuantes e menus usam uma base preta translúcida no tema escuro e branca translúcida no tema claro. Uma borda fina e uniforme define o contorno.
- As Configurações em aba usam um fundo liso, preto ou branco, de acordo com o tema. A paleta do Spark continua colorindo os controles e as seleções.
- As superfícies flutuantes usam um desfoque mais leve que o do Gelo.
- **Ajustável:** Acompanha os controles de opacidade e desfoque abaixo.

### ❄️ Gelo
Um desfoque forte, próximo do visual fosco do desktop COSMIC:
- **Tonalidade Translúcida:** Utiliza uma superfície base semitransparente (`rgba(22, 22, 28, var(--modal-surface-opacity))` no modo escuro; `rgba(255, 255, 255, var(--modal-surface-opacity))` no modo claro).
- **Desfoque Profundo:** Aplica `backdrop-filter: blur(var(--modal-surface-blur)) saturate(160%)` acelerado por hardware, que difunde o conteúdo atrás da janela (páginas da web, papel de parede ou janelas do desktop).
- **Elevação Suave:** Uma borda sutil e sombras ambientes profundas garantem a legibilidade.

O valor salvo do Gelo continua sendo `frosted`, então a escolha feita em versões anteriores é mantida.

---

## 2. Controles e Configuração

As Configurações rápidas têm contornos mais marcados e rótulos com maior contraste nos dois temas. As opções selecionadas usam a cor de destaque da paleta atual.

Todos os controles estão disponíveis na seção **Aparência e disposição** nas Configurações:

1. **Estilo do fundo das janelas (`settings.window_surface_style`)**:
   - Alternância rápida entre **Fosco**, **Vidro** e **Gelo** (Fosco vem ativado por padrão).
2. **Opacidade do fundo das janelas (`settings.window_surface_opacity`)**:
   - Controle de **30%** a **100%** que ajusta a intensidade da camada de cor. O Gelo fica opaco em 100%, enquanto o Vidro mantém o acabamento translúcido.
   - Valor padrão: **70%**.
3. **Desfoque das janelas (`settings.window_surface_blur`)**:
   - Ajusta o raio de difusão do desfoque de **0 px** a **32 px** (apenas Vidro e Gelo).
   - Valor padrão: **20 px**.
4. **Desfoque atrás de diálogos (`settings.background_blur_c21c`)**:
   - Controla o desfoque separado aplicado ao plano de fundo da tela inteira quando um diálogo modal está aberto (0 a 24 px).

---

## 3. Persistência e Compatibilidade Multiplataforma

- **Persistência:** Os valores são armazenados no `safeStorage` sob as chaves `spark_window_surface_style`, `spark_window_surface_opacity` e `spark_window_surface_blur_px`, sendo mantidos entre reinicializações do navegador.
- **Multiplataforma:** O Vidro usa CSS para a superfície translúcida e o desfoque. A parte do fundo que uma WebView consegue desfocar depende do motor de renderização e das superfícies que ele compõe juntas.

