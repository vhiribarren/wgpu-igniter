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

use crate::{draw_context::DrawContext, plugins::PluginRegistry};
use web_time::{Duration, Instant};
use winit::event::{DeviceEvent, KeyEvent, WindowEvent};

#[allow(clippy::manual_non_exhaustive)]
pub struct TimeInfo {
    pub init_start: Instant,
    pub processing_delta: Duration,
    pub(crate) _private: (),
}

impl Default for TimeInfo {
    fn default() -> Self {
        Self {
            init_start: Instant::now(),
            processing_delta: Duration::new(0, 0),
            _private: (),
        }
    }
}

#[derive(Default)]
pub struct EventState {
    pub processed: bool,
}

#[allow(unused_variables)]
pub trait RenderLoopHandler {
    fn on_mouse_event(&mut self, event: &DeviceEvent) {}
    fn on_keyboard_event(&mut self, event: &KeyEvent) {}
    fn on_window_event(&mut self, event: &WindowEvent) -> EventState {
        EventState::default()
    }
    fn on_init(&mut self, plugin_registry: &mut PluginRegistry, draw_context: &mut DrawContext) {}
    fn on_update(
        &mut self,
        plugin_registry: &mut PluginRegistry,
        draw_context: &mut DrawContext,
        time_info: &TimeInfo,
    ) {
    }
    fn on_render(
        &mut self,
        plugin_registry: &mut PluginRegistry,
        draw_context: &DrawContext,
        time_info: &TimeInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
    }
    fn is_finished(&self) -> bool {
        false
    }
}

pub struct LaunchContext<'a> {
    pub draw_context: &'a mut DrawContext,
    pub plugin_registry: &'a mut PluginRegistry,
}

pub type RenderLoopBuilder = dyn Fn(LaunchContext<'_>) -> Box<dyn RenderLoopHandler> + Send;
