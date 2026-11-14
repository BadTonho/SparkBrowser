use super::*;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use webview2_com::{
    CoTaskMemPWSTR, FaviconChangedEventHandler, GetFaviconCompletedHandler,
    Microsoft::Web::WebView2::Win32::*, NavigationCompletedEventHandler,
    NavigationStartingEventHandler,
};
use windows_core::Interface;

pub fn install(app: &AppHandle, view: &tauri::Webview, token: &str) -> Result<(), String> {
    let (app, label, token) = (app.clone(), view.label().to_owned(), token.to_owned());
    view.with_webview(move |platform| {
        let Ok(core) = (unsafe { platform.controller().CoreWebView2() }) else {
            return;
        };
        let Ok(icons) = core.cast::<ICoreWebView2_15>() else {
            return;
        };
        let navigation = Arc::new(AtomicU64::new(0));
        let document_epoch = Arc::new(AtomicU64::new(epoch(&app)));
        let n = navigation.clone();
        let (a, e) = (app.clone(), document_epoch.clone());
        unsafe {
            let _ = core.add_NavigationStarting(
                &NavigationStartingEventHandler::create(Box::new(move |_, args| {
                    let Some(args) = args else {
                        return Ok(());
                    };
                    let mut uri = Default::default();
                    let mut cancelled = Default::default();
                    args.Uri(&mut uri)?;
                    args.Cancel(&mut cancelled)?;
                    let raw = CoTaskMemPWSTR::from(uri).to_string();
                    if cancelled.as_bool() || site_icons_repo::site_key(&raw).is_none() {
                        return Ok(());
                    }
                    n.fetch_add(1, Ordering::SeqCst);
                    e.store(epoch(&a), Ordering::SeqCst);
                    Ok(())
                })),
                &mut 0,
            );
        }
        let (a, l, t, n, e) = (
            app.clone(),
            label.clone(),
            token.clone(),
            navigation.clone(),
            document_epoch.clone(),
        );
        unsafe {
            let _ = icons.add_FaviconChanged(
                &FaviconChangedEventHandler::create(Box::new(move |sender, _| {
                    if let Some(sender) = sender {
                        capture(&a, &l, &t, &sender, &n, e.load(Ordering::SeqCst));
                    }
                    Ok(())
                })),
                &mut 0,
            );
            let _ = core.add_NavigationCompleted(
                &NavigationCompletedEventHandler::create(Box::new(move |sender, _| {
                    if let Some(sender) = sender {
                        capture(
                            &app,
                            &label,
                            &token,
                            &sender,
                            &navigation,
                            document_epoch.load(Ordering::SeqCst),
                        );
                    }
                    Ok(())
                })),
                &mut 0,
            );
        }
    })
    .map_err(|e| e.to_string())
}

fn capture(
    app: &AppHandle,
    label: &str,
    token: &str,
    core: &ICoreWebView2,
    navigation: &Arc<AtomicU64>,
    epoch: u64,
) {
    if epoch != super::epoch(app) {
        return;
    }
    let Ok(icons) = core.cast::<ICoreWebView2_15>() else {
        return;
    };
    let (mut source, mut icon_uri) = (Default::default(), Default::default());
    unsafe {
        if core.Source(&mut source).is_err() || icons.FaviconUri(&mut icon_uri).is_err() {
            return;
        }
    }
    let url = CoTaskMemPWSTR::from(source).to_string();
    if CoTaskMemPWSTR::from(icon_uri).to_string().is_empty() || !current(app, label, token, &url) {
        return;
    }
    let Some(key) = site_icons_repo::site_key(&url) else {
        return;
    };
    let generation = navigation.load(Ordering::SeqCst);
    let (app, label, token, navigation) = (
        app.clone(),
        label.to_owned(),
        token.to_owned(),
        navigation.clone(),
    );
    let handler = GetFaviconCompletedHandler::create(Box::new(move |status, stream| {
        if status.is_err()
            || epoch != super::epoch(&app)
            || generation != navigation.load(Ordering::SeqCst)
            || !current(&app, &label, &token, &url)
        {
            return Ok(());
        }
        let Some(stream) = stream else {
            return Ok(());
        };
        // One bounded read avoids trusting the native stream's declared allocation size.
        let mut bytes = vec![0u8; site_icons_repo::MAX_BYTES + 1];
        let mut read = 0;
        if unsafe {
            stream.Read(
                bytes.as_mut_ptr().cast(),
                bytes.len() as u32,
                Some(&mut read),
            )
        }
        .is_err()
            || read as usize > site_icons_repo::MAX_BYTES
        {
            return Ok(());
        }
        bytes.truncate(read as usize);
        tauri::async_runtime::spawn(async move {
            let worker_app = app.clone();
            let result = crate::database::executor::run(app.clone(), move |conn| {
                if generation != navigation.load(Ordering::SeqCst)
                    || !current(&worker_app, &label, &token, &url)
                {
                    return Ok(false);
                }
                save(&worker_app, conn, epoch, &key, &bytes)
            })
            .await;
            if matches!(result, Ok(true)) {
                super::super::new_tab::invalidate(&app);
                let _ = crate::security::emit_shell(&app, "site-icons-changed", ());
            }
        });
        Ok(())
    }));
    unsafe {
        let _ = icons.GetFavicon(COREWEBVIEW2_FAVICON_IMAGE_FORMAT_PNG, &handler);
    }
}
