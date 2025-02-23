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

pub mod canvas;
pub mod color;
pub mod cube;
pub mod triangle;

use std::cell::RefCell;
use std::rc::Rc;

use crate::draw_context::{DrawContext, Drawable, StorageBuffer};
use crate::draw_context::{InstancesAttribute, Uniform};
use cgmath::SquareMatrix;
use cgmath::{InnerSpace, Matrix, Matrix3, Matrix4};

fn extract_rotation(matrix: Matrix4<f32>) -> Matrix3<f32> {
    // Extract the upper-left 3x3 matrix (which may include scaling)
    let a = Matrix3::from_cols(
        matrix.x.truncate(), // First column
        matrix.y.truncate(), // Second column
        matrix.z.truncate(), // Third column
    );

    // Normalize each column vector to remove scaling
    Matrix3::from_cols(a.x.normalize(), a.y.normalize(), a.z.normalize())
}

pub trait Shareable: Sized {
    fn into_shareable(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }
}

pub trait Transforms {
    fn set_transform(&mut self, context: &DrawContext, transform: Matrix4<f32>);
    fn get_transform(&self) -> &Matrix4<f32>;
    fn apply_transform(&mut self, context: &DrawContext, transform: Matrix4<f32>);
}

pub struct Object3DUniforms {
    pub view: Uniform<[[f32; 4]; 4]>,
    pub normals: Option<Uniform<[[f32; 3]; 3]>>,
}

pub struct Object3D {
    drawable: Drawable,
    transform: Matrix4<f32>,
    opacity: f32,
    uniforms: Object3DUniforms,
}

impl Object3D {
    pub fn new(drawable: Drawable, uniforms: Object3DUniforms) -> Self {
        Object3D {
            drawable,
            transform: Matrix4::<f32>::identity(),
            opacity: 1.0,
            uniforms,
        }
    }
    fn update_normal_mat(&mut self, context: &DrawContext) {
        let Some(normal_tranform) = &mut self.uniforms.normals else {
            return;
        };
        let rotation_mat = extract_rotation(self.transform);
        let normal_mat = rotation_mat.invert().unwrap().transpose();
        normal_tranform.write_uniform(context, normal_mat.into());
    }
    pub fn set_opacity(&mut self, value: f32) {
        self.opacity = value.clamp(0., 1.);
        self.drawable.set_blend_color_opacity(self.opacity as f64);
    }
    pub fn get_opacity(&self) -> f32 {
        self.opacity
    }
}

impl Transforms for Object3D {
    fn set_transform(&mut self, context: &DrawContext, transform: Matrix4<f32>) {
        self.transform = transform;
        self.uniforms
            .view
            .write_uniform(context, self.transform.into());
        self.update_normal_mat(context);
    }
    fn get_transform(&self) -> &Matrix4<f32> {
        &self.transform
    }
    fn apply_transform(&mut self, context: &DrawContext, transform: Matrix4<f32>) {
        self.transform = transform * self.transform;
        self.uniforms
            .view
            .write_uniform(context, self.transform.into());
        self.update_normal_mat(context);
    }
}

impl Shareable for Object3D {}

impl AsRef<Drawable> for Object3D {
    fn as_ref(&self) -> &Drawable {
        &self.drawable
    }
}

pub struct Object3DInstanceGroupHandlers {
    positions: InstancesAttribute<[f32; 3]>,
    normal_mats: StorageBuffer<[[f32; 3]; 3]>,
}

pub struct Object3DInstance<'a> {
    position: &'a mut [f32; 3],
}

impl Object3DInstance<'_> {
    pub fn set_position(&mut self, pos: cgmath::Vector3<f32>) {
        *self.position = pos.into();
    }
}

pub struct Object3DInstanceGroup {
    drawable: Drawable,
    opacity: f32,
    handlers: Object3DInstanceGroupHandlers,
}

impl Object3DInstanceGroup {
    pub fn new(drawable: Drawable, handlers: Object3DInstanceGroupHandlers) -> Self {
        Self {
            drawable,
            opacity: 0.,
            handlers,
        }
    }
    pub fn update_instances<F>(&self, context: &DrawContext, f: F)
    where
        F: Fn(usize, &mut Object3DInstance) + 'static + Send,
    {
        let Object3DInstanceGroupHandlers {
            positions,
            normal_mats,
        } = &self.handlers;
        let mut data = vec![[0.0; 3]; positions.count];
        for (idx, instance) in data.iter_mut().enumerate() {
            f(idx, &mut Object3DInstance { position: instance });
            // TODO Update normal matrix
        }
        context
            .queue
            .write_buffer(&positions.instance_buffer, 0, bytemuck::cast_slice(&data));
    }
    pub fn set_opacity(&mut self, value: f32) {
        self.opacity = value.clamp(0., 1.);
        self.drawable.set_blend_color_opacity(self.opacity as f64);
    }
    pub fn get_opacity(&self) -> f32 {
        self.opacity
    }
}

impl Shareable for Object3DInstanceGroup {}

impl AsRef<Drawable> for Object3DInstanceGroup {
    fn as_ref(&self) -> &Drawable {
        &self.drawable
    }
}
