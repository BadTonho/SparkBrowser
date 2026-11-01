# Visualização dividida

Em **Tools → Tabs**, selecione duas abas diferentes do workspace atual e escolha **Split selected tabs**. As páginas aparecem lado a lado com um divisor de 8 px. Arraste o divisor ou dê foco a ele e use Esquerda/Direita; Home restaura 50%. As proporções vão de 20% a 80%, com cada view nativa tendo pelo menos 100 pixels lógicos de largura.

Clicar em uma página dá foco à aba e aos controles de navegação. Selecionar outra aba substitui o painel focado. Sair da divisão mantém a aba focada. Fechar ou mover uma aba do par desfaz a divisão. Cada workspace lembra seu par, proporção e aba focada entre reinícios; workspaces inativos podem hibernar suas páginas. O par atual é protegido contra hibernação manual/automática e fechamento em lote, mesmo com limite de residentes igual a 1.

Overlays ocultam ambas as views e as restauram depois. Atualizações de geometria reutilizam WebViews residentes sem navegar ou mudar o foco; movimentos do ponteiro são agrupados e a proporção persiste ao terminar o ajuste. Windows usa eventos nativos de foco; outros motores usam a bridge de foco da página com verificação de geração e exigem validação por plataforma.
