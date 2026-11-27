//! WebView2 network-request interception for content WebViews.
use crate::sync::Mutex;
use std::sync::Arc;
use tauri::{AppHandle, Manager, Webview};
use webview2_com::{
    CoTaskMemPWSTR, Microsoft::Web::WebView2::Win32::*, NavigationStartingEventHandler,
    WebResourceRequestedEventHandler,
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
            // NavigationStarting is emitted only for the top-level WebView. Correlate
            // its destination with one document request; never replay a request.
            let pending = Arc::new(Mutex::new(None::<String>));
            let navigation = Arc::clone(&pending);
            let mut navigation_token = 0;
            core.add_NavigationStarting(
                &NavigationStartingEventHandler::create(Box::new(move |_, args| {
                    if let Some(args) = args {
                        let mut value = PWSTR::null();
                        args.Uri(&mut value)?;
                        let raw = CoTaskMemPWSTR::from(value).to_string();
                        if let Ok(mut slot) = navigation.lock() {
                            *slot = canonical_request_url(&raw);
                        }
                    }
                    Ok(())
                })),
                &mut navigation_token,
            )?;
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
                    let mut url = CoTaskMemPWSTR::from(value).to_string();
                    let mut value = PWSTR::null();
                    request.Method(&mut value)?;
                    let method = CoTaskMemPWSTR::from(value).to_string();
                    let mut value = PWSTR::null();
                    sender.Source(&mut value)?;
                    let source = CoTaskMemPWSTR::from(value).to_string();
                    let mut context = COREWEBVIEW2_WEB_RESOURCE_CONTEXT::default();
                    args.ResourceContext(&mut context)?;
                    let top_document = context == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT
                        && pending.lock().ok().is_some_and(|mut slot| {
                            if slot.as_ref() == canonical_request_url(&url).as_ref()
                                && slot.is_some()
                            {
                                *slot = None;
                                true
                            } else {
                                false
                            }
                        });
                    if top_document && method.eq_ignore_ascii_case("GET") {
                        if let (Ok(parsed), Ok(settings)) = (
                            url::Url::parse(&url),
                            state.policies().tracking_settings().lock(),
                        ) {
                            let cleaned = super::tracking_parameters::clean(&parsed, &settings);
                            if cleaned != parsed {
                                request.SetUri(&HSTRING::from(cleaned.as_str()))?;
                                url = cleaned.to_string();
                            }
                        }
                    }
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
                    let source = if top_document {
                        url.as_str()
                    } else if source.starts_with("http") {
                        source.as_str()
                    } else {
                        url.as_str()
                    };
                    let shield = state
                        .policies()
                        .shield_enabled()
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
                        state.policies().record_blocked_request();
                    } else if let Ok(settings) = state.policies().gpc_settings().lock() {
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
            crate::error_log::warn(&format!(
                "Could not install WebView request filtering: {error}"
            ));
            crate::error_log::ignored(crate::security::emit_shell(
                &error_app,
                "browser-error",
                "Network request filtering could not be installed for a page",
            ));
        }
    })
    .map_err(|error| error.to_string())
}

fn canonical_request_url(raw: &str) -> Option<String> {
    let mut url = url::Url::parse(raw).ok()?;
    url.set_fragment(None);
    Some(url.to_string())
}
