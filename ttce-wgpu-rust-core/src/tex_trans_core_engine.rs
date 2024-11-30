use std::collections::HashMap;

use wgpu::CommandEncoder;

use crate::compute_shader::{TTComputeShader, TTComputeShaderID};
use crate::render_texture::{ConvertTextureFormat, TTRenderTexture};
use crate::{TexTransCoreTextureChannel, TexTransCoreTextureFormat};

#[derive(Debug)]
pub struct TexTransCoreEngineDevice {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,

    pub(crate) compute_shader: Vec<TTComputeShader>,
    pub(crate) converter_id: HashMap<ConvertTextureFormat, TTComputeShaderID>,

    default_render_texture_format: TexTransCoreTextureFormat,
    max_command_stack_count: u32,
}

#[derive(Debug)]
pub struct TexTransCoreEngineContext<'a> {
    pub(crate) engine: &'a TexTransCoreEngineDevice,

    command_encoder: Option<CommandEncoder>,
    command_stack_count: u32,
}

impl TexTransCoreEngineDevice {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        TexTransCoreEngineDevice {
            device,
            queue,

            compute_shader: Vec::new(),
            converter_id: HashMap::new(),

            default_render_texture_format: TexTransCoreTextureFormat::Float,
            max_command_stack_count: 16,
            // is_linear: false,
        }
    }
    pub fn create_ctx(&self) -> TexTransCoreEngineContext {
        TexTransCoreEngineContext {
            engine: self,
            command_encoder: None,
            command_stack_count: 0,
        }
    }

    pub fn default_texture_format(&self) -> TexTransCoreTextureFormat {
        self.default_render_texture_format
    }
    pub fn set_default_texture_format(&mut self, format: TexTransCoreTextureFormat) {
        self.default_render_texture_format = format;
    }

    pub(crate) fn create_render_texture(&self, desc: &TTRtRequestDescriptor) -> TTRenderTexture {
        let tex_format = match desc.format {
            RequestFormat::AutoWithChannel(tex_trans_core_texture_channel) => {
                TTRenderTexture::to_wgpu_texture_format(
                    if tex_trans_core_texture_channel == TexTransCoreTextureChannel::RGBA {
                        self.default_render_texture_format
                    } else {
                        TexTransCoreTextureFormat::Float
                    },
                    tex_trans_core_texture_channel,
                )
            }
            RequestFormat::Manual(
                tex_trans_core_texture_format,
                tex_trans_core_texture_channel,
            ) => TTRenderTexture::to_wgpu_texture_format(
                tex_trans_core_texture_format,
                tex_trans_core_texture_channel,
            ),
        };

        let usage = wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT;

        let tex_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
            dimension: wgpu::TextureDimension::D2,
            format: tex_format,
            usage,
            mip_level_count: 1,
            sample_count: 1,
            label: Some("Create-From-RenderTexture-With-ColorTexture"),
            view_formats: &[tex_format],
        };

        let render_texture = self.device.create_texture(&tex_desc);

        TTRenderTexture {
            texture: render_texture,
        }
    }
}
pub(crate) struct TTRtRequestDescriptor {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) format: RequestFormat,
}
pub(crate) enum RequestFormat {
    AutoWithChannel(TexTransCoreTextureChannel),
    Manual(TexTransCoreTextureFormat, TexTransCoreTextureChannel),
}
impl TexTransCoreEngineContext<'_> {
    pub fn get_command_encoder_as_mut(&mut self) -> &mut CommandEncoder {
        if self.command_encoder.is_none() {
            self.command_encoder = Some(
                self.engine
                    .device
                    .create_command_encoder(&Default::default()),
            );
        }
        self.command_stack_count += 1;

        self.command_encoder.as_mut().unwrap()
    }

    pub fn check_command_stack(&mut self) {
        if self.command_stack_count > self.engine.max_command_stack_count {
            self.send_command();
        }
    }

    pub fn send_command(&mut self) {
        if let Some(command_encoder) = self.command_encoder.take() {
            self.engine.queue.submit(Some(command_encoder.finish()));
        } else {
            self.engine.queue.submit([]);
        }
        self.command_stack_count = 0;
    }
}
