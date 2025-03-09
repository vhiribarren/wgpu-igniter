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

use std::sync::LazyLock;

use cgmath::SquareMatrix;

use crate::draw_context::DrawContext;
use crate::draw_context::DrawModeParams;
use crate::draw_context::DrawableBuilder;
use crate::draw_context::IndexData;
use crate::draw_context::Uniform;
use crate::primitives::Object3D;
use crate::primitives::color;
use crate::scene::Scene3DUniforms;

use super::Object3DInstanceGroup;
use super::Object3DInstanceGroupHandlers;
use super::Object3DUniforms;

#[rustfmt::skip]
const CUBE_GEOMETRY_COMPACT: &[[f32; 3]] = &[
    [-0.5, 0.5, -0.5],
    [0.5, 0.5, -0.5],
    [0.5, -0.5, -0.5],
    [-0.5, -0.5, -0.5],
    [-0.5, 0.5, 0.5],
    [0.5, 0.5, 0.5],
    [0.5, -0.5, 0.5],
    [-0.5, -0.5, 0.5],
];
#[rustfmt::skip]
const CUBE_INDICES_COMPACT: &[u16] = &[
    // Front
    0, 2, 1,
    0, 3, 2,
    // Back
    5, 7, 4,
    5, 6, 7,
    // Above
    4, 1, 5,
    4, 0, 1,
    // Below
    6, 3, 7,
    6, 2, 3,
    // Left side
    7, 0, 4,
    7, 3, 0,
    // Right side
    2, 5, 1,
    2, 6, 5,
];
#[rustfmt::skip]
const CUBE_COLOR_COMPACT: &[[f32; 3]] = &[
    color::COLOR_WHITE, 
    color::COLOR_BLACK, 
    color::COLOR_RED, 
    color::COLOR_GREEN, 
    color::COLOR_BLUE, 
    color::COLOR_YELLOW, 
    color::COLOR_CYAN, 
    color::COLOR_MAGENTA, 
];

#[rustfmt::skip]
const CUBE_GEOMETRY_DUPLICATES: &[[f32; 3]] = &[
    // Front
    [-0.5, 0.5, -0.5],
    [-0.5, -0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, 0.5, -0.5],
    [-0.5, 0.5, -0.5],
    // Back
    [0.5, 0.5, 0.5],
    [0.5, -0.5, 0.5],
    [-0.5, -0.5, 0.5],
    [-0.5, -0.5, 0.5],
    [-0.5, 0.5, 0.5],
    [0.5, 0.5, 0.5],
    // Top
    [-0.5, 0.5, -0.5],
    [0.5, 0.5, -0.5],
    [0.5, 0.5, 0.5],
    [0.5, 0.5, 0.5],
    [-0.5, 0.5, 0.5],
    [-0.5, 0.5, -0.5],
    // Bottom
    [-0.5, -0.5, -0.5],
    [-0.5, -0.5, 0.5],
    [0.5, -0.5, 0.5],
    [0.5, -0.5, 0.5],
    [0.5, -0.5, -0.5],
    [-0.5, -0.5, -0.5],
    // Left
    [-0.5, 0.5, 0.5],
    [-0.5, -0.5, 0.5],
    [-0.5, -0.5, -0.5],
    [-0.5, -0.5, -0.5],
    [-0.5, 0.5, -0.5],
    [-0.5, 0.5, 0.5],
    // Right
    [0.5, 0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, -0.5, 0.5],
    [0.5, -0.5, 0.5],
    [0.5, 0.5, 0.5],
    [0.5, 0.5, -0.5],
];

#[rustfmt::skip]
const CUBE_NORMALS_COMPACT: &[[f32; 3]] = &[
    // Front
    [0., 0., -1.],
    // Back
    [0., 0., 1.],
    // Top
    [0., 1., 0.],
    // Bottom
    [0., -1., 0.],
    // Left
    [-1., 0., 0.],
    // Right
    [1., 0., 0.],
];

static CUBE_NORMALS_DUPLICATES: LazyLock<Vec<[f32; 3]>> = LazyLock::new(|| {
    let mut normals = Vec::with_capacity(CUBE_NORMALS_COMPACT.len());
    for normal in CUBE_NORMALS_COMPACT {
        for _ in 0..6 {
            normals.push(*normal);
        }
    }
    normals
});

