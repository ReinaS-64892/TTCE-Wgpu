use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::PathBuf;

use wgpu::util::DeviceExt;
use wgpu::{ComputePipeline, ShaderModule};

use crate::render_texture::TTRenderTexture;
use crate::tex_trans_core_engine::{TexTransCoreEngin, TexTransCoreEngineContext};

#[derive(Debug)]
pub struct TTComputeShader {
    pub(crate) module: ShaderModule,
    pub(crate) pipeline: ComputePipeline,
    pub(crate) binding_map: HashMap<String, u32>,
    pub(crate) work_group_size: WorkGroupSize,
}
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub struct TTComputeShaderID(u32);

impl TTComputeShaderID {
    pub fn from(id: u32) -> TTComputeShaderID {
        TTComputeShaderID { 0: id }
    }
}

impl Deref for TTComputeShaderID {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl TexTransCoreEngin {
    pub fn register_compute_shader_from_hlsl(
        &mut self,
        hlsl_file_path: &str,
        hlsl_source_code: Option<&str>,
    ) -> Result<TTComputeShaderID, Box<dyn std::error::Error>> {
        let binding = PathBuf::from(hlsl_file_path);
        let Some(file_name_os_str) = binding.file_name() else {
            todo!()
        };
        let operator_name: String = file_name_os_str.to_string_lossy().into();

        let mut hlsl_string = String::new();

        if let Some(hlsl_str) = hlsl_source_code {
            hlsl_string.push_str(hlsl_str);
        } else {
            let hlsl_file_result = File::open(hlsl_file_path);
            if let Err(er) = hlsl_file_result?.read_to_string(&mut hlsl_string) {
                return Err(Box::new(er));
            }
        };

        let spv_result = hassle_rs::compile_hlsl(
            hlsl_file_path,
            hlsl_string.as_str(),
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

        // println!("{}", wgsl_string);

        let cs_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some((String::from("shade module with ") + &operator_name).as_str()),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&convert_wgsl_format(
                    wgsl_string,
                    TTRenderTexture::to_wgpu_texture_format(
                        self.default_texture_format(),
                        crate::TexTransCoreTextureChannel::RGBA,
                    ),
                ))),
            });
        let compute_pipeline =
            self.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some((String::from("compute pipeline with ") + &operator_name).as_str()),
                    layout: None,
                    module: &cs_module,
                    entry_point: "CSMain",
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                });

        let id = TTComputeShaderID::from(self.compute_shader.len() as u32);

        self.compute_shader.push(TTComputeShader {
            module: cs_module,
            pipeline: compute_pipeline,
            binding_map: bind_map,
            work_group_size: wg_size,
        });

        Ok(id)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorkGroupSize {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
pub struct TTComputeHandler<'ctx, 'rf, 'cs> {
    ctx: &'rf mut TexTransCoreEngineContext<'ctx>,
    compute_shader: &'cs TTComputeShader,

    bind_tex_view: HashMap<u32, wgpu::TextureView>,
    bind_buffer: HashMap<u32, wgpu::Buffer>,
}
impl TTComputeHandler<'_, '_, '_> {
    pub fn get_bind_index(&mut self, name: &str) -> Option<u32> {
        self.compute_shader.binding_map.get(name).copied()
    }

    pub fn set_render_texture(&mut self, bind_index: u32, render_texture: &TTRenderTexture) {
        let tex_view = render_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.bind_tex_view.insert(bind_index, tex_view);
    }

    pub fn upload_buffer(
        &mut self,
        bind_index: u32,
        buffer_data_span: &[u8],
        is_constants: bool,
    ) {
        if self.bind_buffer.contains_key(&bind_index) {
            //前のバッファーにもう一度詰めて送る方法わかんなかったから破棄
            let _ = self.bind_buffer.remove(&bind_index).unwrap();
        }

        let label = if is_constants {
            format!("{}-constant buffer", bind_index)
        } else {
            format!("{}-storage buffer", bind_index)
        };
        let buffer_desc = wgpu::util::BufferInitDescriptor {
            label: Some(label.as_str()),
            usage: if is_constants {
                wgpu::BufferUsages::UNIFORM
            } else {
                wgpu::BufferUsages::STORAGE
            },
            contents: buffer_data_span,
        };
        let buffer = self.ctx.engine.device.create_buffer_init(&buffer_desc);
        self.bind_buffer.insert(bind_index, buffer);
    }

    pub fn get_work_group_size(&self) -> WorkGroupSize {
        self.compute_shader.work_group_size
    }

    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        let tex_entries = self.bind_tex_view.iter().map(|t| wgpu::BindGroupEntry {
            binding: t.0.clone(),
            resource: wgpu::BindingResource::TextureView(t.1),
        });
        let buffer_entries = self.bind_buffer.iter().map(|b| wgpu::BindGroupEntry {
            binding: b.0.clone(),
            resource: b.1.as_entire_binding(),
        });

        let entries: Vec<_> = tex_entries.chain(buffer_entries).collect();

        let bind_group = self
            .ctx
            .engine
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("dispatch compte from handler"),
                layout: &self.compute_shader.pipeline.get_bind_group_layout(0),
                entries: &entries,
            });

        let encoder = self.ctx.get_command_encoder_as_mut();
        {
            let mut compute_pass = encoder.begin_compute_pass(&Default::default());

            compute_pass.set_pipeline(&self.compute_shader.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(x, y, z);
        }
        // self.ctx.send_command();
    }
}

