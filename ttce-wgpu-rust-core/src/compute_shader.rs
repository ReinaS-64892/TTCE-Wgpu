use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use naga::TypeInner::Image;
use naga::{ImageClass, ImageDimension, Module, StorageFormat};
use wgpu::util::DeviceExt;
use wgpu::{ComputePipeline, ShaderModule};

use crate::render_texture::TTRenderTexture;
use crate::storage_buffer::TTStorageBuffer;
use crate::tex_trans_core_engine::{
    TTCEWgpuError, TexTransCoreEngineContext, TexTransCoreEngineDevice,
};
use crate::{debug_log, TexTransCoreTextureFormat};

#[derive(Debug)]
pub struct TTComputeShader {
    #[allow(dead_code)]
    pub(crate) module: ShaderModule,

    pub(crate) pipeline: ComputePipeline,
    pub(crate) binding_map: HashMap<String, u32>,
    pub(crate) binding_type: HashMap<u32, TTBindingType>,
    pub(crate) work_group_size: WorkGroupSize,
}
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub struct TTComputeShaderID(u32);

impl TTComputeShaderID {
    pub fn from(id: u32) -> TTComputeShaderID {
        TTComputeShaderID(id)
    }
}

impl Deref for TTComputeShaderID {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl TexTransCoreEngineDevice {
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

        let spv = self.dx_compiler().compile_hlsl(
            hlsl_file_path,
            hlsl_string.as_str(),
            "CSMain",
            "cs_6_0",
            &["-spirv", "-HV 2018"],
            // &["-spirv", "-HV 2018","-O0"],
            &[],
        )?;

        let mut naga_ir =
            naga::front::spv::parse_u8_slice(&spv, &naga::front::spv::Options::default())?;

        fix_storage_texture_format(&mut naga_ir, self.default_texture_format());
        clamp_work_group_size(&mut naga_ir);

        let wg_size = get_work_group_size(&naga_ir);
        let bind_map = HashMap::from_iter(get_bindings(&naga_ir));
        let bind_type = HashMap::from_iter(get_binding_types(&naga_ir));

        // let mut validator = naga::valid::Validator::new(
        //     naga::valid::ValidationFlags::empty(),
        //     // naga::valid::ValidationFlags::all(),
        //     naga::valid::Capabilities::STORAGE_TEXTURE_16BIT_NORM_FORMATS,
        // );
        // let validate_info = validator.validate(&naga_ir)?;

        // let wgsl_string = naga::back::wgsl::write_string(
        //     &naga_ir,
        //     &validate_info,
        //     naga::back::wgsl::WriterFlags::empty(),
        // )?;

        // debug_log(operator_name.as_str());
        // debug_log(hlsl_string.as_str());
        // debug_log(format!("{:?}", naga_ir).as_str());

