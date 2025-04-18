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

use cgmath::Rotation3;
use std::cell::RefCell;
use std::rc::Rc;
use wgpu_igniter::cameras::{InteractiveCamera, PerspectiveConfig};
use wgpu_igniter::primitives::cube::CubeOptions;
use wgpu_igniter::primitives::{Object3DInstanceGroup, Shareable, cube};
use wgpu_igniter::scene_3d::{Scene3D, SceneElements, SceneLoopHandler};
use wgpu_igniter::{DrawContext, RenderContext};

const DEFAULT_SHADER: &str = include_str!("cube_instances.wgsl");
const CUBE_WIDTH_COUNT: usize = 50;
const CUBE_DEPTH_COUNT: usize = 50;
const CUBE_OFFSET: f32 = 2.0;

pub struct MainScenario {
    pub cube: Rc<RefCell<Object3DInstanceGroup>>,
    pub scene_elements: SceneElements,
}

impl MainScenario {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    pub fn new(draw_context: &mut DrawContext) -> Self {
        draw_context.set_clear_color(Some(wgpu::Color::BLACK));
        let camera = InteractiveCamera::new(PerspectiveConfig::default().into());
        let shader_module = draw_context.create_shader_module(DEFAULT_SHADER);
        let mut scene = Scene3D::new(draw_context);
        let cube = {
            let mut cube_init = cube::create_cube_with_normals_instances(
                draw_context,
                &shader_module,
                &shader_module,
                scene.scene_uniforms(),
                (CUBE_WIDTH_COUNT * CUBE_DEPTH_COUNT) as u32,
                CubeOptions::default(),
            );
            cube_init.update_instances(|idx, instance| {
                let x = (idx % CUBE_WIDTH_COUNT) as f32;
                let z = (idx / CUBE_WIDTH_COUNT) as f32;
                instance.set_translation(cgmath::Vector3::new(
                    x.mul_add(
                        CUBE_OFFSET,
                        -((CUBE_WIDTH_COUNT as f32 * CUBE_OFFSET) / 2.0),
                    ),
                    0.0,
                    z.mul_add(
                        CUBE_OFFSET,
                        -((CUBE_DEPTH_COUNT as f32 * CUBE_OFFSET) / 2.0),
                    ),
                ));
            });
            cube_init.into_shareable()
        };
        scene.add(cube.clone());
        let scene_elements = SceneElements { camera, scene };
        Self {
            cube,
            scene_elements,
        }
    }
}

impl SceneLoopHandler for MainScenario {
    fn scene_elements_mut(&mut self) -> &mut SceneElements {
        &mut self.scene_elements
    }

    #[allow(clippy::cast_precision_loss)]
    fn on_update(&mut self, render_context: &RenderContext) {
        let &RenderContext {
            time_info: render_interval,
            ..
        } = render_context;
        let delta = render_interval.processing_delta.as_secs_f32();
        self.cube
            .borrow_mut()
            .update_instances(move |index, instance| {
                let rotation = cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::new((index as f32).cos(), (index as f32).sin(), 0.),
                    cgmath::Deg(10. * delta),
                );
                instance.apply_rotation(rotation);
            });
    }
}
