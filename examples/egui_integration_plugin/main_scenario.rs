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

use wgpu_igniter::plugins::PluginRegistry;
use wgpu_igniter::plugins::canvas::CanvasPlugin;
use wgpu_igniter::plugins::egui::EquiPlugin;
use wgpu_igniter::{DrawContext, LaunchContext, RenderLoopHandler, TimeInfo};

const FRAGMENT_SHADER: &str = include_str!("./fragment_shader.wgsl");

struct GuiState {
    pub anim_speed: f32,
    pixels_per_point: f32,
}

pub struct MainScenario {
    gui_state: GuiState,
}

impl MainScenario {
    pub fn new(
        LaunchContext {
            draw_context,
            plugin_registry,
        }: LaunchContext,
    ) -> Self {
        let gui_state = GuiState {
            pixels_per_point: 1.0,
            anim_speed: 1.0,
        };
        let canvas = CanvasPlugin::new(
            &draw_context,
            &draw_context.create_shader_module(FRAGMENT_SHADER),
        );
        plugin_registry.register(canvas);
        plugin_registry.register(EquiPlugin::new(draw_context));

        Self { gui_state }
    }
    fn generate_egui(state: &mut GuiState, egui_context: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(egui_context, |ui| {
            ui.label("Egui Integration Example");
        });
        egui::Window::new("Animation Control").show(egui_context, |ui| {
            ui.label("Adjust the animation speed:");
            ui.add(egui::Slider::new(&mut state.anim_speed, 0.1..=5.0).text("Speed"));
            ui.add(egui::DragValue::new(&mut state.pixels_per_point).range(0.5..=2.0));
            ui.label("Pixels per point");
        });
    }
}

impl RenderLoopHandler for MainScenario {
    fn on_update(
        &mut self,
        plugin_registry: &mut PluginRegistry,
        _draw_context: &mut DrawContext,
        _time_info: &TimeInfo,
    ) {
        //FIXME Take into account speed info, by adding a uniform to the canvas
        let egui_support = plugin_registry
            .get_mut::<EquiPlugin>()
            .expect("EguiSupport should be registered");
        egui_support.set_pixels_per_point(self.gui_state.pixels_per_point);
        egui_support.draw(|egui_context| Self::generate_egui(&mut self.gui_state, egui_context));
    }
}
