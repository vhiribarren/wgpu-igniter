use std::sync::Arc;

use crate::draw_context::DrawContext;
use egui_winit::EventResponse;
use winit::window::Window;

pub struct EguiSupport {
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    pub pixels_per_point: f32,
    window: Arc<Window>,
}
impl EguiSupport {
    const PIXELS_PER_POINT: f32 = 1.0;
    pub fn new(draw_context: &DrawContext) -> Self {
        // TODO Case when window is not available ; mock window?
        let window = Arc::clone(draw_context.window.as_ref().unwrap());
        let egui_state = egui_winit::State::new(
            egui::Context::default(),
            egui::ViewportId::default(),
            &Arc::clone(&window),
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
        Self {
            egui_state,
            egui_renderer,
            pixels_per_point: Self::PIXELS_PER_POINT,
            window,
        }
    }
    pub fn egui_context(&self) -> &egui::Context {
        self.egui_state.egui_ctx()
    }

    pub fn on_window_event(
        &mut self,
        window: &Window,
        event: &winit::event::WindowEvent,
    ) -> EventResponse {
        self.egui_state.on_window_event(window, event)
    }

    pub fn draw<F>(
        &mut self,
        draw_context: &DrawContext,
        rpass: &mut wgpu::RenderPass<'static>,
        run_ui: F,
    ) where
        F: FnOnce(&egui::Context),
    {
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                draw_context.surface_config.width,
                draw_context.surface_config.height,
            ],
            pixels_per_point: self.pixels_per_point,
        };
        let mut encoder = draw_context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.begin_frame();
        let egui_context = self.egui_state.egui_ctx();
        run_ui(egui_context);
        self.end_frame_and_draw(
            &draw_context.device,
            &draw_context.queue,
            screen_descriptor,
            &mut encoder,
            rpass,
        );
    }

    fn begin_frame(&mut self) {
        let raw_input = self.egui_state.take_egui_input(&self.window);
        self.egui_state.egui_ctx().begin_pass(raw_input);
    }

    fn end_frame_and_draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: egui_wgpu::ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        // TODO We must call begin_frame before calling end_frame_and_draw, otherwise panic
        self.egui_state
            .egui_ctx()
            .set_pixels_per_point(screen_descriptor.pixels_per_point);
        let full_output = self.egui_state.egui_ctx().end_pass();

        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);

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
