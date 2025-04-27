use std::any::{Any, TypeId};

use indexmap::IndexMap;

use crate::RenderLoopHandler;

#[cfg(feature = "egui")]
pub mod egui;

pub trait Plugin: RenderLoopHandler + Any {}

impl dyn Plugin {
    fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }
}


pub struct PluginRegistry {
    plugins: IndexMap<TypeId, Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: IndexMap::new(),
        }
    }

    pub fn register<T: Plugin + 'static>(&mut self, plugin: T) {
        self.plugins.insert(TypeId::of::<T>(), Box::new(plugin));
    }

    pub fn get<T: Plugin + 'static>(&self) -> Option<&T> {
        self.plugins
            .get(&TypeId::of::<T>())
            .and_then(|plugin| plugin.as_ref().downcast_ref::<T>())
    }
}
