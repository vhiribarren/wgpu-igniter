/*
MIT License

Copyright (c) 2021, 2022 Vincent Hiribarren

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

use std::cell::RefCell;
use std::rc::Rc;

use wgpu_lite_wrapper::cameras::{PerspectiveConfig, WinitCameraAdapter};
use wgpu_lite_wrapper::draw_context::DrawContext;
use wgpu_lite_wrapper::gen_camera_scene;
use wgpu_lite_wrapper::primitives::{Object3D, Shareable, Transforms, cube};
use wgpu_lite_wrapper::scenario::Scenario;
use wgpu_lite_wrapper::scene::{Scene, Scene3D};

const INTERPOLATED_SHADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/shaders/default.wgsl"
));

const FLAT_SHADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/shaders/flat.wgsl"
));

const ROTATION_DEG_PER_S: f32 = 45.0;

pub struct MainScenario {
    cube_left: Rc<RefCell<Object3D>>,
    cube_right: Rc<RefCell<Object3D>>,
    scene: Scene3D,
    camera: WinitCameraAdapter,
}

impl MainScenario {
    pub fn new(draw_context: &DrawContext) -> Self {
        let camera = WinitCameraAdapter::new(PerspectiveConfig::default().into());
        let mut scene = Scene3D::new(draw_context);
        let interpolated_shader_module = draw_context.create_shader_module(INTERPOLATED_SHADER);
        let flat_shader_module = draw_context.create_shader_module(FLAT_SHADER);
        let cube_left = cube::create_cube_with_colors(
            draw_context,
            &interpolated_shader_module,
            &interpolated_shader_module,
            scene.scene_uniforms(),
            Default::default(),
        )
        .into_shareable();
        let cube_right = cube::create_cube_with_colors(
            draw_context,
            &flat_shader_module,
            &flat_shader_module,
            scene.scene_uniforms(),
            Default::default(),
        )
        .into_shareable();
        cube_left
            .borrow_mut()
            .set_transform(cgmath::Matrix4::from_translation(cgmath::Vector3::new(
                -0.5, 0.0, 5.0,
            )));
        cube_right
            .borrow_mut()
            .set_transform(cgmath::Matrix4::from_translation(cgmath::Vector3::new(
                0.5, 0.0, 0.0,
            )));
        scene.add(cube_left.clone());
        scene.add(cube_right.clone());

        Self {
            cube_left,
            cube_right,
            scene,
            camera,
        }
    }
}

impl Scenario for MainScenario {
    gen_camera_scene!(camera, scene);

    fn on_update(&mut self, update_context: &wgpu_lite_wrapper::scenario::UpdateContext) {
        let delta_rotation =
            ROTATION_DEG_PER_S * update_context.update_interval.update_delta.as_secs_f32();
        self.cube_left
            .borrow_mut()
            .apply_transform(cgmath::Matrix4::from_angle_z(cgmath::Deg(delta_rotation)));
        self.cube_right
            .borrow_mut()
            .apply_transform(cgmath::Matrix4::from_angle_y(cgmath::Deg(delta_rotation)));
    }
}
