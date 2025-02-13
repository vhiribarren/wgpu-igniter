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

use std::marker::PhantomData;
use std::sync::LazyLock;

use bytemuck::NoUninit;
use cgmath::SquareMatrix;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

use crate::draw_context::DrawContext;
use crate::draw_context::DrawModeParams;
use crate::draw_context::Drawable;
use crate::draw_context::DrawableBuilder;
use crate::draw_context::IndexData;
use crate::draw_context::Uniform;
use crate::primitives::color;
use crate::primitives::Object3D;
use crate::scene::Scene3DUniforms;

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
    [0., 1., 0.],
    [0., 0., 0.],
    [1., 0., 0.],
    [1., 0., 0.],
    [1., 1., 0.],
    [0., 1., 0.],
    // Back
    [1., 1., 1.],
    [1., 0., 1.],
    [0., 0., 1.],
    [0., 0., 1.],
    [0., 1., 1.],
    [1., 1., 1.],
    // Top
    [0., 1., 0.],
    [1., 1., 0.],
    [1., 1., 1.],
    [1., 1., 1.],
    [0., 1., 1.],
    [0., 1., 0.],
    // Bottom
    [0., 0., 0.],
    [0., 0., 1.],
    [1., 0., 1.],
    [1., 0., 1.],
    [1., 0., 0.],
    [0., 0., 0.],
    // Left
    [0., 1., 1.],
    [0., 0., 1.],
    [0., 0., 0.],
    [0., 0., 0.],
    [0., 1., 0.],
    [0., 1., 1.],
    // Right
    [1., 1., 0.],
    [1., 0., 0.],
    [1., 0., 1.],
    [1., 0., 1.],
    [1., 1., 1.],
    [1., 1., 0.],
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
) -> Result<Object3D, anyhow::Error> {
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
        )?
        .add_attribute(
            1,
            wgpu::VertexStepMode::Vertex,
            CUBE_COLOR_COMPACT,
            wgpu::VertexFormat::Float32x3,
        )?
        .add_uniform(0, 0, &uniforms.camera_uniform)?
        .add_uniform(1, 0, &transform_uniform)?;
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
    Ok(Object3D::new(
        drawable,
        Object3DUniforms {
            view: transform_uniform,
            normals: None,
        },
    ))
}

pub fn create_cube_with_normals(
    context: &DrawContext,
    vtx_module: &wgpu::ShaderModule,
    frg_module: &wgpu::ShaderModule,
    uniforms: &Scene3DUniforms,
    options: CubeOptions,
) -> Result<Object3D, anyhow::Error> {
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
        )?
        .add_attribute(
            1,
            wgpu::VertexStepMode::Vertex,
            &CUBE_NORMALS_DUPLICATES,
            wgpu::VertexFormat::Float32x3,
        )?
        .add_uniform(0, 0, &uniforms.camera_uniform)?
        .add_uniform(1, 0, &transform_uniform)?
        .add_uniform(1, 1, &normals_uniform)?;

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
    Ok(Object3D::new(
        drawable,
        Object3DUniforms {
            view: transform_uniform,
            normals: Some(normals_uniform),
        },
    ))
}

pub struct DrawInstances<T> {
    count: usize,
    stride: usize,
    instance_buffer: wgpu::Buffer,
    _type: PhantomData<T>,
}

impl<T: NoUninit> DrawInstances<T> {
    pub fn new(context: &DrawContext, data_init: &[T]) -> Self {
        Self {
            count: data_init.len(),
            stride: size_of::<T>(), // FIXME pas de prob d'alignement pour [f32; 3] ?
            instance_buffer: context.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(data_init),
                usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::VERTEX,
            }),
            _type: PhantomData,
        }
    }
    pub fn iter(&self) -> impl Iterator + use<'_, T> {
        DrawInstancesIterator {
            instances: &self,
            index: 0,
        }
    }
    //fn map_async(lambda) {
    //}
}

struct DrawInstancesIterator<'a, T> {
    instances: &'a DrawInstances<T>,
    index: usize,
}

impl<'a, T> Iterator for DrawInstancesIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub fn create_cube_with_normals_instances(
    context: &DrawContext,
    vtx_module: &wgpu::ShaderModule,
    frg_module: &wgpu::ShaderModule,
    uniforms: &Scene3DUniforms,
    count: u32,
    options: CubeOptions,
) -> Result<Drawable, anyhow::Error> {
    let positions = (0..count).map(|i| [(2*i) as f32 - 4.0, 0., 0.]).collect::<Vec<_>>();


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
        )?
        .add_attribute(
            1,
            wgpu::VertexStepMode::Instance,
            &positions,
            wgpu::VertexFormat::Float32x3,
        )?
        .add_uniform(0, 0, &uniforms.camera_uniform)?;

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
    Ok(drawable)
}
