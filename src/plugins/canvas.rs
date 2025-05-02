use wgpu::ShaderModule;

use crate::{DrawContext, DrawModeParams, Drawable, DrawableBuilder, TimeInfo, Uniform};

use super::Plugin;

const CANVAS_STATIC_SHADER: &str = include_str!("./canvas.wgsl");

pub struct CanvasPlugin {
    canvas: Drawable,
    time_uniform: Uniform<f32>,
}

// TODO Make it possible to add uniforms or buffers
impl CanvasPlugin {
    pub fn new(draw_context: &DrawContext, fragment_shader: &ShaderModule) -> Self {
        let time_uniform = Uniform::new(draw_context, 0f32);
        let shader_module = &draw_context.create_shader_module(CANVAS_STATIC_SHADER);
        let mut drawable_builder = DrawableBuilder::new(
            draw_context,
            shader_module,
            fragment_shader,
            DrawModeParams::Direct { vertex_count: 3 },
        );
        drawable_builder
            .add_uniform(0, 0, &time_uniform)
            .expect("Bind group or binding should be different from other uniforms");
        let canvas = drawable_builder.build();
        Self {
            canvas,
            time_uniform,
        }
    }
}

impl Plugin for CanvasPlugin {
    fn on_render(
        &mut self,
        _draw_context: &DrawContext,
        time_info: &TimeInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        self.time_uniform
            .write_uniform(time_info.init_start.elapsed().as_secs_f32());
        self.canvas.render(render_pass);
    }
}
