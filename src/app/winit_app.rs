/// Common boilerplate for setting up a winit application.
/// From: https://github.com/rust-windowing/softbuffer/blob/master/examples/utils/winit_app.rs
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use winit_input_helper::WinitInputHelper;


pub struct WinitApp<S, Init, InitSurface, InputProcessor, RawWindowEventHandler, Render> {
    init: Init,
    init_surface: InitSurface,
    input_processor: InputProcessor,
    raw_window_event_handler: RawWindowEventHandler,
    render: Render,
    window: Option<Arc<Window>>,
    surface_state: Option<S>,
    winit_input_helper: WinitInputHelper,
}

impl<S, Init, InitSurface, InputProcessor, RawWindowEventHandler, Render>
    WinitApp<S, Init, InitSurface, InputProcessor, RawWindowEventHandler, Render>
where
    Init: FnMut(&ActiveEventLoop) -> Window,
    InitSurface: FnMut(&ActiveEventLoop, Arc<Window>) -> S,
    InputProcessor: FnMut(Arc<Window>, &mut S, &mut WinitInputHelper, &ActiveEventLoop),
    RawWindowEventHandler: FnMut(&mut S, &Window, &WindowEvent) -> bool,
    Render: FnMut(Arc<Window>, &mut S, &ActiveEventLoop),
{
    pub fn new(
        init: Init,
        init_surface: InitSurface,
        input_processor: InputProcessor,
        raw_window_event_handler: RawWindowEventHandler,
        render: Render,
    ) -> Self {
        Self {
            init,
            init_surface,
            input_processor,
            raw_window_event_handler,
            render,
            window: None,
            surface_state: None,
            winit_input_helper: WinitInputHelper::new(),
        }
    }

    pub fn run(mut self) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run_app(&mut self)
    }
}

impl<S, Init, InitSurface, InputProcessor, RawWindowEventHandler, Render> ApplicationHandler
    for WinitApp<S, Init, InitSurface, InputProcessor, RawWindowEventHandler, Render>
where
    Init: FnMut(&ActiveEventLoop) -> Window,
    InitSurface: FnMut(&ActiveEventLoop, Arc<Window>) -> S,
    InputProcessor: FnMut(Arc<Window>, &mut S, &mut WinitInputHelper, &ActiveEventLoop),
    RawWindowEventHandler: FnMut(&mut S, &Window, &WindowEvent) -> bool,
    Render: FnMut(Arc<Window>, &mut S, &ActiveEventLoop),
{
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        self.winit_input_helper.step();
    }

    fn resumed(&mut self, el: &ActiveEventLoop) {
        debug_assert!(self.window.is_none());
        let window = Arc::new((self.init)(el));
        self.surface_state = Some((self.init_surface)(el, window.clone()));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(window) = &self.window {
            if let Some(surface_state) = self.surface_state.as_mut() {
                if window.id() == window_id {
                    if (self.raw_window_event_handler)(surface_state, &window, &event) {
                        if self.winit_input_helper.process_window_event(&event) {
                            (self.render)(window.clone(), surface_state, event_loop);
                        }
                    }
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.winit_input_helper.process_device_event(&event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.winit_input_helper.end_step();

        if let Some(window) = &self.window {
            if let Some(surface_state) = self.surface_state.as_mut() {
                (self.input_processor)(
                    window.clone(),
                    surface_state,
                    &mut self.winit_input_helper,
                    event_loop,
                );
            }
            window.request_redraw();
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        let surface_state = self.surface_state.take();
        debug_assert!(surface_state.is_some());
        drop(surface_state);
    }
}
