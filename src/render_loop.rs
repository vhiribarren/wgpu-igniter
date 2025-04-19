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

use crate::draw_context::DrawContext;
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

#[allow(clippy::manual_non_exhaustive)]
pub struct RenderContext<'a> {
    pub time_info: &'a TimeInfo,
    pub draw_context: &'a DrawContext,
    pub(crate) _private: (),
}

#[derive(Default)]
pub struct EventState {
    pub processed: bool,
}

pub trait RenderLoopHandler {
    fn on_mouse_event(&mut self, _event: &DeviceEvent) {}
    fn on_keyboard_event(&mut self, _event: &KeyEvent) {}
    fn on_window_event(&mut self, _event: &WindowEvent) -> EventState {
        EventState::default()
    }
    fn on_render(&mut self, render_context: &RenderContext, render_pass: wgpu::RenderPass<'_>);
    fn is_finished(&self) -> bool {
        false
    }
}

pub type RenderLoopBuilder = dyn Fn(&mut DrawContext) -> Box<dyn RenderLoopHandler>;
