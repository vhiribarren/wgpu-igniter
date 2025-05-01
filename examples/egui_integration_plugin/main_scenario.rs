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
use wgpu_igniter::plugins::egui::EquiPlugin;
use wgpu_igniter::{
    DrawModeParams, Drawable, DrawableBuilder, LaunchContext, RenderContext, RenderLoopHandler,
    Uniform,
};

const CANVAS_STATIC_SHADER: &str = include_str!("./egui_integration.wgsl");

struct GuiState {
    pub anim_speed: f32,
    pixels_per_point: f32,
}

pub struct MainScenario {
    canvas: Drawable,
    time_uniform: Uniform<f32>,
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
        plugin_registry.register(EquiPlugin::new(draw_context));
        let time_uniform = Uniform::new(draw_context, 0f32);
        let shader_module = draw_context.create_shader_module(CANVAS_STATIC_SHADER);
        let mut drawable_builder = DrawableBuilder::new(
            draw_context,
            &shader_module,
            &shader_module,
            DrawModeParams::Direct { vertex_count: 3 },
        );
        drawable_builder
            .add_uniform(0, 0, &time_uniform)
            .expect("Bind group or binding should be different from other uniforms");
        let canvas = drawable_builder.build();
        Self {
            canvas,
            time_uniform,
            gui_state,
        }
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
    fn on_render(
        &mut self,
        plugin_registry: &mut PluginRegistry,
        render_context: &RenderContext,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        let RenderContext {
            time_info: update_interval,
            ..
        } = render_context;
        let egui_support = plugin_registry
            .get_mut::<EquiPlugin>()
            .expect("EguiSupport should be registered");
        egui_support.draw(|egui_context| Self::generate_egui(&mut self.gui_state, egui_context));
        egui_support.set_pixels_per_point(self.gui_state.pixels_per_point);
        self.time_uniform.write_uniform(
            update_interval.init_start.elapsed().as_secs_f32() * self.gui_state.anim_speed,
        );
        self.canvas.render(render_pass);
    }
}
