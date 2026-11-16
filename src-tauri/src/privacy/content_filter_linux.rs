//! Network blocking for WebKitGTK pages. WebKit cannot cancel a request from an event handler, so
//! the filter lists are compiled into a WebKit content rule list that every protected page uses.
use crate::state::browser_state::BrowserState;
use gtk::glib::{self, translate::ToGlibPtr};
use gtk::prelude::WidgetExt;
use std::{cell::RefCell, collections::HashMap, ffi::CString, ptr, sync::Arc};
use tauri::{AppHandle, Manager, Webview};
use webkit2gtk::{LoadEvent, UserContentManager, WebViewExt};
use webkit2gtk_sys as ffi;

const IDENTIFIER: &str = "spark-network-rules";

struct Filter(*mut ffi::WebKitUserContentFilter);

impl Drop for Filter {
    fn drop(&mut self) {
        unsafe { ffi::webkit_user_content_filter_unref(self.0) }
    }
}

struct View {
    manager: UserContentManager,
    attached: bool,
}

thread_local! {
    /// The compiled rules. WebKit objects belong to the main thread, like every other registry here.
    static FILTER: RefCell<Option<Filter>> = const { RefCell::new(None) };
    static VIEWS: RefCell<HashMap<String, View>> = RefCell::new(HashMap::new());
}

/// Compiles the current lists in the background and applies them to every page when ready.
pub fn refresh(app: &AppHandle) {
    let lists = app
        .state::<Arc<super::filter_lists::FilterLists>>()
        .inner()
        .clone();
    let app = app.clone();
    let _ = std::thread::Builder::new()
        .name("spark-content-rules".into())
        .spawn(move || {
            let Some(json) = lists.content_blocking_json() else {
                eprintln!("Could not convert the filter lists to content rules");
                return;
            };
            let main = app.clone();
            let _ = main.run_on_main_thread(move || save(&app, json));
        });
}

fn save(app: &AppHandle, json: String) {
    let Ok(directory) = app
        .path()
        .app_cache_dir()
        .map(|dir| dir.join("content-rules"))
    else {
        return;
    };
    if std::fs::create_dir_all(&directory).is_err() {
        return;
    }
    let (Some(path), Ok(identifier)) = (directory.to_str(), CString::new(IDENTIFIER)) else {
        return;
    };
    let Ok(path) = CString::new(path) else {
        return;
    };
    let bytes = glib::Bytes::from_owned(json.into_bytes());
    unsafe extern "C" fn finished(
        source: *mut glib::gobject_ffi::GObject,
        result: *mut gtk::gio::ffi::GAsyncResult,
        data: glib::ffi::gpointer,
    ) {
        let app = Box::from_raw(data as *mut AppHandle);
        let mut error = ptr::null_mut();
        let filter = ffi::webkit_user_content_filter_store_save_finish(
            source as *mut ffi::WebKitUserContentFilterStore,
            result,
            &mut error,
        );
        glib::gobject_ffi::g_object_unref(source);
        if filter.is_null() {
            if !error.is_null() {
                eprintln!(
                    "Content rules were rejected: {}",
                    std::ffi::CStr::from_ptr((*error).message).to_string_lossy()
                );
                glib::ffi::g_error_free(error);
            }
            return;
        }
        replace(&app, Filter(filter));
    }
    unsafe {
        let store = ffi::webkit_user_content_filter_store_new(path.as_ptr());
        ffi::webkit_user_content_filter_store_save(
            store,
            identifier.as_ptr(),
            bytes.to_glib_none().0,
            ptr::null_mut(),
            Some(finished),
            Box::into_raw(Box::new(app.clone())) as glib::ffi::gpointer,
        );
    }
}

/// Swaps in a new rule list and re-evaluates every page.
fn replace(app: &AppHandle, filter: Filter) {
    VIEWS.with(|views| {
        for view in views.borrow_mut().values_mut() {
            detach(view);
        }
    });
    FILTER.with(|current| *current.borrow_mut() = Some(filter));
    let labels: Vec<String> = VIEWS.with(|views| views.borrow().keys().cloned().collect());
    for label in labels {
        let uri = app
            .get_webview(&label)
            .and_then(|view| view.url().ok())
            .map(|url| url.to_string());
        apply(app, &label, uri.as_deref());
    }
}

fn detach(view: &mut View) {
    if !view.attached {
        return;
    }
    FILTER.with(|filter| {
        if let Some(filter) = filter.borrow().as_ref() {
            unsafe {
                ffi::webkit_user_content_manager_remove_filter(
                    view.manager.to_glib_none().0,
                    filter.0,
                );
            }
        }
    });
    view.attached = false;
}

/// Attaches the rules to a page that the shield protects and detaches them from one that it does not.
fn apply(app: &AppHandle, label: &str, uri: Option<&str>) {
    let state = app.state::<BrowserState>();
    let wanted = state
        .policies()
        .shield_enabled()
        .lock()
        .map(|enabled| *enabled)
        .unwrap_or(true)
        && uri.is_none_or(|uri| super::adblock_engine::site_is_protected(&state, uri));
    VIEWS.with(|views| {
        let mut views = views.borrow_mut();
        let Some(view) = views.get_mut(label) else {
            return;
        };
        if wanted && !view.attached {
            FILTER.with(|filter| {
                if let Some(filter) = filter.borrow().as_ref() {
                    unsafe {
                        ffi::webkit_user_content_manager_add_filter(
                            view.manager.to_glib_none().0,
                            filter.0,
                        );
                    }
                    view.attached = true;
                }
            });
        } else if !wanted {
            detach(view);
        }
    });
}

pub fn install(app: &AppHandle, view: &Webview) -> Result<(), String> {
    let (app, label) = (app.clone(), view.label().to_owned());
    view.with_webview(move |platform| {
        let web_view = platform.inner();
        let Some(manager) = web_view.user_content_manager() else {
            return;
        };
        VIEWS.with(|views| {
            views.borrow_mut().insert(
                label.clone(),
                View {
                    manager,
                    attached: false,
                },
            )
        });
        let (a, l) = (app.clone(), label.clone());
        web_view.connect_load_changed(move |view, event| {
            if matches!(event, LoadEvent::Started | LoadEvent::Redirected) {
                apply(&a, &l, view.uri().as_deref());
            }
        });
        let forget = label.clone();
        web_view.connect_destroy(move |_| {
            VIEWS.with(|views| views.borrow_mut().remove(&forget));
        });
        apply(&app, &label, None);
    })
    .map_err(|e| e.to_string())
}