        let cs_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some((String::from("shade module with ") + &operator_name).as_str()),
                source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(naga_ir)),
                // source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(wgsl_string)),
            });
        let compute_pipeline =
            self.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some((String::from("compute pipeline with ") + &operator_name).as_str()),
                    layout: None,
                    module: &cs_module,
                    entry_point: Some("CSMain"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                });

        let id = TTComputeShaderID::from(self.compute_shader.len() as u32);

        self.compute_shader.push(TTComputeShader {
            module: cs_module,
            pipeline: compute_pipeline,
            binding_map: bind_map,
            binding_type: bind_type,
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
    bind_constants_buffer: HashMap<u32, wgpu::Buffer>,
    bind_storage_buffer: HashMap<u32, Arc<wgpu::Buffer>>,
}
impl TTComputeHandler<'_, '_, '_> {
    pub fn get_bind_index(&mut self, name: &str) -> Option<u32> {
        self.compute_shader.binding_map.get(name).copied()
    }

    pub fn upload_constants_buffer(
        &mut self,
        bind_index: u32,
        buffer_data_span: &[u8],
    ) -> Result<(), TTCEWgpuError> {
        let Some(bind_type) = self.compute_shader.binding_type.get(&bind_index) else {
            return Err(TTCEWgpuError::BindingNotFound);
        };
        if *bind_type != TTBindingType::ConstantsBuffer {
            return Err(TTCEWgpuError::BindingIsNotConstantsBuffer);
        }

        if self.bind_constants_buffer.contains_key(&bind_index) {
            let _ = self.bind_constants_buffer.remove(&bind_index).unwrap();
        }

        let label = format!("{}-storage buffer", bind_index);
        let buffer_desc = wgpu::util::BufferInitDescriptor {
            label: Some(label.as_str()),
            usage: wgpu::BufferUsages::UNIFORM,
            contents: buffer_data_span,
        };
        let buffer = self.ctx.engine.device.create_buffer_init(&buffer_desc);
        self.bind_constants_buffer.insert(bind_index, buffer);

        Ok(())
    }
    pub fn set_storage_buffer(
        &mut self,
        bind_index: u32,
        buffer: &TTStorageBuffer,
    ) -> Result<(), TTCEWgpuError> {
        let Some(bind_type) = self.compute_shader.binding_type.get(&bind_index) else {
            return Err(TTCEWgpuError::BindingNotFound);
        };
        if *bind_type != TTBindingType::StorageBuffer {
            return Err(TTCEWgpuError::BindingIsNotStorageBuffer);
        }

        if self.bind_storage_buffer.contains_key(&bind_index) {
            let _ = self.bind_storage_buffer.remove(&bind_index).unwrap();
        }

        self.bind_storage_buffer
            .insert(bind_index, buffer.buffer.clone());

        Ok(())
    }

    pub fn set_render_texture(
        &mut self,
        bind_index: u32,
        render_texture: &TTRenderTexture,
    ) -> Result<(), TTCEWgpuError> {
        let Some(bind_type) = self.compute_shader.binding_type.get(&bind_index) else {
            return Err(TTCEWgpuError::BindingNotFound);
        };
        if *bind_type != TTBindingType::RWRenderTexture {
            return Err(TTCEWgpuError::BindingIsNotRWTexture);
        }

        let tex_view = render_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.bind_tex_view.insert(bind_index, tex_view);
        Ok(())
    }

    pub fn get_work_group_size(&self) -> WorkGroupSize {
        self.compute_shader.work_group_size
    }

    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        let tex_entries = self.bind_tex_view.iter().map(|t| wgpu::BindGroupEntry {
            binding: *t.0,
            resource: wgpu::BindingResource::TextureView(t.1),
        });
        let constants_buffer_entries =
            self.bind_constants_buffer
                .iter()
                .map(|b| wgpu::BindGroupEntry {
                    binding: *b.0,
                    resource: b.1.as_entire_binding(),
                });
        let storage_buffer_entries =
            self.bind_storage_buffer
                .iter()
                .map(|ab| wgpu::BindGroupEntry {
                    binding: *ab.0,
                    resource: ab.1.as_ref().as_entire_binding(),
                });

        let entries: Vec<_> = tex_entries
            .chain(constants_buffer_entries)
            .chain(storage_buffer_entries)
            .collect();

        let bind_group = self
            .ctx
            .engine
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("dispatch compte from handler"),
                layout: &self.compute_shader.pipeline.get_bind_group_layout(0),
                entries: &entries,
            });

        {
            let encoder = self.ctx.get_command_encoder_as_mut();
            let mut compute_pass = encoder.begin_compute_pass(&Default::default());

            compute_pass.set_pipeline(&self.compute_shader.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(x, y, z);
        }
        self.ctx.check_command_stack();
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
            compute_shader,
            bind_tex_view: HashMap::new(),
            bind_constants_buffer: HashMap::new(),
            bind_storage_buffer: HashMap::new(),
        })
    }
}

