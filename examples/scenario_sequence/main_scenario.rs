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
use wgpu_igniter::{
    DrawContext, LaunchContext, RenderLoopHandler, TimeInfo, plugins::PluginRegistry,
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
    fn get_scenario_mut(&mut self) -> &mut Box<dyn RenderLoopHandler> {
        match self {
            ScenarioDescription::WithDuration { scenario, .. } => scenario,
            ScenarioDescription::WithTermination { scenario } => scenario,
        }
    }
}

pub struct MainScenario {
    scenarios_iter: IntoIter<(ScenarioDescription, PluginRegistry)>,
    current_scenario: ScenarioDescription,
    last_instant: Instant,
    end_flag: bool,
}

impl MainScenario {
    pub fn new(
        LaunchContext {
            draw_context,
            plugin_registry,
        }: LaunchContext,
    ) -> Self {
        let scenarios = vec![
            {
                let mut local_registry = PluginRegistry::default();
                let scenario_desc = ScenarioDescription::WithDuration {
                    scenario: Box::new(scenario_triangle::MainScenario::new(LaunchContext {
                        draw_context,
                        plugin_registry: &mut local_registry,
                    })),
                    duration: Duration::from_secs(5),
                };
                (scenario_desc, local_registry)
            },
            {
                let mut local_registry = PluginRegistry::default();
                let scenario_desc = ScenarioDescription::WithDuration {
                    scenario: Box::new(scenario_cube::MainScenario::new(LaunchContext {
                        draw_context,
                        plugin_registry: &mut local_registry,
                    })),
                    duration: Duration::from_secs(5),
                };
                (scenario_desc, local_registry)
            },
        ];
        let mut scenarios_iter = scenarios.into_iter();

        let (mut current_scenario, current_registry) = scenarios_iter.next().unwrap();
        *plugin_registry = current_registry;
        current_scenario
            .get_scenario_mut()
            .on_init(plugin_registry, draw_context);

        debug!("Switching to next scenario");
        let last_instant = Instant::now();
        Self {
            scenarios_iter,
            current_scenario,
            last_instant,
            end_flag: false,
        }
    }
    fn progress_scenario(
        &mut self,
        plugin_registry: &mut PluginRegistry,
        draw_context: &mut DrawContext,
    ) {
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
        if let Some((next_scenario, next_registry)) = self.scenarios_iter.next() {
            self.current_scenario = next_scenario;
            *plugin_registry = next_registry;
            self.current_scenario
                .get_scenario_mut()
                .on_init(plugin_registry, draw_context);
            debug!("Switching to next scenario");
        } else {
            debug!("No more scenarios to run, stopping");
            self.end_flag = true;
        }
    }
}

impl RenderLoopHandler for MainScenario {
    fn is_finished(&self) -> bool {
        self.end_flag
    }

    fn on_update(
        &mut self,
        plugin_registry: &mut PluginRegistry,
        draw_context: &mut DrawContext,
        time_info: &TimeInfo,
    ) {
        self.progress_scenario(plugin_registry, draw_context);
        if self.is_finished() {
            return;
        }
        self.current_scenario.get_scenario_mut().on_update(
            plugin_registry,
            draw_context,
            time_info,
        );
    }

    fn on_render(
        &mut self,
        plugin_registry: &mut PluginRegistry,
        draw_context: &DrawContext,
        time_info: &TimeInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        let scenario = self.current_scenario.get_scenario_mut();
        scenario.on_render(plugin_registry, draw_context, time_info, render_pass);
    }
}
