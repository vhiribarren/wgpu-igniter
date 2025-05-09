/*
MIT License

Copyright (c) 2021, 2022, 2024, 2025 Vincent Hiribarren

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use crate::LaunchContext;
use crate::draw_context::{self, Dimensions, DrawContext};
use crate::plugins::PluginRegistry;
use crate::render_loop::{RenderLoopBuilder, RenderLoopHandler, TimeInfo};
use log::{debug, info};
use std::sync::Arc;
use web_time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{CursorIcon, Window, WindowId};

#[cfg(target_arch = "wasm32")]
const WEBAPP_CANVAS_ID: &str = "target";

const TARGET_DRAW_FPS: f64 = 60.0;
const TARGET_FPS_DISPLAY_PERIOD: Duration = Duration::from_secs(1);

struct MouseState {
    is_cursor_inside: bool,
    mouse_rotation_enabled: bool,
}

impl MouseState {
    fn new() -> Self {
        Self {
            is_cursor_inside: false,
            mouse_rotation_enabled: false,
        }
    }
    fn left_button_action(&mut self, action: ElementState, window: &Window) {
        if !self.is_cursor_inside {
            return;
        }
        match action {
            ElementState::Pressed => {
                self.mouse_rotation_enabled = true;
                // FIXME disabled due to winit error when resizing in web context: already borrowed: BorrowMutError on window.set_cursor
                #[cfg(not(target_arch = "wasm32"))]
                window.set_cursor_visible(false);
            }
            ElementState::Released => {
                self.mouse_rotation_enabled = false;
                // FIXME disabled due to winit error when resizing in web context: already borrowed: BorrowMutError on window.set_cursor
                #[cfg(not(target_arch = "wasm32"))]
                window.set_cursor_visible(true);
            }
        }
    }

    fn resize_action(&mut self, window: &Window) {
        self.mouse_rotation_enabled = false;
        // FIXME disabled due to winit error when resizing in web context: already borrowed: BorrowMutError on window.set_cursor
        #[cfg(not(target_arch = "wasm32"))]
        window.set_cursor_visible(true);
    }

    fn is_mouse_rotation_enabled(&self) -> bool {
        self.mouse_rotation_enabled
    }

    fn move_action(&mut self) {
        self.mouse_rotation_enabled = false;
    }
}

struct App {
    window: Arc<Window>,
    mouse_state: MouseState,
    scenario_start: Instant,
    last_draw_instant: Instant,
    last_fps_instant: Instant,
    draw_period_target: Duration,
    draw_context: DrawContext,
    scenario: Box<dyn RenderLoopHandler>,
    plugin_registry: PluginRegistry,
}

impl App {
    async fn async_new(
        window: Window,
        dimensions: Option<Dimensions>,
        builder: Box<RenderLoopBuilder>,
    ) -> Self {
        let window = Arc::new(window);
        let mouse_state = MouseState::new();
        let scenario_start = Instant::now();
        let last_draw_instant = scenario_start;
        let last_fps_instant = scenario_start;
        let draw_period_target = Duration::from_secs_f64(1.0 / TARGET_DRAW_FPS);
        let mut draw_context =
            draw_context::DrawContext::new(Some(Arc::clone(&window)), dimensions)
                .await
                .unwrap();

        let mut plugin_registry = PluginRegistry::default();
        let mut scenario = builder(LaunchContext {
            draw_context: &mut draw_context,
            plugin_registry: &mut plugin_registry,
        });
        scenario.on_init(&mut plugin_registry, &mut draw_context);
        Self {
            window,
            mouse_state,
            scenario_start,
            last_draw_instant,
            last_fps_instant,
            draw_period_target,
            draw_context,
            scenario,
            plugin_registry,
        }
    }
}

struct AppHandlerState {
    builder: Option<Box<RenderLoopBuilder>>,
    state: Option<App>,
    event_loop_proxy: Option<EventLoopProxy<App>>,
}

impl AppHandlerState {
    fn new(event_loop: &EventLoop<App>, builder: Box<RenderLoopBuilder>) -> Self {
        Self {
            builder: Some(builder),
            state: None,
            event_loop_proxy: Some(event_loop.create_proxy()),
        }
    }
}

impl ApplicationHandler<App> for AppHandlerState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();
        #[allow(unused_mut)]
        let mut dimensions = None;
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::dpi::PhysicalSize;
            use winit::platform::web::WindowAttributesExtWebSys;
            let dom_window = web_sys::window().unwrap();
            let dom_document = dom_window.document().unwrap();
            let dom_canvas = dom_document.get_element_by_id(WEBAPP_CANVAS_ID).unwrap();
            let canvas = dom_canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
            let width = dom_window.inner_width().unwrap().as_f64().unwrap() as u32;
            let height = dom_window.inner_height().unwrap().as_f64().unwrap() as u32;
            dimensions.replace(Dimensions { width, height });
            // FIXME winit window has size of 0 at startup, so also passing dimensions to draw context
            window_attributes = window_attributes
                .with_canvas(Some(canvas))
                .with_inner_size(PhysicalSize::new(width, height));
        }
        let window = event_loop.create_window(window_attributes).unwrap();
        window.set_cursor(CursorIcon::Grab);
        let app_future = App::async_new(window, dimensions, self.builder.take().unwrap());
        let event_loop_proxy = self.event_loop_proxy.take().unwrap();
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let app = app_future.await;
                assert!(event_loop_proxy.send_event(app).is_ok());
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use pollster::FutureExt;
            let app = app_future.block_on();
            assert!(event_loop_proxy.send_event(app).is_ok());
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: App) {
        self.state = Some(event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(ref mut app) = self.state else {
            return;
        };
        // TODO Refactor
        for listener in app.plugin_registry.iter_mut_rev() {
            let event_response = listener.on_window_event(&event);
            if event_response.processed {
                return;
            }
        }
        let event_response = app.scenario.on_window_event(&event);
        if event_response.processed {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                debug!("Closing app");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                debug!("Window is resizing");
                app.mouse_state.resize_action(&app.window);
                app.draw_context
                    .resize(physical_size.width, physical_size.height);
            }
            WindowEvent::KeyboardInput { ref event, .. } => {
                debug!("Key pressed {:?}", event.physical_key);
                for listenr in app.plugin_registry.iter_mut_rev() {
                    listenr.on_keyboard_event(event);
                }
                app.scenario.on_keyboard_event(event);
            }
            WindowEvent::Moved { .. } => {
                debug!("Window moved");
                app.mouse_state.move_action();
            }
            WindowEvent::CursorEntered { .. } => {
                app.mouse_state.is_cursor_inside = true;
            }
            WindowEvent::CursorLeft { .. } => {
                app.mouse_state.is_cursor_inside = false;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                // Works with WASM and browser canvas
                if button == MouseButton::Left {
                    app.mouse_state
                        .left_button_action(state, app.window.as_ref());
                }
            }
            WindowEvent::RedrawRequested => {
                if app.scenario.is_finished() {
                    info!("Scenario is finished, stopping");
                    event_loop.exit();
                    return;
                }
                let draw_delta = app.last_draw_instant.elapsed();
                app.last_draw_instant = Instant::now();
                if app.last_fps_instant.elapsed() >= TARGET_FPS_DISPLAY_PERIOD {
                    info!("FPS: {}", (1.0 / draw_delta.as_secs_f64()).round());
                    app.last_fps_instant = app.last_draw_instant;
                }

                let plugin_registry = &mut app.plugin_registry;
                let time_info = &TimeInfo {
                    init_start: app.scenario_start,
                    processing_delta: draw_delta,
                    _private: (),
                };
                app.scenario
                    .on_update(plugin_registry, &mut app.draw_context, time_info);
                for listener in plugin_registry.iter_mut() {
                    listener.on_update(&app.draw_context, time_info);
                }
                app.draw_context
                    .render_scene(|render_pass| {
                        let rpass = &mut render_pass.forget_lifetime();
                        app.scenario.on_render(
                            plugin_registry,
                            &app.draw_context,
                            time_info,
                            rpass,
                        );
                        for listener in plugin_registry.iter_mut() {
                            listener.on_render(&app.draw_context, time_info, rpass);
                        }
                    })
                    .unwrap();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let Some(ref mut app) = self.state else {
            return;
        };
        if let DeviceEvent::Button { button, state } = event {
            // Works with MacOS
            if button == 0 {
                app.mouse_state
                    .left_button_action(state, app.window.as_ref());
            }
        }
        if app.mouse_state.is_mouse_rotation_enabled() {
            for listener in app.plugin_registry.iter_mut_rev() {
                let event_state = listener.on_mouse_event(&event);
                if event_state.processed {
                    return;
                }
            }
            app.scenario.on_mouse_event(&event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(ref mut app) = self.state else {
            return;
        };
        let since_last_draw = app.last_draw_instant.elapsed();
        if since_last_draw >= app.draw_period_target {
            app.window.as_ref().request_redraw();
            event_loop.set_control_flow(ControlFlow::Poll);
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now()
                    + app
                        .draw_period_target
                        .checked_sub(since_last_draw)
                        .expect("Substraction of a Duration from an Instant should not underflow"),
            ));
        }
    }
}

pub(crate) fn init_event_loop(builder: Box<RenderLoopBuilder>) {
    let event_loop = EventLoop::with_user_event().build().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let app_handler_state = &mut AppHandlerState::new(&event_loop, builder);
    event_loop.run_app(app_handler_state).unwrap();
}
