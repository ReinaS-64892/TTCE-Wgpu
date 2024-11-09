use std::collections::HashMap;

use wgpu::CommandEncoder;

use crate::compute_shader::{TTComputeShader, TTComputeShaderID};
use crate::render_texture::{ConvertTextureFormat, TTRenderTexture};
use crate::{TexTransCoreTextureChannel, TexTransCoreTextureFormat};

#[derive(Debug)]
pub struct TexTransCoreEngin {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,

    pub(crate) compute_shader: Vec<TTComputeShader>,
    pub(crate) converter_id: HashMap<ConvertTextureFormat, TTComputeShaderID>,

    default_render_texture_format: TexTransCoreTextureFormat,
    // is_linear: bool,
}

#[derive(Debug)]
pub struct TexTransCoreEngineContext<'a> {
    pub(crate) engine: &'a TexTransCoreEngin,

    // pub(crate) pooled_render_textures: HashMap<TTRenderTextureDescriptor, Vec<TTRenderTexture>>,
    command_encoder: Option<CommandEncoder>,
}

impl TexTransCoreEngin {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        TexTransCoreEngin {
            device: device,
            queue: queue,

            compute_shader: Vec::new(),
            converter_id: HashMap::new(),

            default_render_texture_format: TexTransCoreTextureFormat::Float,
            // is_linear: false,
        }
    }
    pub fn create_ctx<'a>(&'a self) -> TexTransCoreEngineContext<'a> {
        TexTransCoreEngineContext {
            engine: &self,
            // pooled_render_textures: HashMap::new(),
            command_encoder: None,
        }
    }

    pub fn default_texture_format(&self) -> TexTransCoreTextureFormat {
        self.default_render_texture_format
    }

    pub(crate) fn create_render_texture(&self, desc: &TTRtRequestDescriptor) -> TTRenderTexture {
        let tex_format = match desc.format {
            RequestFormat::AutoWithChannel(tex_trans_core_texture_channel) => {
                TTRenderTexture::to_wgpu_texture_format(
                    self.default_render_texture_format,
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
            usage: usage,
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

    /*
    fn create_blending_from_hlsl(
        &self,
        hlsl_file_path: &str,
    ) -> Result<TTComputeShader, Box<dyn std::error::Error>> {
        let blending_name = String::from("Normal"); // TODO

        let mut hlsl_string = String::new();
        {
            let hlsl_file_result = File::open(hlsl_file_path);

            if let Err(er) = hlsl_file_result {
                return Err(Box::new(er));
            }
            hlsl_file_result
                .unwrap()
                .read_to_string(&mut hlsl_string)
                .unwrap();
        }

        let spv_result = hassle_rs::compile_hlsl(
            hlsl_file_path,
            (hlsl_string + BLENDING_SHADER_TEMPLATE).as_str(),
            "CSMain",
            "cs_6_0",
            &vec!["-spirv", "-HV 2018"],
            &vec![],
        );
        if let Err(er) = spv_result {
            return Err(Box::new(er));
        }
        let spv = spv_result.unwrap();
        let convert_wgsl_string_result = spv_to_wgsl_and_binding_descriptor(spv);
        if let Err(er) = convert_wgsl_string_result {
            return Err(er);
        }
        let (wgsl_string, bindings, wg_size) = convert_wgsl_string_result.unwrap();
        let bind_map = HashMap::<String, u32>::from_iter(bindings);

        let (cs_module, compute_pipeline) =
            self.wgsl_from_compute_pipeline(&blending_name, wgsl_string);

        Ok(TTComputeShader {
            module: cs_module,
            pipeline: compute_pipeline,
            binding_map: bind_map,
            work_group_size: wg_size,
        })
    } */
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

        self.command_encoder.as_mut().unwrap()
    }

    pub fn send_command(&mut self) {
        if let Some(command_encoder) = self.command_encoder.take() {
            self.engine.queue.submit(Some(command_encoder.finish()));
        } else {
            self.engine.queue.submit([]);
        }
    }
}
