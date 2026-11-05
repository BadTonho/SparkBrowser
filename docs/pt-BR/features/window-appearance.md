# 🪟 Customização do Fundo das Janelas e Superfícies

> 🌍 **Idioma:** [English](../../en/features/window-appearance.md) | **Português (pt-BR)**

O Veloce Browser oferece controle abrangente sobre a superfície visual e transparência das janelas flutuantes, diálogos, popovers e gavetas (como **Configurações do Navegador**, **Configurações Rápidas**, **Personalizar Nova Guia**, **Gerenciador de Extensões** e as **Gavetas de Histórico e Downloads**).

---

## 1. Estilos Visuais

Os usuários podem escolher entre dois estilos principais em **Configurações → Aparência e disposição**:

### 🪟 Vidro Fosco (Efeito Geleia / Blur)
Estética moderna de vidro translúcido (glassmorphism):
- **Tonalidade Translúcida:** Utiliza uma superfície base semitransparente (`rgba(22, 22, 28, var(--modal-surface-opacity))` no modo escuro; `rgba(255, 255, 255, var(--modal-surface-opacity))` no modo claro).
- **Desfoque Fosco Profundo:** Aplica `backdrop-filter: blur(var(--modal-surface-blur)) saturate(160%)` acelerado por hardware que difunde suavemente o conteúdo que está atrás da janela (páginas da web, papel de parede ou janelas do desktop).
- **Borda Especular e Elevação Suave:** Bordas sutis com leve brilho e sombras ambientes profundas que garantem contraste e legibilidade impecáveis.

### ⬛ Preto Fosco (Sólido)
- Oferece uma superfície completamente opaca e uniforme (`#18181b` no modo escuro; `#ffffff` no modo claro).
- Desativa o desfoque de fundo para usuários que preferem superfícies planas e de alto contraste sem transparência.

---

## 2. Controles e Configuração

Todos os controles estão disponíveis na seção **Aparência e disposição** nas Configurações:

1. **Estilo do fundo das janelas (`settings.window_surface_style`)**:
   - Alternância rápida entre **Vidro Fosco** e **Preto Fosco** (Preto Fosco vem ativado por padrão).
2. **Opacidade do fundo das janelas (`settings.window_surface_opacity`)**:
   - Controle deslizante (slider) de **30%** (estética de gelatina vívida e translúcida) a **100%** (opaco).
   - Valor padrão: **70%**.
3. **Desfoque do vidro fosco (`settings.window_surface_blur`)**:
   - Ajusta o raio de difusão do desfoque de **0 px** a **32 px**.
   - Valor padrão: **20 px**.
4. **Desfoque atrás de diálogos (`settings.background_blur_c21c`)**:
   - Controla o desfoque separado aplicado ao plano de fundo da tela inteira quando um diálogo modal está aberto (0 a 24 px).

---

## 3. Persistência e Compatibilidade Multiplataforma

- **Persistência:** Os valores são armazenados no `safeStorage` sob as chaves `veloce_window_surface_style`, `veloce_window_surface_opacity` e `veloce_window_surface_blur_px`, sendo mantidos entre reinicializações do navegador.
- **Multiplataforma:** Desenvolvido com variáveis CSS e `backdrop-filter` nativo acelerado por hardware, garantindo desempenho fluido e sem travamentos tanto no **Windows** (WebView2) quanto no **Linux** (WebKitGTK e compositores modernos Wayland/X11).

