use cef::client::render_handler::CursorType;
use cef_sys::HCURSOR;
use cef::drag::DragOperation;
use cef::values::{Rect, Point};
use cef::client::render_handler::ScreenInfo;
use cef::browser_host::PaintElementType;
use std::{
    time::{Duration, Instant},
    ffi::c_void,
};
use cef::{
    app::{App, AppCallbacks},
    browser::{Browser, BrowserSettings},
    browser_host::BrowserHost,
    browser_process_handler::{BrowserProcessHandler, BrowserProcessHandlerCallbacks},
    client::{
        Client, ClientCallbacks,
        life_span_handler::{LifeSpanHandler, LifeSpanHandlerCallbacks},
        render_handler::{RenderHandler, RenderHandlerCallbacks},
    },
    command_line::CommandLine,
    main_args::MainArgs,
    settings::{Settings, LogSeverity},
    window::WindowInfo,
};
use winit::{
    event::{Event, WindowEvent, StartCause},
    dpi::LogicalPosition,
    platform::windows::WindowExtWindows,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{CursorIcon, Window, WindowBuilder},
};
use winit_blit::{PixelBufferTyped, BGRA};
use parking_lot::Mutex;

pub struct AppCallbacksImpl {
    browser_process_handler: BrowserProcessHandler,
}
pub struct ClientCallbacksImpl {
    life_span_handler: LifeSpanHandler,
    render_handler: RenderHandler,
}
pub struct LifeSpanHandlerImpl {
    proxy: Mutex<EventLoopProxy<CefEvent>>,
}
pub struct BrowserProcessHandlerCallbacksImpl {
    proxy: Mutex<EventLoopProxy<CefEvent>>,
}
pub struct RenderHandlerCallbacksImpl {
    window: Window,
    pixel_buffer: Mutex<PixelBufferTyped<BGRA>>,
    popup_rect: Mutex<Option<Rect>>,
}

#[derive(Clone)]
enum CefEvent {
    ScheduleWork(Instant),
}

impl AppCallbacks for AppCallbacksImpl {
    fn on_before_command_line_processing (&self, process_type: Option<&str>, command_line: CommandLine) {
        if process_type == None {
            command_line.append_switch("disable-gpu");
            command_line.append_switch("disable-gpu-compositing");
        }
    }
    fn get_browser_process_handler(&self) -> Option<BrowserProcessHandler> {
        Some(self.browser_process_handler.clone())
    }
}

impl ClientCallbacks for ClientCallbacksImpl {
    fn get_life_span_handler(&self) -> Option<LifeSpanHandler> {
        Some(self.life_span_handler.clone())
    }
    fn get_render_handler(&self) -> Option<RenderHandler> {
        Some(self.render_handler.clone())
    }
}

impl LifeSpanHandlerCallbacks for LifeSpanHandlerImpl {
    fn on_before_close(&self, _browser: Browser) {
        cef::quit_message_loop()
    }
}

impl BrowserProcessHandlerCallbacks for BrowserProcessHandlerCallbacksImpl {
    fn on_schedule_message_pump_work(&self, delay_ms: i64) {
        println!("schedule work {}", delay_ms);
        if delay_ms <= 0 {
            cef::do_message_loop_work();
        } else {
            self.proxy.lock().send_event(CefEvent::ScheduleWork(Instant::now() + Duration::from_millis(delay_ms as u64))).ok();
        }
    }
}

