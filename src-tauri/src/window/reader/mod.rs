use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

mod images;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderArticle {
    pub title: String,
    pub byline: String,
    pub site_name: String,
    pub content_html: String,
    pub text_content: String,
    pub word_count: usize,
    pub reading_time_mins: usize,
    pub url: String,
    /// The article's images as `data:` URLs, keyed by their address in `content_html`. Images left
    /// out here are removed by the interface's sanitizer.
    #[serde(default)]
    pub images: HashMap<String, String>,
}

/// What the extraction script returns: the article plus the addresses of its images.
#[cfg(windows)]
#[derive(Deserialize)]
struct Extracted {
    #[serde(flatten)]
    article: ReaderArticle,
    #[serde(default)]
    image_urls: Vec<String>,
}

#[cfg(windows)]
const EXTRACT_SCRIPT: &str = r#"
(() => {
  try {
    const doc = document;
    if (!doc || !doc.body) return JSON.stringify({ error: "no_document" });

    const ogTitle = doc.querySelector('meta[property="og:title"]')?.getAttribute('content');
    const h1 = doc.querySelector('article h1, h1')?.innerText?.trim();
    const title = h1 || ogTitle || doc.title || "Untitled";

    const authorMeta = doc.querySelector('meta[name="author"], meta[property="article:author"]')?.getAttribute('content');
    const authorEl = doc.querySelector('[rel="author"], .byline, .author, .entry-author')?.innerText?.trim();
    const byline = authorMeta || authorEl || "";

    const ogSite = doc.querySelector('meta[property="og:site_name"]')?.getAttribute('content');
    const site_name = ogSite || location.hostname.replace(/^www\./, "");

    const candidates = Array.from(doc.querySelectorAll('article, [role="main"], main, .article, .post, .entry-content, .story-body, .article-body, #content, .content'));
    let bestContainer = null;
    let maxScore = 0;

    for (const el of candidates) {
      const pCount = el.querySelectorAll('p').length;
      const textLen = el.innerText?.length || 0;
      const score = pCount * 120 + textLen;
      if (score > maxScore) {
        maxScore = score;
        bestContainer = el;
      }
    }

    if (!bestContainer || maxScore < 200) {
      const divs = Array.from(doc.querySelectorAll('div, section'));
      for (const el of divs) {
        const pCount = el.querySelectorAll('p').length;
        if (pCount >= 2) {
          const textLen = el.innerText?.length || 0;
          const score = pCount * 100 + textLen;
          if (score > maxScore) {
            maxScore = score;
            bestContainer = el;
          }
        }
      }
    }

    if (!bestContainer) {
      bestContainer = doc.body;
    }

    const clone = bestContainer.cloneNode(true);
    const unwanted = clone.querySelectorAll('script, style, noscript, iframe, svg, nav, header, footer, form, button, input, .advertisement, .ad, .social-share, .comments, .related-posts, [aria-hidden="true"]');
    unwanted.forEach(node => node.remove());

    const image_urls = [];
    clone.querySelectorAll('img').forEach(img => {
      const src = img.getAttribute('src') || img.getAttribute('data-src') || '';
      if (!src || src.startsWith('data:image/svg')) {
        img.remove();
        return;
      }
      try {
        const absolute = new URL(src, location.href).href;
        img.setAttribute('src', absolute);
        if (/^https?:/i.test(absolute) && !image_urls.includes(absolute)) image_urls.push(absolute);
      } catch(e) {
        img.remove();
        return;
      }
      img.removeAttribute('srcset');
      img.removeAttribute('style');
      img.removeAttribute('class');
      img.removeAttribute('loading');
    });

    const cleanHtml = clone.innerHTML;
    const textContent = clone.innerText || "";
    const words = textContent.trim().split(/\s+/).filter(Boolean);
    const word_count = words.length;
    const reading_time_mins = Math.max(1, Math.ceil(word_count / 200));

    return JSON.stringify({
      title,
      byline,
      site_name,
      content_html: cleanHtml,
      text_content: textContent.slice(0, 5000),
      word_count,
      reading_time_mins,
      url: location.href,
      image_urls: image_urls.slice(0, __MAX_IMAGES__)
    });
  } catch(e) {
    return JSON.stringify({ error: String(e) });
  }
})()
"#;

pub async fn extract_article(
    app: &AppHandle,
    window_id: &str,
    tab_id: Option<String>,
) -> Result<ReaderArticle, String> {
    let state = app.state::<crate::state::browser_state::BrowserState>();
    let id = match tab_id {
        Some(id) => id,
        None => state
            .session()
            .window(window_id)
            .ok_or("windows.not_found")?
            .active_tab_id()
            .lock()
            .map_err(|e| e.to_string())?
            .clone(),
    };

    let label = crate::state::tab_model::webview_label_for_tab(&id);
    #[cfg_attr(not(windows), allow(unused_variables))]
    let view = app
        .get_webview(&label)
        .ok_or_else(|| "Content webview not found".to_string())?;

    #[cfg(windows)]
    {
        use crate::sync::Mutex;
        use std::sync::Arc;
        use webview2_com::ExecuteScriptCompletedHandler;
        use windows_core::PCWSTR;

        let (sender, receiver) = tokio::sync::oneshot::channel();
        let sender = Arc::new(Mutex::new(Some(sender)));

        let script = EXTRACT_SCRIPT.replace("__MAX_IMAGES__", &images::MAX_IMAGES.to_string());
        let script_wide: Vec<u16> = script.encode_utf16().chain(Some(0)).collect();

        let sender_for_eval = sender.clone();
        view.with_webview(move |platform| {
            let res = (|| -> windows_core::Result<()> {
                let core = unsafe { platform.controller().CoreWebView2()? };
                let cb = sender_for_eval.clone();
                let handler =
                    ExecuteScriptCompletedHandler::create(Box::new(move |hr, json_str| {
                        let result_str = (|| -> windows_core::Result<String> {
                            hr?;
                            Ok(json_str)
                        })();
                        if let Ok(mut lock) = cb.lock() {
                            if let Some(tx) = lock.take() {
                                crate::error_log::ignored(
                                    tx.send(result_str.map_err(|e| e.to_string())),
                                );
                            }
                        }
                        Ok(())
                    }));
                unsafe {
                    core.ExecuteScript(PCWSTR(script_wide.as_ptr()), &handler)?;
                }
                Ok(())
            })();
            if let Err(e) = res {
                if let Ok(mut lock) = sender.lock() {
                    if let Some(tx) = lock.take() {
                        crate::error_log::ignored(tx.send(Err(e.to_string())));
                    }
                }
            }
        })
        .map_err(|e| e.to_string())?;

        let raw_result = tokio::time::timeout(std::time::Duration::from_secs(10), receiver)
            .await
            .map_err(|_| "Reader extraction timed out".to_string())?
            .map_err(|_| "Evaluation channel closed".to_string())??;

        // WebView2 ExecuteScript returns the result JSON-encoded as a JSON string
        let inner_json: String = serde_json::from_str(&raw_result).unwrap_or(raw_result);

        let Extracted {
            mut article,
            image_urls,
        } = serde_json::from_str(&inner_json)
            .map_err(|e| format!("Failed to parse article data: {e}"))?;
        article.images = images::cached(&view, image_urls).await;

        Ok(article)
    }

    #[cfg(not(windows))]
    {
        // WebKitGTK has no article extraction yet. The interface shows this code in its own
        // language instead of a made-up article.
        Err("reader.unsupported".into())
    }
}
