//! WebView2 network-request interception for content WebViews.
use std::sync::Arc;
use tauri::{AppHandle, Manager, Webview};
use webview2_com::{
    CoTaskMemPWSTR, Microsoft::Web::WebView2::Win32::*, WebResourceRequestedEventHandler,
};
use windows_core::{Interface, HSTRING, PWSTR};

pub fn install(app: &AppHandle, view: &Webview) -> Result<(), String> {
    let app = app.clone();
    let error_app = app.clone();
    view.with_webview(move |platform| unsafe {
        let result = (|| -> windows_core::Result<()> {
            let core = platform.controller().CoreWebView2()?;
            for pattern in ["http://*", "https://*"] {
                let filter = HSTRING::from(pattern);
                if let Ok(newer) = core.cast::<ICoreWebView2_22>() {
                    newer.AddWebResourceRequestedFilterWithRequestSourceKinds(
                        &filter,
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                        COREWEBVIEW2_WEB_RESOURCE_REQUEST_SOURCE_KINDS_ALL,
                    )?;
                } else {
                    core.AddWebResourceRequestedFilter(
                        &filter,
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                    )?;
                }
            }
            let env = platform.environment();
            let mut token = 0;
            core.add_WebResourceRequested(
                &WebResourceRequestedEventHandler::create(Box::new(move |sender, args| {
                    let (Some(sender), Some(args)) = (sender, args) else {
                        return Ok(());
                    };
                    let state = app.state::<crate::state::browser_state::BrowserState>();
                    let request = args.Request()?;
                    let mut value = PWSTR::null();
                    request.Uri(&mut value)?;
                    let url = CoTaskMemPWSTR::from(value).to_string();
                    let mut value = PWSTR::null();
                    request.Method(&mut value)?;
                    let method = CoTaskMemPWSTR::from(value).to_string();
                    let mut value = PWSTR::null();
                    sender.Source(&mut value)?;
                    let source = CoTaskMemPWSTR::from(value).to_string();
                    let mut context = COREWEBVIEW2_WEB_RESOURCE_CONTEXT::default();
                    args.ResourceContext(&mut context)?;
                    let kind = match context {
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT => "document",
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_SCRIPT => "script",
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_IMAGE => "image",
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_STYLESHEET => "stylesheet",
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FONT => "font",
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_MEDIA => "media",
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_XML_HTTP_REQUEST
                        | COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FETCH => "xmlhttprequest",
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_WEBSOCKET => "websocket",
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_PING => "ping",
                        _ => "other",
                    };
                    let source = if source.starts_with("http") {
                        source.as_str()
                    } else {
                        url.as_str()
                    };
                    let shield = state
                        .shield_enabled
                        .lock()
                        .map(|enabled| *enabled)
                        .unwrap_or(false);
                    if shield
                        && super::adblock_engine::site_is_protected(&state, source)
                        && super::adblock_engine::site_is_protected(&state, &url)
                        && app
                            .state::<Arc<super::filter_lists::FilterLists>>()
                            .blocks(&url, source, kind, &method)
                    {
                        let response = env.CreateWebResourceResponse(
                            None,
                            403,
                            &HSTRING::from("Blocked"),
                            &HSTRING::from("Cache-Control: no-store\r\n"),
                        )?;
                        args.SetResponse(&response)?;
                        if let Ok(mut count) = state.blocked_count.lock() {
                            *count += 1;
                        }
                    } else if let Ok(settings) = state.gpc_settings.lock() {
                        if let (Ok(target), Ok(owner)) =
                            (url::Url::parse(&url), url::Url::parse(source))
                        {
                            if settings.applies_to(&target) && settings.applies_to(&owner) {
                                request
                                    .Headers()?
                                    .SetHeader(&HSTRING::from("Sec-GPC"), &HSTRING::from("1"))?;
                            }
                        }
                    }
                    Ok(())
                })),
                &mut token,
            )?;
            Ok(())
        })();
        if let Err(error) = result {
            eprintln!("Could not install WebView request filtering: {error}");
            let _ = crate::security::emit_shell(&error_app, "browser-error", "Network request filtering could not be installed for a page");
        }
    })
    .map_err(|error| error.to_string())
}
