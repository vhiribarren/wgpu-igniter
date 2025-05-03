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

use wgpu_igniter::{
    BindingSlot, DrawContext, DrawModeParams, Drawable, DrawableBuilder, LaunchContext,
    RenderLoopHandler, TimeInfo, Uniform, plugins::PluginRegistry,
};

const CANVAS_STATIC_SHADER: &str = include_str!("./canvas_raw.wgsl");

pub struct MainScenario {
    canvas: Drawable,
    time_uniform: Uniform<f32>,
}

impl MainScenario {
    pub fn new(LaunchContext { draw_context, .. }: LaunchContext) -> Self {
        let time_uniform = Uniform::new(draw_context, 0f32);
        let shader_module = draw_context.create_shader_module(CANVAS_STATIC_SHADER);
        let mut drawable_builder = DrawableBuilder::new(
            draw_context,
            &shader_module,
            &shader_module,
            DrawModeParams::Direct { vertex_count: 3 },
        );
        drawable_builder
            .add_binding_slot(BindingSlot {
                bind_group: 0,
                binding: 0,
                resource: &time_uniform,
            })
            .expect("Bind group or binding should be different from other uniforms");
        let canvas = drawable_builder.build();
        Self {
            canvas,
            time_uniform,
        }
    }
}

impl RenderLoopHandler for MainScenario {
    fn on_render(
        &mut self,
        _plugin_registry: &mut PluginRegistry,
        _draw_context: &DrawContext,
        time_info: &TimeInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        self.time_uniform
            .write_uniform(time_info.init_start.elapsed().as_secs_f32());
        self.canvas.render(render_pass);
    }
}
