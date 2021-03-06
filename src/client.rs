use cef_sys::{
    cef_browser_t, cef_client_t, cef_frame_t, cef_load_handler_t, cef_process_id_t,
    cef_process_message_t, cef_request_handler_t, cef_life_span_handler_t, cef_render_handler_t,
    cef_context_menu_handler_t, cef_dialog_handler_t,
    cef_display_handler_t,
    cef_download_handler_t,
    cef_drag_handler_t,
    cef_find_handler_t,
    cef_focus_handler_t,
    cef_jsdialog_handler_t,
    cef_keyboard_handler_t,
    cef_audio_handler_t,
};
use downcast_rs::{impl_downcast, Downcast};
use std::ptr::null_mut;

pub mod audio_handler;
pub mod context_menu_handler;
pub mod dialog_handler;
pub mod display_handler;
pub mod download_handler;
pub mod drag_handler;
pub mod find_handler;
pub mod focus_handler;
pub mod js_dialog_handler;
pub mod keyboard_handler;
pub mod life_span_handler;
pub mod render_handler;
pub mod request_handler;

use self::{
    audio_handler::AudioHandler,
    context_menu_handler::ContextMenuHandler,
    dialog_handler::DialogHandler,
    display_handler::DisplayHandler,
    download_handler::DownloadHandler,
    drag_handler::DragHandler,
    find_handler::FindHandler,
    focus_handler::FocusHandler,
    js_dialog_handler::JsDialogHandler,
    keyboard_handler::KeyboardHandler,
    life_span_handler::LifeSpanHandler,
    render_handler::RenderHandler,
    request_handler::RequestHandler,
};

use crate::{
    browser::Browser,
    frame::Frame,
    load_handler::LoadHandler,
    process::{ProcessId, ProcessMessage},
    refcounted::{RefCountedPtr, Wrapper},
};

ref_counted_ptr!{
    pub struct Client(*mut cef_client_t);
}

impl Client {
    pub fn new<C: ClientCallbacks>(callbacks: C) -> Client {
        unsafe{ Client::from_ptr_unchecked(ClientWrapper::new(Box::new(callbacks)).wrap().into_raw()) }
    }
}

/// Implement this trait to provide handler implementations.
pub trait ClientCallbacks: 'static + Send + Sync + Downcast {
    /// Return the handler for audio rendering events.
    fn get_audio_handler(&self) -> Option<AudioHandler> { None }
    /// Return the handler for context menus. If no handler is provided the default
    /// implementation will be used.
    fn get_context_menu_handler(&self) -> Option<ContextMenuHandler> { None }
    /// Return the handler for dialogs. If no handler is provided the default
    /// implementation will be used.
    fn get_dialog_handler(&self) -> Option<DialogHandler> { None }
    /// Return the handler for browser display state events.
    fn get_display_handler(&self) -> Option<DisplayHandler> { None }
    /// Return the handler for download events. If no handler is returned downloads
    /// will not be allowed.
    fn get_download_handler(&self) -> Option<DownloadHandler> { None }
    /// Return the handler for drag events.
    fn get_drag_handler(&self) -> Option<DragHandler> { None }
    /// Return the handler for find result events.
    fn get_find_handler(&self) -> Option<FindHandler> { None }
    /// Return the handler for focus events.
    fn get_focus_handler(&self) -> Option<FocusHandler> { None }
    /// Return the handler for JavaScript dialogs. If no handler is provided the
    /// default implementation will be used.
    fn get_jsdialog_handler(&self) -> Option<JsDialogHandler> { None }
    /// Return the handler for keyboard events.
    fn get_keyboard_handler(&self) -> Option<KeyboardHandler> { None }
    /// Return the handler for browser life span events.
    fn get_life_span_handler(&self) -> Option<LifeSpanHandler> { None }
    /// Return the handler for browser load status events.
    fn get_load_handler(&self) -> Option<LoadHandler> {
        None
    }
    /// Return the handler for off-screen rendering events.
    fn get_render_handler(&self) -> Option<RenderHandler> { None }
    /// Return the handler for browser request events.
    fn get_request_handler(&self) -> Option<RequestHandler> {
        None
    }
    /// Called when a new message is received from a different process. Return true
    /// if the message was handled or false otherwise.
    fn on_process_message_received(
        &self,
        browser: Browser,
        frame: Frame,
        message: ProcessMessage,
    ) -> bool {
        false
    }
}

