use std::any::{Any, TypeId};

use indexmap::IndexMap;
use winit::event::{DeviceEvent, KeyEvent, WindowEvent};

use crate::{EventState, RenderContext};

#[cfg(feature = "egui")]
pub mod egui;

pub trait Plugin: Any {
    fn on_mouse_event(&mut self, _event: &DeviceEvent) {}
    fn on_keyboard_event(&mut self, _event: &KeyEvent) {}
    fn on_window_event(&mut self, _event: &WindowEvent) -> EventState {
        EventState::default()
    }
    fn on_render(
        &mut self,
        render_context: &RenderContext,
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
    #[must_use]
    pub fn get_mut<T: Plugin + 'static>(&mut self) -> Option<&mut T> {
        self.plugins
            .get_mut(&TypeId::of::<T>())
            .and_then(|plugin| plugin.as_mut().downcast_mut::<T>())
    }
}
