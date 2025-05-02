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
use web_time::Duration;
use wgpu_igniter::cameras::{Camera, InteractiveCamera};
use wgpu_igniter::plugins::PluginRegistry;
use wgpu_igniter::plugins::scene_3d::{Scene3D, Scene3DPlugin};
use wgpu_igniter::primitives::cube::CubeOptions;
use wgpu_igniter::primitives::{Object3D, Shareable, Transforms, cube};
use wgpu_igniter::{DrawContext, LaunchContext, RenderLoopHandler, TimeInfo};

const INTERPOLATED_SHADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/shaders/default.wgsl"
));

const FLAT_SHADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/shaders/flat.wgsl"
));

const ROTATION_DEG_PER_S: f32 = 45.0;
const SHADER_TRANSITION_PERIOD: Duration = Duration::from_secs(1);

pub struct MainScenario {
    pub cube_interpolated: Rc<RefCell<Object3D>>,
    pub cube_flat: Rc<RefCell<Object3D>>,
}

impl MainScenario {
    pub fn new(
        LaunchContext {
            draw_context,
            plugin_registry,
        }: LaunchContext,
    ) -> Self {
        let camera = InteractiveCamera::new(Camera::default());
        let interpolated_shader_module = draw_context.create_shader_module(INTERPOLATED_SHADER);
        let flat_shader_module = draw_context.create_shader_module(FLAT_SHADER);

        let mut scene = Scene3D::new(draw_context);
        let scene_uniforms = scene.scene_uniforms();

        let cube_interpolated = cube::create_cube_with_colors(
            draw_context,
            &interpolated_shader_module,
            &interpolated_shader_module,
            scene_uniforms,
            &Default::default(),
        )
        .into_shareable();
        let cube_flat = cube::create_cube_with_colors(
            draw_context,
            &flat_shader_module,
            &flat_shader_module,
            scene_uniforms,
            &CubeOptions { with_alpha: true },
        )
        .into_shareable();

        scene.add(cube_interpolated.clone());
        scene.add(cube_flat.clone());

        plugin_registry.register(Scene3DPlugin { camera, scene });

        Self {
            cube_interpolated,
            cube_flat,
        }
    }
}

impl RenderLoopHandler for MainScenario {
    fn on_update(
        &mut self,
        _plugin_registry: &mut PluginRegistry,
        _draw_context: &mut DrawContext,
        time_info: &TimeInfo,
    ) {
        let delta_rotation = ROTATION_DEG_PER_S * time_info.processing_delta.as_secs_f32();
        let transform = cgmath::Matrix4::from_angle_z(cgmath::Deg(delta_rotation))
            * cgmath::Matrix4::from_angle_y(cgmath::Deg(delta_rotation));
        self.cube_interpolated
            .borrow_mut()
            .apply_transform(transform);
        {
            let mut cube_flat = self.cube_flat.borrow_mut();
            cube_flat.apply_transform(transform);
            cube_flat.set_opacity(
                0.5 + f32::sin(
                    2. * time_info.init_start.elapsed().as_secs_f32()
                        / SHADER_TRANSITION_PERIOD.as_secs_f32(),
                ) / 2_f32,
            );
        }
    }
}
