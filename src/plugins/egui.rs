use std::sync::Arc;

use crate::{EventState, draw_context::DrawContext};
use winit::window::Window;

use super::Plugin;

pub enum EguiSupport {
    NoWindow(egui::Context),
    WithWindow(EguiSupportWithWindow),
}

pub struct EguiSupportWithWindow {
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    pixels_per_point: f32,
    window: Arc<Window>,
}

impl EguiSupport {
    pub fn new(draw_context: &DrawContext) -> Self {
        let Some(window) = draw_context.window.as_ref() else {
            return Self::NoWindow(egui::Context::default());
        };
        let window = Arc::clone(window);
        let egui_state = egui_winit::State::new(
            egui::Context::default(),
            egui::ViewportId::default(),
            &window,
            None,
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
        Self::WithWindow(EguiSupportWithWindow {
            egui_state,
            egui_renderer,
            #[allow(clippy::cast_possible_truncation)]
            pixels_per_point: window.scale_factor() as f32,
            window,
        })
    }
    pub fn set_pixels_per_point(&mut self, pixels_per_point: f32) {
        match self {
            Self::WithWindow(egui_support) => {
                egui_support.pixels_per_point = pixels_per_point;
            }
            Self::NoWindow(_) => {}
        }
    }
    #[must_use]
    pub fn get_pixels_per_point(&self) -> f32 {
        match self {
            Self::WithWindow(egui_support) => egui_support.pixels_per_point,
            Self::NoWindow(_) => 1.0,
        }
    }
    #[must_use]
    pub fn egui_context(&self) -> &egui::Context {
        match self {
            Self::NoWindow(ctx) => ctx,
            Self::WithWindow(egui_support) => egui_support.egui_state.egui_ctx(),
        }
    }

    pub fn draw<F>(&mut self, run_ui: F)
    where
        F: FnOnce(&egui::Context),
    {
        let Self::WithWindow(egui_support) = self else {
            return;
        };
        Self::begin_frame(egui_support);
        let egui_context = egui_support.egui_state.egui_ctx();
        run_ui(egui_context);
    }

    fn begin_frame(egui_support: &mut EguiSupportWithWindow) {
        let raw_input = egui_support
            .egui_state
            .take_egui_input(&egui_support.window);
        egui_support.egui_state.egui_ctx().begin_pass(raw_input);
    }

    fn end_frame_and_draw(
        egui_support: &mut EguiSupportWithWindow,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        // TODO We must call begin_frame before calling end_frame_and_draw, otherwise panic
        egui_support
            .egui_state
            .egui_ctx()
            .set_pixels_per_point(screen_descriptor.pixels_per_point);
        let full_output = egui_support.egui_state.egui_ctx().end_pass();

        egui_support
            .egui_state
            .handle_platform_output(&egui_support.window, full_output.platform_output);

        let tris = egui_support.egui_state.egui_ctx().tessellate(
            full_output.shapes,
            egui_support.egui_state.egui_ctx().pixels_per_point(),
        );
        for (id, image_delta) in &full_output.textures_delta.set {
            egui_support
                .egui_renderer
                .update_texture(device, queue, *id, image_delta);
        }
        egui_support
            .egui_renderer
            .update_buffers(device, queue, encoder, &tris, screen_descriptor);

        egui_support
            .egui_renderer
            .render(render_pass, &tris, screen_descriptor);
        for x in &full_output.textures_delta.free {
            egui_support.egui_renderer.free_texture(x);
        }
    }
}

impl Plugin for EguiSupport {
    fn on_window_event(&mut self, event: &winit::event::WindowEvent) -> EventState {
        match self {
            Self::WithWindow(egui_support) => {
                let event_response = egui_support
                    .egui_state
                    .on_window_event(&egui_support.window, event);
                EventState {
                    processed: event_response.consumed,
                }
            }
            Self::NoWindow(_) => EventState::default(),
        }
    }
    fn on_render(
        &mut self,
        render_context: &crate::RenderContext,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        let Self::WithWindow(egui_support) = self else {
            return;
        };
        let draw_context = render_context.draw_context;
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                draw_context.surface_config.width,
                draw_context.surface_config.height,
            ],
            pixels_per_point: egui_support.pixels_per_point,
        };
        let mut encoder = draw_context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        Self::end_frame_and_draw(
            egui_support,
            &draw_context.device,
            &draw_context.queue,
            &screen_descriptor,
            &mut encoder,
            render_pass,
        );
    }
}
