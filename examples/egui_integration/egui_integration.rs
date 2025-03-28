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
use winit::event::DeviceId;
use winit::window::Window;

const CANVAS_STATIC_SHADER: &str = include_str!("./egui_integration.wgsl");

pub struct MainScenario {
    canvas: Drawable,
    time_uniform: Uniform<f32>,
    egui_renderer: egui_wgpu::Renderer,
    egui_state: egui_winit::State,
    window: Arc<Window>,
}

impl MainScenario {
    pub fn new(draw_context: &DrawContext) -> Self {
        let window = Arc::clone(draw_context.window.as_ref().unwrap());
        let egui_state = egui_winit::State::new(
            egui::Context::default(),
            egui::ViewportId::default(),
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &draw_context.device,
            draw_context.surface_config.format,
            Some(draw_context.depth_texture.format()),
            draw_context.multisample_config.get_multisample_count(),
            true,
        );
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
            egui_renderer,
            egui_state,
            window,
        }
    }

    pub fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.egui_state.take_egui_input(window);
        self.egui_state.egui_ctx().begin_pass(raw_input);
    }

    pub fn end_frame_and_draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        screen_descriptor: egui_wgpu::ScreenDescriptor,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        // TODO We must call begin_frame before calling end_frame_and_draw
        self.egui_state
            .egui_ctx()
            .set_pixels_per_point(screen_descriptor.pixels_per_point);
        let full_output = self.egui_state.egui_ctx().end_pass();

        self.egui_state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self.egui_state.egui_ctx().tessellate(
            full_output.shapes,
            self.egui_state.egui_ctx().pixels_per_point(),
        );
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(device, queue, *id, image_delta);
        }
        self.egui_renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        self.egui_renderer
            .render(render_pass, &tris, &screen_descriptor);
        for x in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(x)
        }
    }
}

impl WinitEventLoopHandler for MainScenario {
    fn on_window_event(&mut self, event: &winit::event::WindowEvent) {
        let _ = self.egui_state.on_window_event(&self.window, event);
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

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                draw_context.surface_config.width,
                draw_context.surface_config.height,
            ],
            pixels_per_point: 2.,
        };

        {
            let mut encoder = draw_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            let window = Arc::clone(&self.window);
            self.begin_frame(&window);
            let egui_context = self.egui_state.egui_ctx();

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
                            egui::ScrollArea::vertical().show(ui, |ui| {});
                        });

                    egui::SidePanel::right("right_panel")
                        .resizable(true)
                        .default_width(150.0)
                        .width_range(80.0..=200.0)
                        .show_inside(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading("Right Panel");
                            });
                            egui::ScrollArea::vertical().show(ui, |ui| {});
                        });

                    egui::TopBottomPanel::bottom("bottom_panel")
                        .resizable(false)
                        .min_height(0.0)
                        .show_inside(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading("Bottom Panel");
                            });
                            ui.vertical_centered(|ui| {});
                        });

                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Central Panel");
                        });
                        egui::ScrollArea::vertical().show(ui, |ui| {});
                    });
                });

            let window = Arc::clone(&self.window);
            self.end_frame_and_draw(
                &draw_context.device,
                &draw_context.queue,
                &mut encoder,
                &window,
                screen_descriptor,
                &mut rpass,
            );
        }
    }
}