impl RenderHandlerCallbacks for RenderHandlerCallbacksImpl {
    fn get_view_rect(&self, _: Browser) -> Rect {
        let inner_size = self.window.inner_size();
        Rect {
            x: 0,
            y: 0,
            width: inner_size.width.round() as i32,
            height: inner_size.height.round() as i32,
        }
    }
    fn on_popup_show(&self, browser: Browser, show: bool) {
        if !show {
            *self.popup_rect.lock() = None;
        }
    }
    fn get_screen_point(
        &self,
        browser: Browser,
        point: Point,
    ) -> Option<Point> {
        let physical_pos = LogicalPosition::new(point.x as _, point.y as _).to_physical(self.window.hidpi_factor());
        Some(Point::new(physical_pos.x as i32, physical_pos.y as i32))
    }
    fn on_popup_size(&self, _: Browser, mut rect: Rect) {
        let window_size: (u32, u32) = self.window.inner_size().into();
        let window_size = (window_size.0 as i32, window_size.1 as i32);
        rect.x = i32::max(rect.x, 0);
        rect.y = i32::max(rect.y, 0);
        rect.x = i32::min(rect.x, window_size.0 - rect.width);
        rect.y = i32::min(rect.y, window_size.1 - rect.height);
        *self.popup_rect.lock() = Some(rect);
    }
    fn get_screen_info(&self, _: Browser) -> Option<ScreenInfo> {
        let inner_size = self.window.inner_size();
        let rect = Rect {
            x: 0,
            y: 0,
            width: inner_size.width.round() as i32,
            height: inner_size.height.round() as i32,
        };

        Some(ScreenInfo {
            device_scale_factor: self.window.hidpi_factor() as f32,
            depth: 32,
            depth_per_component: 8,
            is_monochrome: false,
            rect: rect,
            available_rect: rect,
        })
    }
    fn on_paint(
        &self,
        _: Browser,
        element_type: PaintElementType,
        dirty_rects: &[Rect],
        buffer: &[u8],
        width: i32,
        height: i32
    ) {
        let buffer = BGRA::from_raw_slice(buffer);
        let buffer_row = |row: u32| &buffer[row as usize * width as usize..(1 + row) as usize * width as usize];
        let mut pixel_buffer = self.pixel_buffer.lock();
        if pixel_buffer.width() != width as u32 || pixel_buffer.height() != height as u32 {
            *pixel_buffer = PixelBufferTyped::new_supported(width as u32, height as u32, &self.window);
        }
        match (element_type, *self.popup_rect.lock()) {
            (PaintElementType::View, _) => {
                for rect in dirty_rects {
                    let row_span = rect.x as usize..rect.x as usize + rect.width as usize;
                    for row in (rect.y..rect.y+rect.height).map(|r| r as u32) {
                        pixel_buffer.row_mut(row).unwrap()
                            [row_span.clone()]
                            .copy_from_slice(&buffer_row(row)[row_span.clone()])
                    }

                    pixel_buffer.blit_rect(
                        (rect.x as u32, rect.y as u32),
                        (rect.x as u32, rect.y as u32),
                        (rect.width as u32, rect.height as u32),
                        &self.window
                    ).unwrap();
                }
            },
            (PaintElementType::Popup, Some(rect)) => {
                let row_span = rect.x as usize..rect.x as usize + rect.width as usize;
                for row in (rect.y..rect.y+rect.height).map(|r| r as u32) {
                    pixel_buffer.row_mut(row).unwrap()
                        [row_span.clone()]
                        .copy_from_slice(&buffer_row(row)[row_span.clone()])
                }

                pixel_buffer.blit_rect(
                    (rect.x as u32, rect.y as u32),
                    (rect.x as u32, rect.y as u32),
                    (rect.width as u32, rect.height as u32),
                    &self.window
                ).unwrap();
            },
            _ => (),
        }
    }
    fn on_accelerated_paint(
        &self,
        _browser: Browser,
        _type_: PaintElementType,
        _dirty_rects: &[Rect],
        _shared_handle: *mut c_void
    ) {
        unimplemented!()
    }
    fn on_cursor_change(
        &self,
        browser: Browser,
        cursor: HCURSOR,
        type_: CursorType
    ) {
        let winit_cursor = match type_ {
            CursorType::MiddlePanning |
            CursorType::EastPanning |
            CursorType::NorthPanning |
            CursorType::NorthEastPanning |
            CursorType::NorthWestPanning |
            CursorType::SouthPanning |
            CursorType::SouthEastPanning |
            CursorType::SouthWestPanning |
            CursorType::WestPanning |
            CursorType::Custom(_) |
            CursorType::Pointer => Some(CursorIcon::Default),
            CursorType::Cross => Some(CursorIcon::Crosshair),
            CursorType::Hand => Some(CursorIcon::Hand),
            CursorType::IBeam => Some(CursorIcon::Text),
            CursorType::Wait => Some(CursorIcon::Wait),
            CursorType::Help => Some(CursorIcon::Help),
            CursorType::EastResize => Some(CursorIcon::EResize),
            CursorType::NorthResize => Some(CursorIcon::NResize),
            CursorType::NorthEastResize => Some(CursorIcon::NeResize),
            CursorType::NorthWestResize => Some(CursorIcon::NwResize),
            CursorType::SouthResize => Some(CursorIcon::SResize),
            CursorType::SouthEastResize => Some(CursorIcon::SeResize),
            CursorType::SouthWestResize => Some(CursorIcon::SwResize),
            CursorType::WestResize => Some(CursorIcon::WResize),
            CursorType::NorthSouthResize => Some(CursorIcon::NsResize),
            CursorType::EastWestResize => Some(CursorIcon::EwResize),
            CursorType::NorthEastSouthWestResize => Some(CursorIcon::NeswResize),
            CursorType::NorthWestSouthEastResize => Some(CursorIcon::NwseResize),
            CursorType::ColumnResize => Some(CursorIcon::ColResize,),
            CursorType::RowResize => Some(CursorIcon::RowResize),
            CursorType::Move => Some(CursorIcon::Move),
            CursorType::VerticalText => Some(CursorIcon::VerticalText),
            CursorType::Cell => Some(CursorIcon::Cell),
            CursorType::ContextMenu => Some(CursorIcon::ContextMenu),
            CursorType::Alias => Some(CursorIcon::Alias),
            CursorType::Progress => Some(CursorIcon::Progress),
            CursorType::NoDrop => Some(CursorIcon::NoDrop),
            CursorType::Copy => Some(CursorIcon::Copy),
            CursorType::None => None,
            CursorType::NotAllowed => Some(CursorIcon::NotAllowed),
            CursorType::ZoomIn => Some(CursorIcon::ZoomIn),
            CursorType::ZoomOut => Some(CursorIcon::ZoomOut),
            CursorType::Grab => Some(CursorIcon::Grab),
            CursorType::Grabbing => Some(CursorIcon::Grabbing),
        };
        match winit_cursor {
            Some(cursor) => {
                self.window.set_cursor_icon(cursor);
                self.window.set_cursor_visible(true);
            },
            None => self.window.set_cursor_visible(false),
        }
    }
    fn update_drag_cursor(&self, browser: Browser, operation: DragOperation) {

    }
}

