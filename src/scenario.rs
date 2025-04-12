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

use crate::{
    cameras::WinitCameraAdapter,
    draw_context::DrawContext,
    scene::{Scene, Scene3D},
};
use egui_winit::EventResponse;
use web_time::{Duration, Instant};
use winit::event::{DeviceEvent, KeyEvent, WindowEvent};

pub struct ProcessingInterval {
    pub scenario_start: Instant,
    pub processing_delta: Duration,
}

impl Default for ProcessingInterval {
    fn default() -> Self {
        Self {
            scenario_start: Instant::now(),
            processing_delta: Duration::new(0, 0),
        }
    }
}

pub struct RenderContext<'a> {
    pub render_interval: &'a ProcessingInterval,
    pub draw_context: &'a DrawContext,
}

pub trait WinitEventLoopHandler {
    fn on_mouse_event(&mut self, _event: &DeviceEvent) {}
    fn on_keyboard_event(&mut self, _event: &KeyEvent) {}
    fn on_window_event(&mut self, _event: &WindowEvent) -> EventResponse {
        EventResponse::default()
    }
    fn on_render(&mut self, render_context: &RenderContext, render_pass: wgpu::RenderPass<'_>);
    fn is_finished(&self) -> bool {
        false
    }
}

pub struct SceneElements {
    pub camera: WinitCameraAdapter,
    pub scene: Scene3D,
}

pub trait Scenario {
    fn scene_elements_mut(&mut self) -> &mut SceneElements;
    fn on_mouse_event(&mut self, event: &DeviceEvent) {
        self.scene_elements_mut().camera.mouse_event_listener(event);
    }
    fn on_keyboard_event(&mut self, event: &KeyEvent) {
        self.scene_elements_mut()
            .camera
            .keyboard_event_listener(event);
    }
    fn on_window_event(&mut self, _event: &WindowEvent) -> EventResponse {
        EventResponse::default()
    }
    fn on_resize(&mut self, _draw_context: &DrawContext) {}
    fn on_update(&mut self, update_context: &RenderContext);
    fn on_post_render(
        &mut self,
        _render_context: &RenderContext,
        _render_pass: &mut wgpu::RenderPass<'static>,
    ) {
    }
}

pub struct ScenarioScheduler {
    scenario: Box<dyn Scenario>,
}

pub type WinitEventLoopBuilder = dyn Fn(&mut DrawContext) -> Box<dyn WinitEventLoopHandler>;

impl ScenarioScheduler {
    pub fn run(scenario: impl Scenario + 'static) -> Box<dyn WinitEventLoopHandler> {
        Box::new(Self {
            scenario: Box::new(scenario),
        })
    }
}

impl WinitEventLoopHandler for ScenarioScheduler {
    fn on_mouse_event(&mut self, event: &DeviceEvent) {
        self.scenario.on_mouse_event(event);
    }

    fn on_keyboard_event(&mut self, event: &KeyEvent) {
        self.scenario
            .scene_elements_mut()
            .camera
            .keyboard_event_listener(event);
    }

    fn on_window_event(&mut self, event: &WindowEvent) -> EventResponse {
        self.scenario.on_window_event(event)
    }

    fn on_render(&mut self, render_context: &RenderContext, render_pass: wgpu::RenderPass<'_>) {
        /*
        self.scenario.scene_elements_mut().camera.update();
        let scenario = &mut *self.scenario;
        let SceneElements { camera, scene } = scenario.scene_elements_mut();
        scene.update(render_context, &camera.camera);
        let mut rpass = render_pass.forget_lifetime();
        scene.render(&mut rpass);
        scenario.on_post_render(render_context, &mut rpass);
        */

        let scenario = &mut *self.scenario;
        let elements = scenario.scene_elements_mut();

        elements.camera.update();
        elements
            .scene
            .update(render_context, &elements.camera.camera);

        let mut rpass = render_pass.forget_lifetime();
        elements.scene.render(&mut rpass);
        scenario.on_post_render(render_context, &mut rpass);
    }
}
