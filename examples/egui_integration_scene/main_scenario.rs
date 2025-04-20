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
use wgpu_igniter::cameras::{InteractiveCamera, PerspectiveConfig};
use wgpu_igniter::primitives::cube::CubeOptions;
use wgpu_igniter::primitives::{Object3D, Shareable, Transforms, cube};
use wgpu_igniter::scene_3d::{Scene3D, SceneElements, SceneLoopHandler};
use wgpu_igniter::support::egui::EguiSupport;
use wgpu_igniter::{DrawContext, EventState, RenderContext};
use winit::event::DeviceEvent;

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
    scene_elements: SceneElements,
    egui_support: EguiSupport,
    gui_state: GuiState,
}

impl MainScenario {
    pub fn new(draw_context: &DrawContext) -> Self {
        let egui_support = EguiSupport::new(draw_context);
        let gui_state = GuiState {
            pixels_per_point: egui_support.get_pixels_per_point(),
            anim_speed: 1.0,
        };
        let camera = InteractiveCamera::new(PerspectiveConfig::default().into());
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

        let scene_elements = SceneElements { camera, scene };
        Self {
            cube,
            scene_elements,
            egui_support,
            gui_state,
        }
    }
}

impl SceneLoopHandler for MainScenario {
    fn scene_elements_mut(&mut self) -> &mut SceneElements {
        &mut self.scene_elements
    }

    fn on_update(&mut self, context: &RenderContext) {
        let total_seconds = context.time_info.init_start.elapsed().as_secs_f32();
        let new_rotation = ROTATION_DEG_PER_S * total_seconds;
        // Translation on z to be in the clipped space (between -w and w) and camera in front of the cube
        let z_translation: cgmath::Matrix4<f32> =
            cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 0.0, 1.0));
        let transform: cgmath::Matrix4<f32> =
            cgmath::Matrix4::from_angle_z(cgmath::Deg(new_rotation));
        self.cube
            .borrow_mut()
            .set_transform(transform * z_translation);
    }

    // NOTE Or maybe EguiSupport could add some callbacks to the window event, to avoid having to write those lines?
    // Actually, could be the base of other mechanisms like for Scene3D, instead of manually iterating on the drawables?
    fn on_window_event(&mut self, event: &winit::event::WindowEvent) -> EventState {
        self.egui_support.on_window_event(event)
    }

    fn on_mouse_event(&mut self, event: &DeviceEvent) {
        if self.egui_support.egui_context().is_using_pointer() {
            return;
        }
        self.scene_elements_mut().camera.mouse_event_listener(event);
    }

    fn on_post_render(
        &mut self,
        render_context: &RenderContext,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        self.egui_support
            .draw(render_context.draw_context, render_pass, |egui_context| {
                self.gui_state.generate_egui(egui_context);
            });
    }
}
