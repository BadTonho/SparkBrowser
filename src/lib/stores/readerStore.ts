import { writable, get } from "svelte/store";
import type { ReaderArticle, ReaderTheme, ReaderFont, ReaderWidth } from "../types/reader";
import { invoke, isTauriRuntime } from "../services/tauri-bridge";
import { setOverlayVisibility } from "../services/bridge/overlays";
import { sanitizeArticleHtml } from "../services/reader-sanitizer";

export const isReaderOpen = writable<boolean>(false);
export const readerArticle = writable<ReaderArticle | null>(null);
export const readerLoading = writable<boolean>(false);
export const readerError = writable<string | null>(null);

export const readerTheme = writable<ReaderTheme>("sepia");
export const readerFont = writable<ReaderFont>("serif");
export const readerFontSize = writable<number>(18);
export const readerWidth = writable<ReaderWidth>("medium");

export async function openReaderMode(tabId?: string): Promise<void> {
  try {
    readerLoading.set(true);
    readerError.set(null);
    isReaderOpen.set(true);
    await setOverlayVisibility(true, "reader-mode");

    if (!isTauriRuntime()) {
      readerArticle.set({
        title: "Exemplo de Artigo para Modo Leitura",
        byline: "Spark",
        site_name: "veloce.internal",
        content_html: "<p>Este é um exemplo de artigo renderizado no modo leitura imersivo. O conteúdo é limpo de distrações, anúncios e barras laterais para uma experiência de leitura focada e confortável.</p><p>Você pode personalizar a tipografia, o tamanho da fonte, a largura da coluna e o tema visual nas opções acima.</p>",
        text_content: "Este é um exemplo de artigo renderizado no modo leitura imersivo.",
        word_count: 65,
        reading_time_mins: 1,
        url: "https://example.com/article",
      });
      return;
    }

    const article = await invoke<ReaderArticle>("get_reader_article", { tabId: tabId ?? null });
    if (!article || !article.content_html) {
      throw new Error("Não foi possível extrair o conteúdo principal desta página.");
    }
    readerArticle.set({ ...article, content_html: sanitizeArticleHtml(article.content_html) });
  } catch (err: any) {
    readerError.set(err?.message ?? String(err));
  } finally {
    readerLoading.set(false);
  }
}

export async function closeReaderMode(): Promise<void> {
  isReaderOpen.set(false);
  readerArticle.set(null);
  readerError.set(null);
  await setOverlayVisibility(false, "reader-mode");
}

export function toggleReaderTheme(): void {
  const current = get(readerTheme);
  const themes: ReaderTheme[] = ["paper", "sepia", "dark", "oled"];
  const next = themes[(themes.indexOf(current) + 1) % themes.length];
  readerTheme.set(next);
}

export function changeReaderFontSize(delta: number): void {
  readerFontSize.update(size => Math.max(14, Math.min(28, size + delta)));
}

