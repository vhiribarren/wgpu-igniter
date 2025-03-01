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

use std::cell::RefCell;
use std::rc::Rc;

use cgmath::Rotation3;
use wgpu_lite_wrapper::cameras::{PerspectiveConfig, WinitCameraAdapter};
use wgpu_lite_wrapper::draw_context::DrawContext;
use wgpu_lite_wrapper::gen_camera_scene;
use wgpu_lite_wrapper::primitives::{Object3DInstanceGroup, Shareable, cube};
use wgpu_lite_wrapper::scenario::{Scenario, UpdateContext};
use wgpu_lite_wrapper::scene::{Scene, Scene3D};

const DEFAULT_SHADER: &str = include_str!("cube_instances.wgsl");
const CUBE_WIDTH_COUNT: usize = 10;
const CUBE_DEPTH_COUNT: usize = 10;
const CUBE_OFFSET: f32 = 1.5;

pub struct MainScenario {
    pub cube: Rc<RefCell<Object3DInstanceGroup>>,
    pub scene: Scene3D,
    pub camera: WinitCameraAdapter,
}

impl MainScenario {
    pub fn new(draw_context: &DrawContext) -> Self {
        let camera = WinitCameraAdapter::new(PerspectiveConfig::default().into());
        let shader_module = draw_context.create_shader_module(DEFAULT_SHADER);
        let mut scene = Scene3D::new(draw_context);
        let cube = {
            let mut cube_init = cube::create_cube_with_normals_instances(
                draw_context,
                &shader_module,
                &shader_module,
                scene.scene_uniforms(),
                (CUBE_WIDTH_COUNT * CUBE_DEPTH_COUNT) as u32,
                Default::default(),
            )
            .unwrap();
            cube_init.update_instances(draw_context, |idx, instance| {
                let x = (idx % CUBE_WIDTH_COUNT) as f32;
                let z = (idx / CUBE_WIDTH_COUNT) as f32;
                instance.set_translation(cgmath::Vector3::new(
                    x * CUBE_OFFSET - (CUBE_WIDTH_COUNT as f32 * CUBE_OFFSET) / 2.0,
                    0.0,
                    z * CUBE_OFFSET - (CUBE_DEPTH_COUNT as f32 * CUBE_OFFSET) / 2.0,
                ));
            });
            cube_init.into_shareable()
        };
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

    fn on_update(&mut self, update_context: &UpdateContext) {
        let &UpdateContext {
            draw_context,
            update_interval,
        } = update_context;
        let delta = update_interval.scenario_start.elapsed().as_secs_f32().cos();
        // self.cube
        //     .borrow_mut()
        //     .update_instances(draw_context, move |index, instance| {
        //         let rotation =
        //             cgmath::Quaternion::from_angle_y(cgmath::Deg(index as f32 * delta * 45.));
        //         let translation =
        //             cgmath::Vector3::new(delta * index as f32, delta * index as f32, 0.);
        //         //instance.set_translation(translation);
        //         //instance.set_rotation(rotation);
        //     });
    }
}
