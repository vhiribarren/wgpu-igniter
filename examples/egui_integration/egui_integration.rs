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

use std::sync::Arc;

use wgpu_lite_wrapper::draw_context::{
    DrawContext, DrawModeParams, Drawable, DrawableBuilder, Uniform,
};
use wgpu_lite_wrapper::scenario::{UpdateContext, WinitEventLoopHandler};
use wgpu_lite_wrapper::support::egui::EguiSupport;
use winit::window::Window;

const CANVAS_STATIC_SHADER: &str = include_str!("./egui_integration.wgsl");

pub struct MainScenario {
    canvas: Drawable,
    time_uniform: Uniform<f32>,
    egui_support: EguiSupport,
    window: Arc<Window>,
}

impl MainScenario {
    pub fn new(draw_context: &DrawContext) -> Self {
        let window = Arc::clone(draw_context.window.as_ref().unwrap());
        let egui_support = EguiSupport::new(draw_context, Arc::clone(&window));
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
            egui_support,
            window,
        }
    }
    fn generate_egui(egui_context: &egui::Context) {
        egui::Window::new("winit + egui + wgpu says hello!")
            .resizable(true)
            .vscroll(true)
            .default_open(true)
            .show(egui_context, |ui| {
                egui::TopBottomPanel::top("top_panel")
                    .resizable(true)
                    .min_height(32.0)
                    .show_inside(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading("Expandable Upper Panel");
                            });
                        });
                    });
                egui::SidePanel::left("left_panel")
                    .resizable(true)
                    .default_width(150.0)
                    .width_range(80.0..=200.0)
                    .show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Left Panel");
                        });
                        egui::ScrollArea::vertical().show(ui, |_ui| {});
                    });
                egui::SidePanel::right("right_panel")
                    .resizable(true)
                    .default_width(150.0)
                    .width_range(80.0..=200.0)
                    .show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Right Panel");
                        });
                        egui::ScrollArea::vertical().show(ui, |_ui| {});
                    });
                egui::TopBottomPanel::bottom("bottom_panel")
                    .resizable(false)
                    .min_height(0.0)
                    .show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Bottom Panel");
                        });
                        ui.vertical_centered(|_ui| {});
                    });
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Central Panel");
                    });
                    egui::ScrollArea::vertical().show(ui, |_ui| {});
                });
            });
    }
}

impl WinitEventLoopHandler for MainScenario {
    fn on_window_event(&mut self, event: &winit::event::WindowEvent) {
        let _ = self.egui_support.on_window_event(&self.window, event);
    }
    fn on_update(&mut self, update_context: &UpdateContext) {
        let &UpdateContext { update_interval } = update_context;
        self.time_uniform
            .write_uniform(update_interval.scenario_start.elapsed().as_secs_f32());
    }
    fn on_render<'drawable>(
        &'drawable mut self,
        draw_context: &DrawContext,
        render_pass: wgpu::RenderPass<'drawable>,
    ) {
        let mut rpass = render_pass.forget_lifetime();
        self.canvas.render(&mut rpass);
        self.egui_support
            .draw(draw_context, rpass, Self::generate_egui);
    }
}
