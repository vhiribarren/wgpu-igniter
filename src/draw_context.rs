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

use std::array;
use std::collections::{BTreeMap, HashSet};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::scenario::WinitEventLoopHandler;
use anyhow::{anyhow, bail, Ok};
use bytemuck::NoUninit;
use log::debug;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{PipelineLayoutDescriptor, SurfaceConfiguration, Texture};
use winit::window::Window;

pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

enum DrawMode {
    Direct {
        vertex_count: u32,
    },
    Indexed {
        format: wgpu::IndexFormat,
        index_count: u32,
        index_buffer: wgpu::Buffer,
    },
}

pub enum DrawModeParams<'a> {
    Direct { vertex_count: u32 },
    Indexed { index_data: IndexData<'a> },
}

pub enum IndexData<'a> {
    U32(&'a [u32]),
    U16(&'a [u16]),
}

impl IndexData<'_> {
    pub fn format(&self) -> wgpu::IndexFormat {
        match self {
            IndexData::U32(_) => wgpu::IndexFormat::Uint32,
            IndexData::U16(_) => wgpu::IndexFormat::Uint16,
        }
    }
    pub fn size(&self) -> u32 {
        match self {
            IndexData::U32(data) => data.len() as u32,
            IndexData::U16(data) => data.len() as u32,
        }
    }
    pub fn data(&self) -> &[u8] {
        match self {
            IndexData::U32(data) => bytemuck::cast_slice(data),
            IndexData::U16(data) => bytemuck::cast_slice(data),
        }
    }
}

pub trait UnitformType {
    type AlignedType: NoUninit;
    fn apply_alignment(&self) -> Self::AlignedType;
}

macro_rules! impl_uniform {
    ( $($type:ty),+ ) => {
        $(
            impl UnitformType for $type {
                type AlignedType = Self;
                fn apply_alignment(&self) -> Self::AlignedType {
                    *self
                }

            }
        )*
    };
}
impl_uniform!(f32, u32, i32);
impl_uniform!([f32; 2], [f32; 3], [f32; 4]);
impl_uniform!([u32; 2], [u32; 3], [u32; 4]);
impl_uniform!([i32; 2], [i32; 3], [i32; 4]);
impl_uniform!([[f32; 4]; 4], [[u32; 4]; 4], [[i32; 4]; 4]);

impl UnitformType for [[f32; 3]; 3] {
    type AlignedType = [[f32; 4]; 3];
    fn apply_alignment(&self) -> Self::AlignedType {
        array::from_fn(|i| [self[i][0], self[i][1], self[i][2], 0.])
    }
}

pub struct Uniform<T> {
    value: T,
    buffer: wgpu::Buffer,
}

impl<T: UnitformType> Uniform<T> {
    pub fn new(context: &DrawContext, value: T) -> Self {
        let buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[value.apply_alignment()]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });
        Self { value, buffer }
    }
    pub fn binding_resource(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
    pub fn read_uniform(&self) -> &T {
        &self.value
    }
    pub fn write_uniform(&mut self, context: &DrawContext, data: T) {
        self.value = data;
        context.queue.write_buffer(
            &self.buffer,
            0 as wgpu::BufferAddress,
            bytemuck::cast_slice(&[self.value.apply_alignment()]),
        );
    }
}


pub trait StorageBufferType: NoUninit {
    type AlignedType: NoUninit;
    fn apply_alignment(&self) -> Self::AlignedType;
 }
impl StorageBufferType for [[f32; 3]; 3] {
    type AlignedType = [[f32; 4]; 3];
    fn apply_alignment(&self) -> Self::AlignedType {
        array::from_fn(|i| [self[i][0], self[i][1], self[i][2], 0.])
    }
}
impl StorageBufferType for [[f32; 4]; 4] {
    type AlignedType = [[f32; 4]; 4];
    fn apply_alignment(&self) -> Self::AlignedType {
        *self
    }
}


#[derive(Clone)]
pub struct StorageBuffer<T: StorageBufferType> {
    pub(crate) count: usize,
    pub(crate) remote_buffer: Arc<wgpu::Buffer>,
    local_buffer: Vec<T::AlignedType>,
}

