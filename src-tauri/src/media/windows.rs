use super::{current, MediaService};
use tauri::{AppHandle, Manager};
use webview2_com::{
    IsDocumentPlayingAudioChangedEventHandler, IsMutedChangedEventHandler,
    Microsoft::Web::WebView2::Win32::*,
};
use windows_core::Interface;

pub async fn set_muted(app: &AppHandle, view: &tauri::Webview, muted: bool) -> Result<(), String> {
    let label = view.label().to_owned();
    let token = app
        .state::<crate::state::browser_state::BrowserState>()
        .keyboard_bridge_tokens
        .lock()
        .map_err(|_| "media.error")?
        .get(&label)
        .cloned()
        .ok_or("media.stale")?;
    let app = app.clone();
    let (send, receive) = tokio::sync::oneshot::channel();
    view.with_webview(move |platform| {
        let result = (|| unsafe {
            if !current(&app, &label, &token) {
                return Err("media.stale".to_string());
            }
            platform
                .controller()
                .CoreWebView2()
                .and_then(|c| c.cast::<ICoreWebView2_8>())
                .and_then(|c| c.SetIsMuted(muted))
                .map_err(|_| "media.unavailable".to_string())
        })();
        let _ = send.send(result);
    })
    .map_err(|_| "media.unavailable")?;
    tokio::time::timeout(std::time::Duration::from_secs(5), receive)
        .await
        .map_err(|_| "media.error")?
        .map_err(|_| "media.error")?
}

pub fn install(
    app: &AppHandle,
    view: &tauri::Webview,
    id: &str,
    token: &str,
) -> Result<(), String> {
    let (app, id, label, token) = (
        app.clone(),
        id.to_owned(),
        view.label().to_owned(),
        token.to_owned(),
    );
    view.with_webview(move |platform| unsafe {
        let Ok(core) = platform
            .controller()
            .CoreWebView2()
            .and_then(|c| c.cast::<ICoreWebView2_8>())
        else {
            return;
        };
        let initial = app.state::<MediaService>().tab(&id);
        let muted = initial.muted || initial.volume == 0;
        if core.SetIsMuted(muted).is_err() {
            return;
        }
        app.state::<MediaService>().update(&app, &id, |t| {
            t.audible = false;
            t.mute_available = true;
        });
        let (a, i, l, g) = (app.clone(), id.clone(), label.clone(), token.clone());
        let _ = core.add_IsDocumentPlayingAudioChanged(
            &IsDocumentPlayingAudioChangedEventHandler::create(Box::new(move |sender, _| {
                if current(&a, &l, &g) {
                    if let Some(c) = sender {
                        let c = c.cast::<ICoreWebView2_8>()?;
                        let mut audible = Default::default();
                        c.IsDocumentPlayingAudio(&mut audible)?;
                        a.state::<MediaService>()
                            .update(&a, &i, |t| t.audible = audible.as_bool());
                    }
                }
                Ok(())
            })),
            &mut 0,
        );
        let _ = core.add_IsMutedChanged(
            &IsMutedChangedEventHandler::create(Box::new(move |sender, _| {
                if current(&app, &label, &token) {
                    if let Some(c) = sender {
                        let c = c.cast::<ICoreWebView2_8>()?;
                        let mut muted = Default::default();
                        c.IsMuted(&mut muted)?;
                        app.state::<MediaService>().update(&app, &id, |t| {
                            if t.volume != 0 {
                                t.muted = muted.as_bool();
                            }
                        });
                    }
                }
                Ok(())
            })),
            &mut 0,
        );
    })
    .map_err(|_| "media.unavailable".into())
}
