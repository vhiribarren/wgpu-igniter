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

use crate::draw_context::{DrawContext, Drawable, DrawableBuilder};

#[rustfmt::skip]
pub const TRIANGLE_GEOMETRY_CANVAS: &[[f32; 2]] = &[
    [-1., -1.],
    [3., -1.],
    [-1., 3.],
];

pub fn create_canvas(
    context: &DrawContext,
    vtx_module: &wgpu::ShaderModule,
    frg_module: &wgpu::ShaderModule,
    //uniforms: &[Uniform]
) -> Result<Drawable, anyhow::Error> {
    let mut drawable_builder = DrawableBuilder::new(
        context,
        vtx_module,
        frg_module,
        crate::draw_context::DrawModeParams::Direct {
            vertex_count: u32::try_from(TRIANGLE_GEOMETRY_CANVAS.len())
                .expect("Len of geometry must fit in u32"),
        },
    );
    drawable_builder.add_attribute(
        0,
        wgpu::VertexStepMode::Vertex,
        TRIANGLE_GEOMETRY_CANVAS,
        wgpu::VertexFormat::Float32x2,
    )?;
    let drawable = drawable_builder.build();
    Ok(drawable)
}