impl<T: StorageBufferType> StorageBuffer<T> {
    pub fn new_array(context: &DrawContext, data_init: &[T]) -> Self {
        let local_buffer: Vec<T::AlignedType> = data_init.iter().map(|item| item.apply_alignment()).collect();
        Self {
            count: data_init.len(),
            remote_buffer: Arc::new(context.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Storage Buffer"),
                contents: bytemuck::cast_slice(&local_buffer),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            })),
            local_buffer
        }
    }
    pub fn binding_resource(&self) -> wgpu::BindingResource {
        self.remote_buffer.as_entire_binding()
    }
    pub fn start_write<'a>(&'a mut self, context: &'a DrawContext) -> StorageBufferWriteGuard<'a, T> {
        StorageBufferWriteGuard {context, storage_buffer: self}
    }
}

pub struct  StorageBufferWriteGuard<'a, T: StorageBufferType> {
    context: &'a DrawContext,
    storage_buffer: &'a mut StorageBuffer<T>
}

impl <'a, T: StorageBufferType> StorageBufferWriteGuard<'a, T> {
    pub fn apply_write(self) {
        drop(self);
    }
    pub fn len(&self) -> usize {
        self.storage_buffer.count
    }
    pub fn set_value(&mut self, index: usize, value: T) {
        self.storage_buffer.local_buffer[index] = value.apply_alignment();
    }
}

impl<'a, T: StorageBufferType> Drop for StorageBufferWriteGuard<'a, T> {
    fn drop(&mut self) {
        self.context.queue.write_buffer(
            &self.storage_buffer.remote_buffer,
            0 as wgpu::BufferAddress,
            bytemuck::cast_slice(&self.storage_buffer.local_buffer),
        );
    }
}


pub trait InstancesAttributeType: NoUninit {
    fn vertex_format() -> wgpu::VertexFormat;
}
impl InstancesAttributeType for [f32; 3] {
    fn vertex_format() -> wgpu::VertexFormat {
        wgpu::VertexFormat::Float32x3
    }
}

#[derive(Clone)]
pub struct InstancesAttribute<T> {
    pub(crate) count: usize,
    pub(crate) instance_buffer: Arc<wgpu::Buffer>,
    _type: PhantomData<T>,
}

impl<T: InstancesAttributeType> InstancesAttribute<T> {
    pub fn new(context: &DrawContext, data_init: &[T]) -> Self {
        Self {
            count: data_init.len(),
            instance_buffer: Arc::new(context.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(data_init),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            })),
            _type: PhantomData,
        }
    }
}


pub struct DrawableBuilder<'a> {
    context: &'a DrawContext,
    vtx_shader_module: &'a wgpu::ShaderModule,
    frg_shader_module: &'a wgpu::ShaderModule,
    used_locations: HashSet<u32>,
    attributes: Vec<Vec<wgpu::VertexAttribute>>,
    buffers: Vec<Arc<wgpu::Buffer>>,
    draw_mode: DrawMode,
    layouts: Vec<wgpu::VertexBufferLayout<'a>>,
    instance_count: u32,
    blend_option: Option<wgpu::BlendState>,
    binding_groups: Vec<Option<BTreeMap<u32, (wgpu::BindingResource<'a>, wgpu::BindGroupLayoutEntry)>>>,
}

