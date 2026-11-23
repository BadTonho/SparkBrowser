/** Labels of the background editor page. The page cannot read the browser's catalogs, so it has its own. */
const en = {
  title: "Custom background", leftPanel: "Looks", rightPanel: "Settings",
  colors: "Colors", randomize: "Randomize", fromImage: "From image", activeColors: "Active colors",
  gradientMap: "Gradient map", reset: "Reset", blend: "Blend", stops: "Stops", dragToReposition: "Drag to reposition", stop: "Stop",
  bias: "Bias", sharpness: "Sharpness",
  shape: "Shape", contrast: "Contrast",
  motion: "Motion",
  grade: "Grade", brightness: "Brightness", saturation: "Saturation",
  effects: "Effects", grain: "Grain", classic: "Classic", film: "Film", amount: "Amount", size: "Size", color: "Color",
  midtone: "Mid-tone focus", seed: "Seed", animateGrain: "Animate grain (video)",
  tone: "Tone", none: "None", aces: "ACES", exposure: "Exposure",
  vignette: "Vignette", ditherArt: "Dither art", orderedDither: "Ordered dither",
  options: "Options", presets: "Presets", savePreset: "Save preset", presetName: "Preset name", deletePreset: "Delete preset",
  noPresets: "No preset saved yet. Use Save preset to keep the settings you have now.",
  swatch: "Color", noWebgl: "This web view cannot draw the background (WebGL is not available).",
};
const pt: typeof en = {
  title: "Plano de fundo personalizado", leftPanel: "Looks", rightPanel: "Configurações",
  colors: "Cores", randomize: "Aleatório", fromImage: "De uma imagem", activeColors: "Cores ativas",
  gradientMap: "Mapa de gradiente", reset: "Redefinir", blend: "Mistura", stops: "Paradas", dragToReposition: "Arraste para reposicionar", stop: "Parada",
  bias: "Viés", sharpness: "Nitidez",
  shape: "Forma", contrast: "Contraste",
  motion: "Movimento",
  grade: "Correção", brightness: "Brilho", saturation: "Saturação",
  effects: "Efeitos", grain: "Granulação", classic: "Clássica", film: "Filme", amount: "Quantidade", size: "Tamanho", color: "Cor",
  midtone: "Foco nos tons médios", seed: "Semente", animateGrain: "Animar granulação (vídeo)",
  tone: "Tom", none: "Nenhum", aces: "ACES", exposure: "Exposição",
  vignette: "Vinheta", ditherArt: "Arte com dither", orderedDither: "Dither ordenado",
  options: "Opções", presets: "Presets", savePreset: "Salvar preset", presetName: "Nome do preset", deletePreset: "Excluir preset",
  noPresets: "Nenhum preset salvo ainda. Use Salvar preset para guardar as configurações atuais.",
  swatch: "Cor", noWebgl: "Esta janela não consegue desenhar o plano de fundo (o WebGL não está disponível).",
};

export const portuguese = navigator.language.toLowerCase().startsWith("pt");
export const L: typeof en = portuguese ? pt : en;
