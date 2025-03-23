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

use std::{cell::RefCell, rc::Rc};

use cgmath::{SquareMatrix, Zero};

use crate::{
    cameras::Camera,
    draw_context::{DrawContext, Drawable, Uniform},
    scenario::RenderContext,
};

pub type DrawableWrapper = Rc<RefCell<dyn AsRef<Drawable>>>;

pub trait Scene {
    fn add(&mut self, element: DrawableWrapper);
    fn drawables(&self) -> &[DrawableWrapper];
    fn render(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        for drawable in self.drawables() {
            drawable.borrow().as_ref().render(render_pass);
        }
    }
}

#[allow(clippy::manual_non_exhaustive)]
pub struct Scene3DUniforms {
    pub camera_mat: Uniform<[[f32; 4]; 4]>,
    pub camera_pos: Uniform<[f32; 3]>,
    _private: (),
}

pub struct CameraData {
    pub matrix: cgmath::Matrix4<f32>,
    pub position: cgmath::Point3<f32>,
}

pub struct Scene3D {
    drawables: Vec<DrawableWrapper>,
    scene_uniforms: Scene3DUniforms,
}

impl Scene3D {
    pub fn new(context: &DrawContext) -> Scene3D {
        Scene3D {
            drawables: Vec::new(),
            scene_uniforms: Scene3DUniforms {
                camera_mat: Uniform::new(context, cgmath::Matrix4::identity().into()),
                camera_pos: Uniform::new(context, cgmath::Vector3::zero().into()),
                _private: (),
            },
        }
    }
    pub fn scene_uniforms(&self) -> &Scene3DUniforms {
        &self.scene_uniforms
    }

    pub fn update(&mut self, _context: &RenderContext, camera: &Camera) {
        self.scene_uniforms
            .camera_mat
            .write_uniform(camera.get_camera_matrix().into());
        self.scene_uniforms
            .camera_pos
            .write_uniform(camera.get_eye_position().into());
    }

    pub fn update_with_camera_data(&mut self, _context: &RenderContext, camera_data: CameraData) {
        self.scene_uniforms
            .camera_mat
            .write_uniform(camera_data.matrix.into());
        self.scene_uniforms
            .camera_pos
            .write_uniform(camera_data.position.into());
    }
}

impl Scene for Scene3D {
    fn add(&mut self, element: DrawableWrapper) {
        self.drawables.push(element);
    }
    fn drawables(&self) -> &[DrawableWrapper] {
        &self.drawables
    }
}