impl<'a> DrawableBuilder<'a> {
    pub fn new(
        context: &'a DrawContext,
        vtx_shader_module: &'a wgpu::ShaderModule,
        frg_shader_module: &'a wgpu::ShaderModule,
        draw_params: DrawModeParams,
    ) -> Self {
        let draw_mode = match draw_params {
            DrawModeParams::Direct { vertex_count } => DrawMode::Direct { vertex_count },
            DrawModeParams::Indexed { index_data } => {
                let index_buffer =
                    context
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: index_data.data(),
                            usage: wgpu::BufferUsages::INDEX,
                        });
                DrawMode::Indexed {
                    format: index_data.format(),
                    index_count: index_data.size(),
                    index_buffer,
                }
            }
        };
        Self {
            context,
            vtx_shader_module,
            frg_shader_module,
            used_locations: HashSet::new(),
            attributes: Vec::new(),
            buffers: Vec::new(),
            layouts: Vec::new(),
            binding_groups: Vec::new(),
            instance_count: 1,
            draw_mode,
            blend_option: None,
        }
    }
    pub fn set_instance_count(&mut self, value: u32) -> &mut Self {
        self.instance_count = value;
        self
    }
    pub fn set_blend_option(&mut self, blend_option: wgpu::BlendState) -> &mut Self {
        self.blend_option = Some(blend_option);
        self
    }
    pub fn add_uniform<T>(
        &mut self,
        bind_group: u32,
        binding: u32,
        uniform: &'a Uniform<T>,
    ) -> Result<&mut Self, anyhow::Error>
    where
        T: UnitformType,
    {
        let bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let bind_group = bind_group as usize;
        if bind_group >= self.binding_groups.len() {
            self.binding_groups.resize(bind_group + 1, None);
        }
        let to_store = (uniform.binding_resource(), bind_group_layout_entry);
        match self.binding_groups.get_mut(bind_group).unwrap() {
            Some(entry) => {
                entry.insert(binding, to_store);
            }
            None => {
                let mut bindings = BTreeMap::new();
                bindings.insert(binding, to_store);
                self.binding_groups[bind_group] = Some(bindings);
            }
        };
        // TODO Ensure group and binding are not already used
        Ok(self)
    }
    pub fn add_storage_buffer<T>(
        &mut self,
        bind_group: u32,
        binding: u32,
        storage_buffer: &'a StorageBuffer<T>,
    ) -> Result<&mut Self, anyhow::Error>
    where
        T: StorageBufferType,
    {
        let bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let bind_group = bind_group as usize;
        if bind_group >= self.binding_groups.len() {
            self.binding_groups.resize(bind_group + 1, None);
        }
        let to_store = (storage_buffer.binding_resource(), bind_group_layout_entry);
        match self.binding_groups.get_mut(bind_group).unwrap() {
            Some(entry) => {
                entry.insert(binding, to_store);
            }
            None => {
                let mut bindings = BTreeMap::new();
                bindings.insert(binding, to_store);
                self.binding_groups[bind_group] = Some(bindings);
            }
        };
        // TODO Ensure group and binding are not already used
        Ok(self)
    }
    pub fn add_attribute<T>(
        &mut self,
        shader_location: u32,
        step_mode: wgpu::VertexStepMode,
        data: &[T],
        format: wgpu::VertexFormat,
    ) -> Result<&mut Self, anyhow::Error>
    where
        T: bytemuck::NoUninit,
    {
        if self.used_locations.contains(&shader_location) {
            bail!("Location {} already used!", shader_location);
        }
        self.used_locations.insert(shader_location);
        let attributes = vec![wgpu::VertexAttribute {
            format,
            offset: 0,
            shader_location,
        }];
        let layout = wgpu::VertexBufferLayout {
            array_stride: format.size() as wgpu::BufferAddress,
            step_mode,
            attributes: &[], // Filled later during build
        };
        let buffer = self
            .context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::VERTEX,
            });
        self.attributes.push(attributes);
        self.layouts.push(layout);
        self.buffers.push(Arc::new(buffer));
        Ok(self)
    }
    pub fn add_instances_attribute<T>(
        &mut self,
        shader_location: u32,
        instances_attributes: &InstancesAttribute<T>,
    ) -> Result<&mut Self, anyhow::Error>
    where
        T: InstancesAttributeType,
    {
        if self.used_locations.contains(&shader_location) {
            bail!("Location {} already used!", shader_location);
        }
        self.used_locations.insert(shader_location);
        let attributes = vec![wgpu::VertexAttribute {
            format: T::vertex_format(),
            offset: 0,
            shader_location,
        }];
        let layout = wgpu::VertexBufferLayout {
            array_stride: T::vertex_format().size() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[], // Filled later during build
        };
        self.attributes.push(attributes);
        self.layouts.push(layout);
        self.buffers
            .push(Arc::clone(&instances_attributes.instance_buffer));
        Ok(self)
    }
    pub fn build(self) -> Drawable {
        let mut bind_groups = BTreeMap::<u32, wgpu::BindGroup>::new();
        let mut bind_group_layouts = Vec::new();
        for (group_id, group) in self.binding_groups.into_iter().enumerate() {
            let mut bind_group_layout_entries = Vec::new();
            let mut bind_group_entries = Vec::new();
            if let Some(group) = group {
                for (bind_id, (bind, entry)) in group {
                    bind_group_layout_entries.push(entry);
                    bind_group_entries.push(wgpu::BindGroupEntry {
                        binding: bind_id,
                        resource: bind,
                    });
                }
            }
            let bind_group_layout =
                self.context
                    .device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &bind_group_layout_entries,
                    });
            let bind_group = self
                .context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &bind_group_layout,
                    entries: &bind_group_entries,
                });
            bind_group_layouts.push(bind_group_layout);
            bind_groups.insert(group_id as u32, bind_group);
        }

        let mut vertex_buffer_layouts = self.layouts;
        for (layout, attribute) in vertex_buffer_layouts.iter_mut().zip(self.attributes.iter()) {
            layout.attributes = attribute;
        }
        let vertex_state = wgpu::VertexState {
            module: self.vtx_shader_module,
            entry_point: None,
            buffers: &vertex_buffer_layouts,
            compilation_options: Default::default(),
        };
        let fragment_state = wgpu::FragmentState {
            module: self.frg_shader_module,
            entry_point: None,
            targets: &[Some(wgpu::ColorTargetState {
                format: self.context.surface_config.format,
                blend: self.blend_option,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        };
        let pipeline_layout =
            self.context
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(), // Not sure if right order here
                    push_constant_ranges: &[],
                });
        let pipeline =
            self.context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    cache: None,
                    label: Some("Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: vertex_state,
                    fragment: Some(fragment_state),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill, // wgpu::PolygonMode::Line
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: self.context.multisample_config.get_multisample_count(),
                        ..Default::default()
                    },
                    multiview: None,
                });
        let blend_color_opacity = wgpu::Color::WHITE;

        Drawable {
            draw_mode: self.draw_mode,
            buffers: self.buffers,
            instance_count: self.instance_count,
            pipeline,
            bind_groups,
            blend_color_opacity,
        }
    }
}

