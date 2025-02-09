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

use cgmath::SquareMatrix;
use wgpu_lite_wrapper::draw_context::{
    DrawContext, DrawModeParams, Drawable, DrawableBuilder, Uniform,
};
use wgpu_lite_wrapper::primitives::triangle::{
    TRIANGLE_COLOR, TRIANGLE_GEOMETRY, TRIANGLE_VERTEX_COUNT,
};
use wgpu_lite_wrapper::scenario::{UpdateContext, WinitEventLoopHandler};

const DEFAULT_SHADER: &str = include_str!("./triangle_direct.wgsl");

const ROTATION_DEG_PER_S: f32 = 45.0;

pub struct MainScenario {
    triangle: Drawable,
    transform_uniform: Uniform<[[f32; 4]; 4]>,
}

impl MainScenario {
    pub fn new(draw_context: &DrawContext) -> Self {
        let shader_module = draw_context.create_shader_module(DEFAULT_SHADER);
        let transform_uniform = Uniform::new(draw_context, cgmath::Matrix4::identity().into());
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
            .unwrap()
            .add_attribute(
                1,
                wgpu::VertexStepMode::Vertex,
                TRIANGLE_COLOR,
                wgpu::VertexFormat::Float32x3,
            )
            .unwrap()
            .add_uniform(0, 0, &transform_uniform)
            .unwrap();
        let triangle = drawable_builder.build();
        Self {
            triangle,
            transform_uniform,
        }
    }
}

impl WinitEventLoopHandler for MainScenario {
    fn on_update(&mut self, update_context: &UpdateContext) {
        let total_seconds = update_context
            .update_interval
            .scenario_start
            .elapsed()
            .as_secs_f32();
        let new_rotation = ROTATION_DEG_PER_S * total_seconds;
        let transform: cgmath::Matrix4<f32> = cgmath::Matrix4::from_scale(0.5)
            * cgmath::Matrix4::from_angle_z(cgmath::Deg(new_rotation));
        self.transform_uniform
            .write_uniform(update_context.draw_context, transform.into());
    }
    fn on_render<'drawable>(&'drawable self, render_pass: &mut wgpu::RenderPass<'drawable>) {
        self.triangle.render(render_pass);
    }

    fn on_mouse_event(&mut self, _event: &winit::event::DeviceEvent) {}

    fn on_keyboard_event(&mut self, _event: &winit::event::KeyEvent) {}
}