impl<'ctx> TexTransCoreEngineContext<'ctx> {
    pub fn get_compute_handler<'rf>(
        &'rf mut self,
        id: &TTComputeShaderID,
    ) -> Result<TTComputeHandler<'ctx, 'rf, 'ctx>, &str> {
        let Some(compute_shader) = self.engine.compute_shader.get(*id.deref() as usize) else {
            return Err("un registered id");
        };

        Ok(TTComputeHandler {
            ctx: self,
            compute_shader: compute_shader,
            bind_tex_view: HashMap::new(),
            bind_buffer: HashMap::new(),
        })
    }
}

pub const BLENDING_SHADER_TEMPLATE: &str = r#"
RWTexture2D<float4> AddTex;
RWTexture2D<float4> DistTex;

[numthreads(16, 16, 1)] void CSMain(uint3 id : SV_DispatchThreadID)
{
    DistTex[id.xy] = ColorBlend( DistTex[id.xy] , AddTex[id.xy] );
}
"#;

pub(crate) fn spv_to_wgsl_and_binding_descriptor(
    spv: Vec<u8>,
) -> Result<(String, Vec<(String, u32)>, WorkGroupSize), Box<dyn Error>> {
    let naga_il_parse_result =
        naga::front::spv::parse_u8_slice(&spv, &naga::front::spv::Options::default());
    if let Err(er) = naga_il_parse_result {
        return Err(Box::new(er));
    }
    let mut naga_il = naga_il_parse_result.unwrap();

    for e in naga_il.entry_points.iter_mut() {
        if e.workgroup_size == [32, 32, 1] {
            e.workgroup_size = [16, 16, 1]
        }
    }

    let entry = naga_il.entry_points.first().unwrap();
    let wg_size = WorkGroupSize {
        x: entry.workgroup_size[0],
        y: entry.workgroup_size[1],
        z: entry.workgroup_size[2],
    };

    // println!("{:?}", naga_il);

    let bindings: Vec<_> = naga_il
        .global_variables
        .iter()
        .filter_map(|gv_h| {
            let gv = gv_h.1;

            if gv.name.is_none() || gv.binding.is_none() {
                return None;
            }
            if gv.binding.as_ref().unwrap().group != 0 {
                panic!("not supported binding group is not 0")
            }

            Some((
                String::from(gv.name.as_ref().unwrap().as_str()),
                // group: gv.binding.as_ref().unwrap().group,
                gv.binding.as_ref().unwrap().binding,
                // variable_type: naga_il.types[gv.ty].clone(),
            ))
        })
        .collect();

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::empty(),
        naga::valid::Capabilities::empty(),
    );
    let validate_info_result = validator.validate(&naga_il);
    if let Err(er) = validate_info_result {
        return Err(Box::new(er));
    }
    let validate_info = validate_info_result.unwrap();
    let convert_wgsl_string_result = naga::back::wgsl::write_string(
        &naga_il,
        &validate_info,
        naga::back::wgsl::WriterFlags::empty(),
    );
    if let Err(er) = convert_wgsl_string_result {
        return Err(Box::new(er));
    }

    let wgsl_string_with_32float = convert_wgsl_string_result.unwrap();

    Ok((wgsl_string_with_32float, bindings, wg_size))
}

fn convert_wgsl_format(str: String, format: wgpu::TextureFormat) -> String {
    // テクスチャの形式をごり押しで書き換える。すべて取り出したら f32 になる形式なので問題ない。
    // 正直できるなら naga IL レベルで書き換えたくもなるが、書き換える良い手段が実装できなかった...
    match format {
        wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba16Unorm
        | wgpu::TextureFormat::Rgba16Float => str.replace(WGSL_RGBA32FLOAT, get_format_str(format)),
        _ => str,
    }
}

pub(crate) fn get_format_str(format: wgpu::TextureFormat) -> &'static str {
    match format {
        wgpu::TextureFormat::Rgba8Unorm => WGSL_RGBA8UNORM,
        wgpu::TextureFormat::Rgba16Unorm => WGSL_RGBA16UNORM,
        wgpu::TextureFormat::Rgba16Float => WGSL_RGBA16FLOAT,
        wgpu::TextureFormat::Rgba32Float => WGSL_RGBA32FLOAT,
        wgpu::TextureFormat::Depth32Float => WGSL_R32FLOAT,
        _ => panic!(),
    }
}

const WGSL_RGBA8UNORM: &str = "rgba8unorm";
const WGSL_RGBA16UNORM: &str = "rgba16unorm";
const WGSL_RGBA16FLOAT: &str = "rgba16float";
const WGSL_RGBA32FLOAT: &str = "rgba32float";
const WGSL_R32FLOAT: &str = "r32float";