fn main() {
    let event_loop = EventLoop::with_user_event();

    let app = App::new(AppCallbacksImpl {
        browser_process_handler: BrowserProcessHandler::new(BrowserProcessHandlerCallbacksImpl {
            proxy: Mutex::new(event_loop.create_proxy()),
        })
    });
    #[cfg(windows)]
    cef::enable_highdpi_support();
    let args = MainArgs::new();
    let result = cef::execute_process(&args, Some(app.clone()), None);
    if result >= 0 {
        std::process::exit(result);
    }
    let mut settings = Settings::new();
    settings.enable_windowless_rendering();
    settings.set_log_severity(LogSeverity::Verbose);
    settings.disable_sandbox();
    let resources_folder = std::path::Path::new("./Resources").canonicalize().unwrap();
    settings.set_resources_dir_path(&resources_folder);

    cef::initialize(&args, &settings, Some(app), None);

    let window = WindowBuilder::new()
        .with_title("CEF Example Window")
        .build(&event_loop)
        .unwrap();

    let width = window.inner_size().to_physical(window.hidpi_factor()).width.round() as u32;
    let height = window.inner_size().to_physical(window.hidpi_factor()).height.round() as u32;

    let window_info = WindowInfo {
        windowless_rendering_enabled: true,
        parent_window: window.hwnd() as _,
        width: width as _,
        height: height as _,
        ..WindowInfo::new()
    };

    let browser_settings = BrowserSettings::new();

    let client = Client::new(ClientCallbacksImpl {
        life_span_handler: LifeSpanHandler::new(LifeSpanHandlerImpl {
            proxy: Mutex::new(event_loop.create_proxy()),
        }),
        render_handler: RenderHandler::new(RenderHandlerCallbacksImpl {
            pixel_buffer: Mutex::new(PixelBufferTyped::new_supported(width, height, &window)),
            window,
            popup_rect: Mutex::new(None),
        })
    });


    let browser = BrowserHost::create_browser_sync(
        &window_info,
        client,
        "https://www.github.com/anlumo/cef",
        &browser_settings,
        None,
        None,
    );

    println!("initialize done");

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::NewEvents(StartCause::ResumeTimeReached{..}) => {
                *control_flow = ControlFlow::Wait;
                cef::do_message_loop_work();
            }
            Event::WindowEvent {
                event,
                window_id: _,
            } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::RedrawRequested =>  {
                    browser.get_host().invalidate(PaintElementType::View);
                },
                _ => (),
            },
            Event::UserEvent(event) => match event {
                CefEvent::ScheduleWork(instant) => {
                    *control_flow = ControlFlow::WaitUntil(instant);
                }
            }
            Event::LoopDestroyed => {
                cef::shutdown();
            },
            _ => *control_flow = ControlFlow::Wait,
        }
    });
}
