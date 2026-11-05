export interface ReaderArticle {
  title: string;
  byline: string;
  site_name: string;
  content_html: string;
  text_content: string;
  word_count: number;
  reading_time_mins: number;
  url: string;
}

export type ReaderTheme = "paper" | "sepia" | "dark" | "oled";
export type ReaderFont = "serif" | "sans" | "mono";
export type ReaderWidth = "narrow" | "medium" | "wide";

