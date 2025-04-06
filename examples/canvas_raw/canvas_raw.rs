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

use wgpu_lite_wrapper::draw_context::{
    DrawContext, DrawModeParams, Drawable, DrawableBuilder, Uniform,
};
use wgpu_lite_wrapper::scenario::{UpdateContext, WinitEventLoopHandler};

const CANVAS_STATIC_SHADER: &str = include_str!("./canvas_raw.wgsl");

pub struct MainScenario {
    canvas: Drawable,
    time_uniform: Uniform<f32>,
}

impl MainScenario {
    pub fn new(draw_context: &DrawContext) -> Self {
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
        }
    }
}

impl WinitEventLoopHandler for MainScenario {
    fn on_update(&mut self, update_context: &UpdateContext) {
        let &UpdateContext { update_interval } = update_context;
        self.time_uniform
            .write_uniform(update_interval.scenario_start.elapsed().as_secs_f32());
    }
    fn on_render<'a>(
        &'a mut self,
        _draw_context: &DrawContext,
        mut render_pass: wgpu::RenderPass<'a>,
    ) {
        self.canvas.render(&mut render_pass);
    }
}