fn fix_storage_texture_format(naga_ir: &mut Module, tt_format: TexTransCoreTextureFormat) {
    for gv in naga_ir.global_variables.iter_mut() {
        let ref_gv = gv.1;

        let origin_type = &naga_ir.types[ref_gv.ty];

        let Image {
            dim: id,
            arrayed: ad,
            class: ic,
        } = origin_type.inner
        else {
            continue;
        };
        if ad || id != ImageDimension::D2 {
            continue;
        }

        let naga::ImageClass::Storage {
            format: fm,
            access: ac,
        } = ic
        else {
            continue;
        };

        if fm != StorageFormat::Rgba32Float {
            continue;
        }

        let new_type = naga::Type {
            name: origin_type.name.clone(),
            inner: Image {
                dim: id,
                arrayed: ad,
                class: ImageClass::Storage {
                    format: TTRenderTexture::to_naga_storage_texture_format(
                        tt_format,
                        crate::TexTransCoreTextureChannel::RGBA,
                    ),
                    access: ac,
                },
            },
        };

        ref_gv.ty = naga_ir
            .types
            .insert(new_type, naga_ir.types.get_span(ref_gv.ty));
    }
}

fn get_bindings(naga_ir: &Module) -> Vec<(String, u32)> {
    naga_ir
        .global_variables
        .iter()
        .filter_map(|gv_h| {
            let gv = gv_h.1;

            if gv.name.is_none() || gv.binding.is_none() {
                return None;
            }
            if gv.binding.as_ref()?.group != 0 {
                debug_log("not supported binding group is not 0");
                return None;
            }

            Some((
                String::from(gv.name.as_ref().unwrap().as_str()),
                gv.binding.as_ref().unwrap().binding,
            ))
        })
        .collect()
}
fn get_binding_types(naga_ir: &Module) -> Vec<(u32, TTBindingType)> {
    naga_ir
        .global_variables
        .iter()
        .filter_map(|gv_h| {
            let gv = gv_h.1;

            if gv.name.is_none() || gv.binding.is_none() {
                return None;
            }
            if gv.binding.as_ref()?.group != 0 {
                debug_log("not supported binding group is not 0");
                return None;
            }

            let bind_type = match naga_ir.types[gv.ty].inner {
                naga::TypeInner::Struct { .. } => match gv.space {
                    naga::AddressSpace::Uniform { .. } => Some(TTBindingType::ConstantsBuffer),
                    naga::AddressSpace::Storage { .. } => Some(TTBindingType::StorageBuffer),
                    _ => None,
                },
                naga::TypeInner::Image { .. } => Some(TTBindingType::RWRenderTexture),
                _ => None,
            }?;

            Some((gv.binding.as_ref().unwrap().binding, bind_type))
        })
        .collect()
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TTBindingType {
    ConstantsBuffer,
    StorageBuffer,
    RWRenderTexture,
}

fn get_work_group_size(naga_ir: &Module) -> WorkGroupSize {
    let entry = naga_ir.entry_points.first().unwrap();

    WorkGroupSize {
        x: entry.workgroup_size[0],
        y: entry.workgroup_size[1],
        z: entry.workgroup_size[2],
    }
}

fn clamp_work_group_size(naga_ir: &mut Module) {
    for e in naga_ir.entry_points.iter_mut() {
        if e.workgroup_size == [32, 32, 1] {
            e.workgroup_size = [16, 16, 1]
        }
    }
}
pub trait AsTypeStr {
    fn as_type_str(&self) -> &'static str;
}
impl AsTypeStr for wgpu::TextureFormat {
    fn as_type_str(&self) -> &'static str {
        match self {
            wgpu::TextureFormat::Rgba8Unorm => WGSL_RGBA8UNORM,
            wgpu::TextureFormat::Rgba16Unorm => WGSL_RGBA16UNORM,
            wgpu::TextureFormat::Rgba16Float => WGSL_RGBA16FLOAT,
            wgpu::TextureFormat::Rgba32Float => WGSL_RGBA32FLOAT,
            _ => panic!(),
        }
    }
}

const WGSL_RGBA8UNORM: &str = "rgba8unorm";
const WGSL_RGBA16UNORM: &str = "rgba16unorm";
const WGSL_RGBA16FLOAT: &str = "rgba16float";
const WGSL_RGBA32FLOAT: &str = "rgba32float";
