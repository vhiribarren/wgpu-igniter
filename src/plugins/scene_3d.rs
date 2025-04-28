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

use crate::{
    cameras::{Camera, InteractiveCamera},
    draw_context::{DrawContext, Drawable, Uniform},
    render_loop::RenderContext,
};
use cgmath::{SquareMatrix, Zero};
use std::{cell::RefCell, rc::Rc};
use winit::event::{DeviceEvent, KeyEvent};

use super::Plugin;

pub type DrawableWrapper = Rc<RefCell<dyn AsRef<Drawable>>>;

#[allow(clippy::manual_non_exhaustive)]
pub struct Scene3DUniforms {
    pub camera_mat: Uniform<[[f32; 4]; 4]>,
    pub camera_pos: Uniform<[f32; 3]>,
    _private: (),
}

pub struct Scene3D {
    drawables: Vec<DrawableWrapper>,
    scene_uniforms: Scene3DUniforms,
}

impl Scene3D {
    pub fn new(context: &DrawContext) -> Self {
        Self {
            drawables: Vec::new(),
            scene_uniforms: Scene3DUniforms {
                camera_mat: Uniform::new(context, cgmath::Matrix4::identity().into()),
                camera_pos: Uniform::new(context, cgmath::Vector3::zero().into()),
                _private: (),
            },
        }
    }
    #[must_use]
    pub fn scene_uniforms(&self) -> &Scene3DUniforms {
        &self.scene_uniforms
    }

    pub fn update(&mut self, _context: &RenderContext, camera: &Camera) {
        self.scene_uniforms
            .camera_mat
            .write_uniform(camera.get_camera_matrix().into());
        self.scene_uniforms
            .camera_pos
            .write_uniform(camera.eye_position().into());
    }

    pub fn add(&mut self, element: DrawableWrapper) {
        self.drawables.push(element);
    }

    #[must_use]
    pub fn drawables(&self) -> &[DrawableWrapper] {
        &self.drawables
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        for drawable in self.drawables() {
            drawable.borrow().as_ref().render(render_pass);
        }
    }
}

pub struct SceneElements {
    pub camera: InteractiveCamera,
    pub scene: Scene3D,
}

impl Plugin for SceneElements {
    fn on_mouse_event(&mut self, event: &DeviceEvent) {
        self.camera.mouse_event_listener(event);
    }
    fn on_keyboard_event(&mut self, event: &KeyEvent) {
        self.camera.keyboard_event_listener(event);
    }
    fn on_render(
        &mut self,
        render_context: &RenderContext,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        let Self { camera, scene } = self;
        camera.update_screen_size(render_context.draw_context.surface_dimensions());
        camera.update_control();
        scene.update(render_context, &camera.controled_camera);
        scene.render(render_pass);
    }
}
