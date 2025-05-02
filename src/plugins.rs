/*
MIT License

Copyright (c) 2025 Vincent Hiribarren

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

use std::any::{Any, TypeId};

use indexmap::IndexMap;
use winit::event::{DeviceEvent, KeyEvent, WindowEvent};

use crate::{DrawContext, EventState, TimeInfo};

#[cfg(feature = "egui")]
pub mod egui;
pub mod scene_3d;

pub trait Plugin: Any {
    fn on_mouse_event(&mut self, _event: &DeviceEvent) -> EventState {
        EventState::default()
    }
    fn on_keyboard_event(&mut self, _event: &KeyEvent) {}
    fn on_window_event(&mut self, _event: &WindowEvent) -> EventState {
        EventState::default()
    }
    fn on_render(
        &mut self,
        draw_context: &DrawContext,
        time_info: &TimeInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
    );
}

impl dyn Plugin {
    fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }
    fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        (self as &mut dyn Any).downcast_mut::<T>()
    }
}

#[derive(Default)]
pub struct PluginRegistry {
    plugins: IndexMap<TypeId, Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn register<T: Plugin + 'static>(&mut self, plugin: T) {
        self.plugins.insert(TypeId::of::<T>(), Box::new(plugin));
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn Plugin>> {
        self.plugins.values_mut()
    }
    pub fn iter_mut_rev(&mut self) -> impl Iterator<Item = &mut Box<dyn Plugin>> {
        self.plugins.values_mut().rev()
    }
    #[must_use]
    pub fn get<T: Plugin + 'static>(&self) -> Option<&T> {
        self.plugins
            .get(&TypeId::of::<T>())
            .and_then(|plugin| plugin.as_ref().downcast_ref::<T>())
    }
    // TODO will need to allow multiple borrows at the same time
    #[must_use]
    pub fn get_mut<T: Plugin + 'static>(&mut self) -> Option<&mut T> {
        self.plugins
            .get_mut(&TypeId::of::<T>())
            .and_then(|plugin| plugin.as_mut().downcast_mut::<T>())
    }
}