pub struct Drawable {
    draw_mode: DrawMode,
    buffers: Vec<Arc<wgpu::Buffer>>,
    pub(crate) instance_count: u32,
    pipeline: wgpu::RenderPipeline,
    blend_color_opacity: wgpu::Color,
    bind_groups: BTreeMap<u32, wgpu::BindGroup>,
}

impl Drawable {
    pub fn set_blend_color_opacity(&mut self, value: f64) {
        let value = value.clamp(0., 1.);
        self.blend_color_opacity = wgpu::Color {
            r: value,
            g: value,
            b: value,
            a: 1.0,
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_blend_constant(self.blend_color_opacity);
        for (group_id, bind_group) in &self.bind_groups {
            render_pass.set_bind_group(*group_id, bind_group, &[]);
        }
        for (slot, vertex_buffer) in self.buffers.iter().enumerate() {
            render_pass.set_vertex_buffer(slot as u32, vertex_buffer.slice(..));
        }
        match &self.draw_mode {
            DrawMode::Direct { vertex_count } => {
                render_pass.draw(0..*vertex_count, 0..self.instance_count);
            }
            DrawMode::Indexed {
                format,
                index_count,
                index_buffer,
            } => {
                render_pass.set_index_buffer(index_buffer.slice(..), *format);
                render_pass.draw_indexed(0..*index_count, 0, 0..self.instance_count);
            }
        }
    }
}

impl AsRef<Drawable> for Drawable {
    fn as_ref(&self) -> &Drawable {
        self
    }
}

pub struct MultiSampleConfig {
    multisample_enabled: bool,
    multisample_count: u32,
}

impl MultiSampleConfig {
    pub fn get_multisample_count(&self) -> u32 {
        match self.multisample_enabled {
            true => self.multisample_count,
            false => 1,
        }
    }
    pub fn is_multisample_enabled(&self) -> bool {
        self.multisample_enabled
    }
}

trait DeviceLocalExt {
    fn create_depth_texture(
        &self,
        surface_config: &wgpu::SurfaceConfiguration,
        multisample_config: &MultiSampleConfig,
    ) -> wgpu::Texture;
    fn create_multisample_texture(
        &self,
        surface_config: &wgpu::SurfaceConfiguration,
        multisample_config: &MultiSampleConfig,
    ) -> Option<wgpu::Texture>;
}

impl DeviceLocalExt for wgpu::Device {
    fn create_depth_texture(
        &self,
        surface_config: &SurfaceConfiguration,
        multisample_config: &MultiSampleConfig,
    ) -> Texture {
        self.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: multisample_config.get_multisample_count(),
            dimension: wgpu::TextureDimension::D2,
            view_formats: &[],
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        })
    }

    fn create_multisample_texture(
        &self,
        surface_config: &SurfaceConfiguration,
        multisample_config: &MultiSampleConfig,
    ) -> Option<Texture> {
        match multisample_config.multisample_enabled {
            true => Some(self.create_texture(&wgpu::TextureDescriptor {
                label: Some("Mutisample Texture"),
                size: wgpu::Extent3d {
                    width: surface_config.width,
                    height: surface_config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: multisample_config.get_multisample_count(),
                dimension: wgpu::TextureDimension::D2,
                format: surface_config.format,
                view_formats: &[],
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            })),
            false => None,
        }
    }
}

