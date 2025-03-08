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
use web_time::{Duration, Instant};
use winit::event::{DeviceEvent, KeyEvent};

pub struct UpdateInterval {
    pub scenario_start: Instant,
    pub update_delta: Duration,
}

pub struct UpdateContext<'a> {
    pub update_interval: &'a UpdateInterval,
}

pub trait WinitEventLoopHandler {
    fn on_mouse_event(&mut self, _event: &DeviceEvent) {}
    fn on_keyboard_event(&mut self, _event: &KeyEvent) {}
    fn on_update(&mut self, _update_context: &UpdateContext) {}
    fn on_render<'drawable>(&'drawable self, render_pass: &mut wgpu::RenderPass<'drawable>);
}

pub trait Scenario {
    fn camera(&self) -> &WinitCameraAdapter;
    fn camera_mut(&mut self) -> &mut WinitCameraAdapter;
    fn scene(&self) -> &Scene3D;
    fn scene_mut(&mut self) -> &mut Scene3D;
    fn on_update(&mut self, update_context: &UpdateContext);
}

#[macro_export]
macro_rules! gen_camera_scene {
    ($camera:ident, $scene:ident) => {
        fn camera(&self) -> &WinitCameraAdapter {
            &self.$camera
        }
        fn camera_mut(&mut self) -> &mut WinitCameraAdapter {
            &mut self.$camera
        }
        fn scene(&self) -> &Scene3D {
            &self.$scene
        }
        fn scene_mut(&mut self) -> &mut Scene3D {
            &mut self.$scene
        }
    };
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
        self.scenario.camera_mut().mouse_event_listener(event);
    }

    fn on_keyboard_event(&mut self, event: &KeyEvent) {
        self.scenario.camera_mut().keyboard_event_listener(event);
    }

    fn on_update(&mut self, update_context: &UpdateContext) {
        self.scenario.camera_mut().update();
        let camera_matrix = self.scenario.camera().get_camera_matrix();
        self.scenario
            .scene_mut()
            .update(update_context, camera_matrix);
        self.scenario.on_update(update_context);
    }

    fn on_render<'drawable>(&'drawable self, render_pass: &mut wgpu::RenderPass<'drawable>) {
        self.scenario.scene().render(render_pass);
    }
}
