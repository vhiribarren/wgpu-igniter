use anyhow::Result;
use chrono::{Datelike, Timelike, Utc};
use wgpu::ShaderModule;
use winit::event::DeviceEvent;

use crate::{
    BindingSlot, DrawContext, DrawModeParams, Drawable, DrawableBuilder, EventState, TimeInfo,
    Uniform,
};

use super::Plugin;

const CANVAS_STATIC_SHADER: &str = include_str!("./canvas.wgsl");

/*
// TODO Implement shadertoy variables

ShaderToy variables:

- [X] uniform float iTime;
- [X] uniform float iTimeDelta;
- [X] uniform float iFrame;
- [X] uniform vec3 iResolution;
- [ ] uniform vec4 iMouse;
- [X] uniform vec4 iDate;
- [ ] uniform float iSampleRate;
- [ ] uniform float iChannelTime[4];
- [ ] uniform vec3 iChannelResolution[4];
- [ ] uniform samplerXX iChanneli;
*/

pub struct CanvasPlugin {
    canvas: Drawable,
    u_time: Uniform<f32>,
    u_time_delta: Uniform<f32>,
    u_frame: Uniform<f32>,
    u_resolution: Uniform<[f32; 3]>,
    u_mouse: Uniform<[f32; 4]>,
    u_date: Uniform<[f32; 4]>,
}

impl CanvasPlugin {
    pub fn new(
        draw_context: &DrawContext,
        fragment_shader: &ShaderModule,
        uniforms: &[BindingSlot],
    ) -> Result<Self> {
        let u_time = Uniform::new(draw_context, 0f32);
        let u_time_delta = Uniform::new(draw_context, 0f32);
        let u_frame = Uniform::new(draw_context, 0f32);
        let u_resolution = Uniform::new(draw_context, [0f32; 3]);
        let u_mouse = Uniform::new(draw_context, [0f32; 4]);
        let u_date = Uniform::new(draw_context, [0f32; 4]);
        let shader_module = &draw_context.create_shader_module(CANVAS_STATIC_SHADER);
        let mut drawable_builder = DrawableBuilder::new(
            draw_context,
            shader_module,
            fragment_shader,
            DrawModeParams::Direct { vertex_count: 3 },
        );
        drawable_builder
            .add_binding_slot(&BindingSlot {
                binding: 0,
                bind_group: 0,
                resource: &u_time,
            })
            .expect("Bind group 0 and binding 0 should not have been already taken.")
            .add_binding_slot(&BindingSlot {
                binding: 1,
                bind_group: 0,
                resource: &u_time_delta,
            })
            .expect("Bind group 0 and binding 1 should not have been already taken.")
            .add_binding_slot(&BindingSlot {
                binding: 2,
                bind_group: 0,
                resource: &u_frame,
            })
            .expect("Bind group 0 and binding 2 should not have been already taken.")
            .add_binding_slot(&BindingSlot {
                binding: 3,
                bind_group: 0,
                resource: &u_resolution,
            })
            .expect("Bind group 0 and binding 3 should not have been already taken.")
            .add_binding_slot(&BindingSlot {
                binding: 4,
                bind_group: 0,
                resource: &u_mouse,
            })
            .expect("Bind group 0 and binding 4 should not have been already taken.")
            .add_binding_slot(&BindingSlot {
                binding: 5,
                bind_group: 0,
                resource: &u_date,
            })
            .expect("Bind group 0 and binding 5 should not have been already taken.");
        for uniform in uniforms {
            drawable_builder.add_binding_slot(&BindingSlot {
                binding: uniform.binding,
                bind_group: uniform.bind_group,
                resource: uniform.resource,
            })?;
        }
        let canvas = drawable_builder.build();
        Ok(Self {
            canvas,
            u_time,
            u_time_delta,
            u_frame,
            u_resolution,
            u_mouse,
            u_date,
        })
    }
}

impl Plugin for CanvasPlugin {
    fn on_mouse_event(&mut self, event: &DeviceEvent) -> EventState {
        // TODO Actually, behavior depends on if button is pressed ; probably requires better mouse event API
        if let DeviceEvent::MouseMotion { delta } = event {
            #[allow(clippy::cast_possible_truncation)]
            self.u_mouse
                .write_uniform([delta.0 as f32, delta.1 as f32, 0.0, 0.0]);
        }
        EventState::default()
    }

    #[allow(clippy::cast_precision_loss)]
    fn on_update(&mut self, draw_context: &DrawContext, time_info: &TimeInfo) {
        let dimensions = draw_context.surface_dimensions();
        let now = Utc::now();
        let year = now.year() as f32;
        let month = now.month() as f32;
        let day = now.day() as f32;
        let seconds_since_midnight =
            now.num_seconds_from_midnight() as f32 + now.nanosecond() as f32 / 1_000_000_000.0;

        self.u_time
            .write_uniform(time_info.init_start.elapsed().as_secs_f32());
        self.u_time_delta
            .write_uniform(time_info.processing_delta.as_secs_f32());
        self.u_frame
            .write_uniform(self.u_frame.read_uniform() + 1.0);
        self.u_resolution.write_uniform([
            dimensions.width as f32,
            dimensions.height as f32,
            dimensions.surface_ratio(),
        ]);
        self.u_date
            .write_uniform([year, month, day, seconds_since_midnight]);
    }

    fn on_render(
        &mut self,
        _draw_context: &DrawContext,
        _time_info: &TimeInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        self.canvas.render(render_pass);
    }
}
