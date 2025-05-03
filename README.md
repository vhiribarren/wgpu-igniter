# wgpu-igniter

Library wrapper on top of `winit` and `wgpu` to quickly bootstrap simple wgpu
projects.

Used technologies: Rust, winit, wgpu, WebAssembly, WebGPU.

## Convention choices

- coordinate system is left-handed
- triangle front face is counter clock wise

## How to run

A default example is loaded as the main binary. It can be launched with:

    cargo run

Numerous examples are available in the `examples` directory:

    cargo run --example cube_shader_transition

In order to check if the launcher panics at simple executions, a headless mode
exists used by tests. It can be enabled by using the `HEADLESS` environment
variable:

    HEADLESS=true cargo run

To test the main app and all examples compile and run without an immediate
crash:

    cargo test

## How to use

The main element is the `wgpu_igniter::RenderLoopHandler` trait, for which an
implementation must be provided to  `wgpu_igniter::launch_app`.

All methods have a default implementation, to avoid cluttering your code with
unused methods, and also because a plugin mechanism may already provide the
implementation you need.

```rust
use wgpu_igniter::{launch_app, RenderLoopHandler, LaunchContext};

fn main() {
    launch_app(|c: LaunchContext| Box::new(MainScenario::new(c)));
}

struct MainScenario {
    // Your elements
}

impl MainScenario {
    pub fn new(ctx: LaunchContext) -> Self {
        // Prepare your objects
        MainScenario {}
    }
}

impl RenderLoopHandler for MainScenario {

    // To react to some input or winit window events
    fn on_mouse_event(&mut self, event: &DeviceEvent) {}
    fn on_keyboard_event(&mut self, event: &KeyEvent) {}
    fn on_window_event(&mut self, event: &WindowEvent) -> EventState {
        EventState::default()
    }

    // Called once at startup
    fn on_init(&mut self, plugin_registry: &mut PluginRegistry, draw_context: &mut DrawContext) {}

    // Called in the render loop
    fn on_update(
        &mut self,
        plugin_registry: &mut PluginRegistry,
        draw_context: &mut DrawContext,
        time_info: &TimeInfo,
    ) {
    }
    fn on_render(
        &mut self,
        plugin_registry: &mut PluginRegistry,
        draw_context: &DrawContext,
        time_info: &TimeInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
    }

}
```

## Current plugins

The goal of this library is to allow implementation at the low level, with less
WebGPU boilerplate. Default implementations for recurrent patterns are provided:

- `egui`: ease integration of the egui framework for simple GUI
- `scene_3d`: management of camera and scene graph for 3D scenes
- `canvas`: ready-to use canvas for fragment shader effects, with default
  uniforms like the one provided by the ShaderToy website

## WASM version

For the web version, you must be sure you can compile to the WebAssembly target first:

    rustup target add wasm32-unknown-unknown
    cargo install -f wasm-bindgen-cli

If you have Python3 installed, you can compile to WASM and host a local web server
by launching the command:

    ./run-web.sh

... then launch a browser with the displayed URL.

> [!NOTE]  
> For now, examples cannot be launched like that. Only the default example is
> launched.

## References

I heavily read and used:
- [Learn WGPU tutorial](https://sotrh.github.io/learn-wgpu).
- [WebGPU Fundamentals](https://webgpufundamentals.org/)

Mains references links are:
- https://github.com/gfx-rs/wgpu
- https://www.w3.org/TR/webgpu
- https://www.w3.org/TR/WGSL

Having a look to some [wasm-compatible examples](https://github.com/gfx-rs/wgpu/tree/master/wgpu/examples)
did helped a lot too. 


## TODO

- [ ] Load gltf models
- [ ] Allow usage of webgl shaders
- [ ] Replicate ShaderToys shader features
- [ ] Usage of compute shader with output usable by vertex/fragment shaders
- [ ] Composition of various shader stages

## License

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