pub struct DrawContext {
    _adapter: wgpu::Adapter,
    multisample_texture: Option<wgpu::Texture>,
    surface: wgpu::Surface<'static>,
    pub multisample_config: MultiSampleConfig,
    pub depth_texture: wgpu::Texture,
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,
    pub surface_config: wgpu::SurfaceConfiguration,
}

impl DrawContext {
    const DEFAULT_MULTISAMPLE_ENABLED: bool = true;
    const DEFAULT_MULTISAMPLE_COUNT: u32 = 4;
    pub const BIND_GROUP_INDEX_CAMERA: u32 = 0;

    // FIXME winit window has size of 0 at startup for web browser, so also passing dimensions to draw context
    pub async fn new(
        window: Arc<Window>,
        dimensions: Option<Dimensions>,
    ) -> anyhow::Result<DrawContext> {
        let (width, height) = match dimensions {
            Some(d) => (d.width, d.height),
            None => (window.inner_size().width, window.inner_size().height),
        };
        let multisample_config = MultiSampleConfig {
            multisample_enabled: Self::DEFAULT_MULTISAMPLE_ENABLED,
            multisample_count: Self::DEFAULT_MULTISAMPLE_COUNT,
        };
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: Default::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or_else(|| anyhow!("Could not create WebGPU adapter"))?;
        debug!("{:?}", adapter);
        debug!("{:?}", adapter.features());
        let required_limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device Descriptor"),
                    required_features: wgpu::Features::empty(),
                    required_limits,
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            desired_maximum_frame_latency: 2,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            view_formats: vec![],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &surface_config);
        let depth_texture = device.create_depth_texture(&surface_config, &multisample_config);
        let multisample_texture =
            device.create_multisample_texture(&surface_config, &multisample_config);

        Ok(DrawContext {
            multisample_config,
            multisample_texture,
            _adapter: adapter,
            surface,
            device,
            queue,
            surface_config,
            depth_texture,
        })
    }

    pub fn create_shader_module(&self, wgsl_shader: &str) -> wgpu::ShaderModule {
        self.device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(wgsl_shader.into()),
            })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
        self.depth_texture = self
            .device
            .create_depth_texture(&self.surface_config, &self.multisample_config);
        self.multisample_texture = self
            .device
            .create_multisample_texture(&self.surface_config, &self.multisample_config);
    }

    pub fn render_scene(&self, scene: &dyn WinitEventLoopHandler) -> anyhow::Result<()> {
        let depth_texture_view = self
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let displayed_texture = self.surface.get_current_texture()?;
        let displayed_view = displayed_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let (pass_view, pass_resolve_target) = if self.multisample_config.is_multisample_enabled() {
            let multisample_texture = self
                .multisample_texture
                .as_ref()
                .expect("When multisample_enabled is at true, this optional should not be empty");
            let multisample_view =
                multisample_texture.create_view(&wgpu::TextureViewDescriptor::default());
            (multisample_view, Some(&displayed_view))
        } else {
            (displayed_view, None)
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            timestamp_writes: None,
            occlusion_query_set: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &pass_view,
                resolve_target: pass_resolve_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.5,
                        b: 0.5,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
        });
        scene.on_render(&mut render_pass);
        drop(render_pass);
        let command_buffers = std::iter::once(encoder.finish());
        self.queue.submit(command_buffers);
        displayed_texture.present();
        Ok(())
    }
}