impl_downcast!(ClientCallbacks);

#[repr(transparent)]
pub(crate) struct ClientWrapper(Box<dyn ClientCallbacks>);

impl Wrapper for ClientWrapper {
    type Cef = cef_client_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_client_t {
                base: unsafe { std::mem::zeroed() },
                get_audio_handler: Some(Self::get_audio_handler),
                get_context_menu_handler: Some(Self::get_context_menu_handler),
                get_dialog_handler: Some(Self::get_dialog_handler),
                get_display_handler: Some(Self::get_display_handler),
                get_download_handler: Some(Self::get_download_handler),
                get_drag_handler: Some(Self::get_drag_handler),
                get_find_handler: Some(Self::get_find_handler),
                get_focus_handler: Some(Self::get_focus_handler),
                get_jsdialog_handler: Some(Self::get_jsdialog_handler),
                get_keyboard_handler: Some(Self::get_keyboard_handler),
                get_life_span_handler: Some(Self::get_life_span_handler),
                get_load_handler: Some(Self::get_load_handler),
                get_render_handler: Some(Self::get_render_handler),
                get_request_handler: Some(Self::get_request_handler),
                on_process_message_received: Some(Self::process_message_received),
            },
            self,
        )
    }
}

impl ClientWrapper {
    pub(crate) fn new(delegate: Box<dyn ClientCallbacks>) -> Self {
        Self(delegate)
    }
    pub(crate) fn get_client<C: ClientCallbacks>(&self) -> Option<&C> {
        self.0.downcast_ref()
    }
}

cef_callback_impl! {
    impl for ClientWrapper: cef_client_t {
        fn get_audio_handler(&self) -> *mut cef_audio_handler_t {
            self.0.get_audio_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_context_menu_handler(&self) -> *mut cef_context_menu_handler_t {
            self.0.get_context_menu_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_dialog_handler(&self) -> *mut cef_dialog_handler_t {
            self.0.get_dialog_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_display_handler(&self) -> *mut cef_display_handler_t {
            self.0.get_display_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_download_handler(&self) -> *mut cef_download_handler_t {
            self.0.get_download_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_drag_handler(&self) -> *mut cef_drag_handler_t {
            self.0.get_drag_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_find_handler(&self) -> *mut cef_find_handler_t {
            self.0.get_find_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_focus_handler(&self) -> *mut cef_focus_handler_t {
            self.0.get_focus_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_jsdialog_handler(&self) -> *mut cef_jsdialog_handler_t {
            self.0.get_jsdialog_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_keyboard_handler(&self) -> *mut cef_keyboard_handler_t {
            self.0.get_keyboard_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_life_span_handler(&self) -> *mut cef_life_span_handler_t {
            self.0.get_life_span_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_load_handler(&self) -> *mut cef_load_handler_t {
            self.0.get_load_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_render_handler(&self) -> *mut cef_render_handler_t {
            self.0.get_render_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_request_handler(&self) -> *mut cef_request_handler_t {
            self.0.get_request_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn process_message_received(
            &self,
            browser       : Browser       : *mut cef_browser_t,
            frame         : Frame         : *mut cef_frame_t,
            source_process: ProcessId     : cef_process_id_t::Type,
            message       : ProcessMessage: *mut cef_process_message_t
        ) -> std::os::raw::c_int {
            assert_eq!(ProcessId::Renderer, source_process);
            self.0.on_process_message_received(browser, frame, message) as std::os::raw::c_int
        }
    }
}
