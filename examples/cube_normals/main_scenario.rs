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

use std::rc::Rc;

use wgpu_lite_wrapper::cameras::{PerspectiveConfig, WinitCameraAdapter};
use wgpu_lite_wrapper::draw_context::DrawContext;
use wgpu_lite_wrapper::gen_camera_scene;
use wgpu_lite_wrapper::primitives::{cube, Object3D};
use wgpu_lite_wrapper::scenario::{Scenario, UpdateContext};
use wgpu_lite_wrapper::scene::{Scene, Scene3D};

const DEFAULT_SHADER: &str = include_str!("cube_normals.wgsl");

const ROTATION_DEG_PER_S: f32 = 45.0;

pub struct MainScenario {
    pub cube: Rc<std::cell::RefCell<Object3D>>,
    pub scene: Scene3D,
    pub camera: WinitCameraAdapter,
}

impl MainScenario {
    pub fn new(draw_context: &DrawContext) -> Self {
        let camera = WinitCameraAdapter::new(PerspectiveConfig::default().into());
        let shader_module = draw_context.create_shader_module(DEFAULT_SHADER);
        let mut scene = Scene3D::new(draw_context);
        let cube = cube::create_cube_with_normals(
            draw_context,
            &shader_module,
            &shader_module,
            scene.scene_uniforms(),
            Default::default(),
        )
        .unwrap()
        .as_shareable();
        scene.add(cube.clone());
        Self {
            cube,
            scene,
            camera,
        }
    }
}

impl Scenario for MainScenario {
    gen_camera_scene!(camera, scene);

    fn on_update(&mut self, context: &UpdateContext) {
        let total_seconds = context
            .update_interval
            .scenario_start
            .elapsed()
            .as_secs_f32();
        let new_rotation = ROTATION_DEG_PER_S * total_seconds;
        let z_translation: cgmath::Matrix4<f32> =
            cgmath::Matrix4::from_translation(cgmath::Vector3::new(-0.5, -0.5, -0.5));
        let transform: cgmath::Matrix4<f32> =
            cgmath::Matrix4::from_angle_x(cgmath::Deg(new_rotation));
        self.cube
            .borrow_mut()
            .set_transform(context.draw_context, transform * z_translation);
    }
}
