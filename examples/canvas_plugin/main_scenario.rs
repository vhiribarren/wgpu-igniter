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

use wgpu_igniter::{LaunchContext, RenderLoopHandler, plugins::canvas::CanvasPlugin};

const FRAGMENT_SHADER: &str = include_str!("./fragment_shader.wgsl");

pub struct MainScenario {}

impl MainScenario {
    pub fn new(
        LaunchContext {
            draw_context,
            plugin_registry,
        }: LaunchContext,
    ) -> Self {
        let canvas = CanvasPlugin::new(
            &draw_context,
            &draw_context.create_shader_module(FRAGMENT_SHADER),
            &[],
        )
        .expect("Issue with canvas plugin creation");
        plugin_registry.register(canvas);
        MainScenario {}
    }
}

impl RenderLoopHandler for MainScenario {}
