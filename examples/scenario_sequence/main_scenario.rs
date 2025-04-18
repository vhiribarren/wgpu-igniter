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

use std::vec::IntoIter;

use log::debug;
use web_time::{Duration, Instant};
use wgpu_lite_wrapper::{
    draw_context::DrawContext,
    render_loop::{RenderContext, RenderLoopHandler, SceneLoopScheduler},
};

use crate::{scenario_cube, scenario_triangle};

enum ScenarioDescription {
    WithDuration {
        scenario: Box<dyn RenderLoopHandler>,
        duration: Duration,
    },
    #[allow(dead_code)]
    WithTermination {
        scenario: Box<dyn RenderLoopHandler>,
    },
}

impl ScenarioDescription {
    pub fn get_scenario_mut(&mut self) -> &mut Box<dyn RenderLoopHandler> {
        match self {
            ScenarioDescription::WithDuration { scenario, .. } => scenario,
            ScenarioDescription::WithTermination { scenario } => scenario,
        }
    }
}

pub struct MainScenario {
    scenarios_iter: IntoIter<ScenarioDescription>,
    current_scenario: ScenarioDescription,
    last_instant: Instant,
    end_flag: bool,
}

impl MainScenario {
    pub fn new(draw_context: &DrawContext) -> Self {
        let scenarios = vec![
            ScenarioDescription::WithDuration {
                scenario: Box::new(scenario_triangle::MainScenario::new(draw_context)),
                duration: Duration::from_secs(5),
            },
            ScenarioDescription::WithDuration {
                scenario: SceneLoopScheduler::run(scenario_cube::MainScenario::new(draw_context)),
                duration: Duration::from_secs(5),
            },
        ];
        let mut scenarios_iter = scenarios.into_iter();
        let current_scenario = scenarios_iter.next().unwrap();
        let last_instant = Instant::now();
        Self {
            scenarios_iter,
            current_scenario,
            last_instant,
            end_flag: false,
        }
    }
    fn progress_scenario(&mut self) {
        match &self.current_scenario {
            ScenarioDescription::WithDuration { duration, .. } => {
                let now = Instant::now();
                if now - self.last_instant >= *duration {
                    self.last_instant = now;
                } else {
                    return;
                }
            }
            ScenarioDescription::WithTermination { scenario } => {
                if scenario.is_finished() {
                    self.last_instant = Instant::now();
                } else {
                    return;
                }
            }
        }
        if let Some(next_scenario) = self.scenarios_iter.next() {
            self.current_scenario = next_scenario;
            debug!("Switching to next scenario");
        } else {
            debug!("No more scenarios to run, stopping");
            self.end_flag = true;
        }
    }
}

impl RenderLoopHandler for MainScenario {
    fn on_render(&mut self, render_context: &RenderContext, render_pass: wgpu::RenderPass<'_>) {
        self.progress_scenario();
        if self.is_finished() {
            return;
        }
        self.current_scenario
            .get_scenario_mut()
            .on_render(render_context, render_pass);
    }

    fn is_finished(&self) -> bool {
        self.end_flag
    }
}