pub struct CubeOptions {
    pub with_alpha: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for CubeOptions {
    fn default() -> Self {
        Self { with_alpha: false }
    }
}

pub fn create_cube_with_colors(
    context: &DrawContext,
    vtx_module: &wgpu::ShaderModule,
    frg_module: &wgpu::ShaderModule,
    uniforms: &Scene3DUniforms,
    options: CubeOptions,
) -> Object3D {
    let transform_uniform = Uniform::new(context, cgmath::Matrix4::identity().into());

    let mut drawable_builder = DrawableBuilder::new(
        context,
        vtx_module,
        frg_module,
        DrawModeParams::Indexed {
            index_data: IndexData::U16(CUBE_INDICES_COMPACT),
        },
    );
    drawable_builder
        .add_attribute(
            0,
            wgpu::VertexStepMode::Vertex,
            CUBE_GEOMETRY_COMPACT,
            wgpu::VertexFormat::Float32x3,
        )
        .expect("Location should be different than for another attribute.")
        .add_attribute(
            1,
            wgpu::VertexStepMode::Vertex,
            CUBE_COLOR_COMPACT,
            wgpu::VertexFormat::Float32x3,
        )
        .expect("Location should be different than for another attribute.")
        .add_uniform(0, 0, &uniforms.camera_uniform)
        .expect("Bind group or binding should be different from other uniforms.")
        .add_uniform(1, 0, &transform_uniform)
        .expect("Bind group or binding should be different from other uniforms.");
    if options.with_alpha {
        drawable_builder.set_blend_option(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Constant,
                dst_factor: wgpu::BlendFactor::OneMinusConstant,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: Default::default(),
        });
    }
    let drawable = drawable_builder.build();
    Object3D::new(
        drawable,
        Object3DUniforms {
            view: transform_uniform,
            normals: None,
        },
    )
}

pub fn create_cube_with_normals(
    context: &DrawContext,
    vtx_module: &wgpu::ShaderModule,
    frg_module: &wgpu::ShaderModule,
    uniforms: &Scene3DUniforms,
    options: CubeOptions,
) -> Object3D {
    let transform_uniform = Uniform::new(context, cgmath::Matrix4::identity().into());
    let normals_uniform = Uniform::new(context, cgmath::Matrix3::identity().into());

    let mut drawable_builder = DrawableBuilder::new(
        context,
        vtx_module,
        frg_module,
        DrawModeParams::Direct {
            vertex_count: CUBE_GEOMETRY_DUPLICATES.len() as u32,
        },
    );
    drawable_builder
        .add_attribute(
            0,
            wgpu::VertexStepMode::Vertex,
            CUBE_GEOMETRY_DUPLICATES,
            wgpu::VertexFormat::Float32x3,
        )
        .expect("Location should be different than for another attribute.")
        .add_attribute(
            1,
            wgpu::VertexStepMode::Vertex,
            &CUBE_NORMALS_DUPLICATES,
            wgpu::VertexFormat::Float32x3,
        )
        .expect("Location should be different than for another attribute.")
        .add_uniform(0, 0, &uniforms.camera_uniform)
        .expect("Bind group or binding should be different from other uniforms.")
        .add_uniform(1, 0, &transform_uniform)
        .expect("Bind group or binding should be different from other uniforms.")
        .add_uniform(1, 1, &normals_uniform)
        .expect("Bind group or binding should be different from other uniforms.");

    if options.with_alpha {
        drawable_builder.set_blend_option(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Constant,
                dst_factor: wgpu::BlendFactor::OneMinusConstant,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: Default::default(),
        });
    }
    let drawable = drawable_builder.build();
    Object3D::new(
        drawable,
        Object3DUniforms {
            view: transform_uniform,
            normals: Some(normals_uniform),
        },
    )
}

pub fn create_cube_with_normals_instances(
    context: &DrawContext,
    vtx_module: &wgpu::ShaderModule,
    frg_module: &wgpu::ShaderModule,
    uniforms: &Scene3DUniforms,
    count: u32,
    options: CubeOptions,
) -> Object3DInstanceGroup {
    let handlers = Object3DInstanceGroupHandlers::new(context, count);
    let mut drawable_builder = DrawableBuilder::new(
        context,
        vtx_module,
        frg_module,
        DrawModeParams::Direct {
            vertex_count: CUBE_GEOMETRY_DUPLICATES.len() as u32,
        },
    );
    drawable_builder
        .set_instance_count(count)
        .add_attribute(
            0,
            wgpu::VertexStepMode::Vertex,
            CUBE_GEOMETRY_DUPLICATES,
            wgpu::VertexFormat::Float32x3,
        )
        .expect("Location should be different than for another attribute.")
        .add_attribute(
            1,
            wgpu::VertexStepMode::Vertex,
            &CUBE_NORMALS_DUPLICATES,
            wgpu::VertexFormat::Float32x3,
        )
        .expect("Location should be different than for another attribute.")
        .add_uniform(0, 0, &uniforms.camera_uniform)
        .expect("Bind group or binding should be different from other uniforms.")
        .add_storage_buffer(1, 0, &handlers.transforms)
        .expect("Bind group or binding should be different from other uniforms.")
        .add_storage_buffer(1, 1, &handlers.normal_mats)
        .expect("Bind group or binding should be different from other uniforms.");

    if options.with_alpha {
        drawable_builder.set_blend_option(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Constant,
                dst_factor: wgpu::BlendFactor::OneMinusConstant,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::default(),
        });
    }
    let drawable = drawable_builder.build();
    Object3DInstanceGroup::new(drawable, handlers)
}
