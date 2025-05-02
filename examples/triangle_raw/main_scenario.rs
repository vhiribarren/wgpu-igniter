/*
MIT License

Copyright (c) 2021, 2022, 2024, 2025 Vincent Hiribarren

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
use wgpu_igniter::primitives::triangle::{
    TRIANGLE_COLOR, TRIANGLE_GEOMETRY, TRIANGLE_VERTEX_COUNT,
};
use wgpu_igniter::{
    DrawContext, DrawModeParams, Drawable, DrawableBuilder, LaunchContext, RenderLoopHandler,
    TimeInfo,
};

const DEFAULT_SHADER: &str = include_str!("./triangle_raw.wgsl");

pub struct MainScenario {
    triangle: Drawable,
}

impl MainScenario {
    pub fn new(LaunchContext { draw_context, .. }: LaunchContext) -> Self {
        let shader_module = draw_context.create_shader_module(DEFAULT_SHADER);
        let mut drawable_builder = DrawableBuilder::new(
            draw_context,
            &shader_module,
            &shader_module,
            DrawModeParams::Direct {
                vertex_count: TRIANGLE_VERTEX_COUNT,
            },
        );
        drawable_builder
            .add_attribute(
                0,
                wgpu::VertexStepMode::Vertex,
                TRIANGLE_GEOMETRY,
                wgpu::VertexFormat::Float32x3,
            )
            .expect("Location should be different than for another attribute.")
            .add_attribute(
                1,
                wgpu::VertexStepMode::Vertex,
                TRIANGLE_COLOR,
                wgpu::VertexFormat::Float32x3,
            )
            .expect("Location should be different than for another attribute.");
        let triangle = drawable_builder.build();
        Self { triangle }
    }
}

impl RenderLoopHandler for MainScenario {
    fn on_render(
        &mut self,
        _plugin_registry: &mut PluginRegistry,
        _draw_context: &DrawContext,
        _time_info: &TimeInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        self.triangle.render(render_pass);
    }
}
