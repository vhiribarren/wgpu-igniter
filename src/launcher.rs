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

use log::info;
use std::env;

use crate::{
    LaunchContext,
    draw_context::DrawContext,
    render_loop::{RenderLoopBuilder, RenderLoopHandler},
    window::init_event_loop,
};

const GLOBAL_LOG_FILTER: log::LevelFilter = log::LevelFilter::Info;
const ENV_HEADLESS: &str = "HEADLESS";

pub fn launch_app<F>(builder: F)
where
    F: Fn(LaunchContext) -> Box<dyn RenderLoopHandler> + 'static + Send,
{
    init_log();
    info!("Init app");
    let is_headless = env::var(ENV_HEADLESS).is_ok();
    if is_headless {
        info!("Running in headless mode");
        init_headless(Box::new(builder));
    } else {
        init_event_loop(Box::new(builder));
    }
}

fn init_log() {
    let mut builder = fern::Dispatch::new();
    let level_formatter;
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        level_formatter = |level| level;
        builder = builder.chain(fern::Output::call(console_log::log));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use fern::colors::{Color, ColoredLevelConfig};
        let colors = ColoredLevelConfig::new()
            .info(Color::Blue)
            .debug(Color::Green);
        level_formatter = move |level| colors.color(level);
        builder = builder.chain(std::io::stdout());
    }
    builder
        .level(GLOBAL_LOG_FILTER)
        .level_for(env!("CARGO_PKG_NAME"), log::LevelFilter::Debug)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}:{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                level_formatter(record.level()),
                record.target(),
                record.line().unwrap_or_default(),
                message
            ));
        })
        .apply()
        .unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::needless_pass_by_value)]
fn init_headless(builder: Box<RenderLoopBuilder>) {
    use pollster::FutureExt;

    use crate::{TimeInfo, plugins::PluginRegistry, render_loop::RenderContext};
    let draw_context = &mut DrawContext::new(None, None).block_on().unwrap();
    let plugin_registry = &mut PluginRegistry::default();

    let mut scene_handler = builder(LaunchContext {
        draw_context,
        plugin_registry,
    });
    // NOTE I do not like this circular dependency on context
    let render_context = RenderContext {
        time_info: &TimeInfo::default(),
        draw_context,
        _private: (),
    };
    draw_context
        .render_scene(|pass| {
            scene_handler.on_render(
                plugin_registry,
                &render_context,
                &mut pass.forget_lifetime(),
            );
        })
        .unwrap();
}

#[cfg(target_arch = "wasm32")]
fn init_headless(_builder: Box<RenderLoopBuilder>) {
    todo!("Headless mode is not supported in WASM");
}
