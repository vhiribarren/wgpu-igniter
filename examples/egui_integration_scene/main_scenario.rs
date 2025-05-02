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
use wgpu_igniter::cameras::{Camera, InteractiveCamera};
use wgpu_igniter::plugins::PluginRegistry;
use wgpu_igniter::plugins::egui::EquiPlugin;
use wgpu_igniter::plugins::scene_3d::{Scene3D, Scene3DPlugin};
use wgpu_igniter::primitives::cube::CubeOptions;
use wgpu_igniter::primitives::{Object3D, Shareable, Transforms, cube};
use wgpu_igniter::{DrawContext, LaunchContext, RenderLoopHandler, TimeInfo};

const DEFAULT_SHADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/shaders/default.wgsl"
));

const ROTATION_DEG_PER_S: f32 = 45.0;

struct GuiState {
    pub anim_speed: f32,
    pixels_per_point: f32,
}

impl GuiState {
    fn generate_egui(&mut self, egui_context: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(egui_context, |ui| {
            ui.label("Egui Integration Example");
        });
        egui::Window::new("Animation Control").show(egui_context, |ui| {
            ui.label("Adjust the animation speed:");
            ui.add(egui::Slider::new(&mut self.anim_speed, 0.1..=5.0).text("Speed"));
            ui.add(egui::DragValue::new(&mut self.pixels_per_point).range(0.5..=2.0));
            ui.label("Pixels per point");
        });
    }
}

pub struct MainScenario {
    cube: Rc<std::cell::RefCell<Object3D>>,
    gui_state: GuiState,
}

impl MainScenario {
    pub fn new(
        LaunchContext {
            draw_context,
            plugin_registry,
        }: LaunchContext,
    ) -> Self {
        let egui_support = EquiPlugin::new(draw_context);
        let gui_state = GuiState {
            pixels_per_point: egui_support.get_pixels_per_point(),
            anim_speed: 1.0,
        };

        let camera = InteractiveCamera::new(Camera::default());
        let shader_module = draw_context.create_shader_module(DEFAULT_SHADER);
        let mut scene = Scene3D::new(draw_context);
        let cube = cube::create_cube_with_colors(
            draw_context,
            &shader_module,
            &shader_module,
            scene.scene_uniforms(),
            &CubeOptions::default(),
        )
        .into_shareable();
        scene.add(cube.clone());

        plugin_registry.register(Scene3DPlugin { camera, scene });
        plugin_registry.register(egui_support);

        Self { cube, gui_state }
    }
}

impl RenderLoopHandler for MainScenario {
    fn on_update(
        &mut self,
        plugin_registry: &mut PluginRegistry,
        _draw_context: &mut DrawContext,
        time_info: &TimeInfo,
    ) {
        let total_seconds = time_info.init_start.elapsed().as_secs_f32();
        let new_rotation = ROTATION_DEG_PER_S * total_seconds;
        // Translation on z to be in the clipped space (between -w and w) and camera in front of the cube
        let z_translation: cgmath::Matrix4<f32> =
            cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 0.0, 1.0));
        let transform: cgmath::Matrix4<f32> =
            cgmath::Matrix4::from_angle_z(cgmath::Deg(new_rotation));
        self.cube
            .borrow_mut()
            .set_transform(transform * z_translation);

        let egui_support = plugin_registry
            .get_mut::<EquiPlugin>()
            .expect("EguiSupport should be registered");
        egui_support.draw(|egui_context| {
            self.gui_state.generate_egui(egui_context);
        });
    }
}
