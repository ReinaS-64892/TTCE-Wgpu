use std::{collections::HashMap, ops::Deref};

use crate::{
    compute_shader::{AsTypeStr, TTComputeShader, TTComputeShaderID, WorkGroupSize},
    tex_trans_core_engine::{
        RequestFormat, TTRtRequestDescriptor, TexTransCoreEngineContext, TexTransCoreEngineDevice,
    },
    TexTransCoreTextureChannel, TexTransCoreTextureFormat,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ConvertTextureFormat {
    pub from: wgpu::TextureFormat,
    pub to: wgpu::TextureFormat,
}

#[derive(Debug)]
pub struct TTRenderTexture {
    pub texture: wgpu::Texture,
}
impl TTRenderTexture {
    pub(crate) fn get_request_descriptor(&self) -> TTRtRequestDescriptor {
        let format = Self::from_wgpu_texture_format(self.format()).unwrap();
        TTRtRequestDescriptor {
            width: self.width(),
            height: self.height(),
            format: RequestFormat::Manual(format.0, format.1),
        }
    }

    pub fn tt_format(&self) -> Option<(TexTransCoreTextureFormat, TexTransCoreTextureChannel)> {
        Self::from_wgpu_texture_format(self.format())
    }
    pub(crate) fn from_wgpu_texture_format(
        format: wgpu::TextureFormat,
    ) -> Option<(TexTransCoreTextureFormat, TexTransCoreTextureChannel)> {
        match format {
            wgpu::TextureFormat::R8Unorm => Some((
                TexTransCoreTextureFormat::Byte,
                TexTransCoreTextureChannel::R,
            )),
            wgpu::TextureFormat::Rg8Unorm => Some((
                TexTransCoreTextureFormat::Byte,
                TexTransCoreTextureChannel::RG,
            )),
            wgpu::TextureFormat::Rgba8Unorm => Some((
                TexTransCoreTextureFormat::Byte,
                TexTransCoreTextureChannel::RGBA,
            )),

            wgpu::TextureFormat::R16Unorm => Some((
                TexTransCoreTextureFormat::UShort,
                TexTransCoreTextureChannel::R,
            )),
            wgpu::TextureFormat::Rg16Unorm => Some((
                TexTransCoreTextureFormat::UShort,
                TexTransCoreTextureChannel::RG,
            )),
            wgpu::TextureFormat::Rgba16Unorm => Some((
                TexTransCoreTextureFormat::UShort,
                TexTransCoreTextureChannel::RGBA,
            )),

            wgpu::TextureFormat::R16Float => Some((
                TexTransCoreTextureFormat::Half,
                TexTransCoreTextureChannel::R,
            )),
            wgpu::TextureFormat::Rg16Float => Some((
                TexTransCoreTextureFormat::Half,
                TexTransCoreTextureChannel::RG,
            )),
            wgpu::TextureFormat::Rgba16Float => Some((
                TexTransCoreTextureFormat::Half,
                TexTransCoreTextureChannel::RGBA,
            )),

            wgpu::TextureFormat::R32Float => Some((
                TexTransCoreTextureFormat::Float,
                TexTransCoreTextureChannel::R,
            )),
            wgpu::TextureFormat::Rg32Float => Some((
                TexTransCoreTextureFormat::Float,
                TexTransCoreTextureChannel::RG,
            )),
            wgpu::TextureFormat::Rgba32Float => Some((
                TexTransCoreTextureFormat::Float,
                TexTransCoreTextureChannel::RGBA,
            )),
            _ => None,
        }
    }
    pub(crate) fn to_wgpu_texture_format(
        format: TexTransCoreTextureFormat,
        channel: TexTransCoreTextureChannel,
    ) -> wgpu::TextureFormat {
        match (format, channel) {
            (TexTransCoreTextureFormat::Byte, TexTransCoreTextureChannel::R) => {
                wgpu::TextureFormat::R8Unorm
            }
            (TexTransCoreTextureFormat::Byte, TexTransCoreTextureChannel::RG) => {
                wgpu::TextureFormat::Rg8Unorm
            }
            (TexTransCoreTextureFormat::Byte, TexTransCoreTextureChannel::RGBA) => {
                wgpu::TextureFormat::Rgba8Unorm
            }

            (TexTransCoreTextureFormat::UShort, TexTransCoreTextureChannel::R) => {
                wgpu::TextureFormat::R16Unorm
            }
            (TexTransCoreTextureFormat::UShort, TexTransCoreTextureChannel::RG) => {
                wgpu::TextureFormat::Rg16Unorm
            }
            (TexTransCoreTextureFormat::UShort, TexTransCoreTextureChannel::RGBA) => {
                wgpu::TextureFormat::Rgba16Unorm
            }

            (TexTransCoreTextureFormat::Half, TexTransCoreTextureChannel::R) => {
                wgpu::TextureFormat::R16Float
            }
            (TexTransCoreTextureFormat::Half, TexTransCoreTextureChannel::RG) => {
                wgpu::TextureFormat::Rg16Float
            }
            (TexTransCoreTextureFormat::Half, TexTransCoreTextureChannel::RGBA) => {
                wgpu::TextureFormat::Rgba16Float
            }

            (TexTransCoreTextureFormat::Float, TexTransCoreTextureChannel::R) => {
                wgpu::TextureFormat::R32Float
            }
            (TexTransCoreTextureFormat::Float, TexTransCoreTextureChannel::RG) => {
                wgpu::TextureFormat::Rg32Float
            }
            (TexTransCoreTextureFormat::Float, TexTransCoreTextureChannel::RGBA) => {
                wgpu::TextureFormat::Rgba32Float
            }
        }
    }

    pub(crate) fn to_naga_storage_texture_format(
        format: TexTransCoreTextureFormat,
        channel: TexTransCoreTextureChannel,
    ) -> naga::StorageFormat {
        match (format, channel) {
            (TexTransCoreTextureFormat::Byte, TexTransCoreTextureChannel::R) => {
                naga::StorageFormat::R8Unorm
            }
            (TexTransCoreTextureFormat::Byte, TexTransCoreTextureChannel::RG) => {
                naga::StorageFormat::Rg8Unorm
            }
            (TexTransCoreTextureFormat::Byte, TexTransCoreTextureChannel::RGBA) => {
                naga::StorageFormat::Rgba8Unorm
            }

            (TexTransCoreTextureFormat::UShort, TexTransCoreTextureChannel::R) => {
                naga::StorageFormat::R16Unorm
            }
            (TexTransCoreTextureFormat::UShort, TexTransCoreTextureChannel::RG) => {
                naga::StorageFormat::Rg16Unorm
            }
            (TexTransCoreTextureFormat::UShort, TexTransCoreTextureChannel::RGBA) => {
                naga::StorageFormat::Rgba16Unorm
            }

            (TexTransCoreTextureFormat::Half, TexTransCoreTextureChannel::R) => {
                naga::StorageFormat::R16Float
            }
            (TexTransCoreTextureFormat::Half, TexTransCoreTextureChannel::RG) => {
                naga::StorageFormat::Rg16Float
            }
            (TexTransCoreTextureFormat::Half, TexTransCoreTextureChannel::RGBA) => {
                naga::StorageFormat::Rgba16Float
            }

            (TexTransCoreTextureFormat::Float, TexTransCoreTextureChannel::R) => {
                naga::StorageFormat::R32Float
            }
            (TexTransCoreTextureFormat::Float, TexTransCoreTextureChannel::RG) => {
                naga::StorageFormat::Rg32Float
            }
            (TexTransCoreTextureFormat::Float, TexTransCoreTextureChannel::RGBA) => {
                naga::StorageFormat::Rgba32Float
            }
        }
    }
}
impl Deref for TTRenderTexture {
    type Target = wgpu::Texture;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

// Upload Download Copy Util
impl TexTransCoreEngineContext<'_> {
    pub fn copy_texture(&mut self, dist: &TTRenderTexture, src: &TTRenderTexture) {
        if dist.width() != src.width() {
            panic!("Size Different!!!")
        }
        if dist.height() != src.height() {
            panic!("Size Different!!!")
        }

        let encoder = self.get_command_encoder_as_mut();

        encoder.copy_texture_to_texture(
            src.as_image_copy(),
            dist.as_image_copy(),
            wgpu::Extent3d {
                width: dist.width(),
                height: dist.height(),
                depth_or_array_layers: 1,
            },
        );
    }

    fn convert_to_copy(&mut self, dist: &TTRenderTexture, src: &TTRenderTexture) {
        let converter_id = self
            .engine
            .converter_id
            .get(&ConvertTextureFormat {
                from: src.format(),
                to: dist.format(),
            })
            .unwrap();

        // println!("{:?}", converter_id);

        let mut converter_handler = self.get_compute_handler(converter_id).unwrap();

        let src_index = converter_handler.get_bind_index("SrcTex").unwrap();
        converter_handler.set_render_texture(src_index, src);
        let dist_index = converter_handler.get_bind_index("DistTex").unwrap();
        converter_handler.set_render_texture(dist_index, dist);

        let wg_size = converter_handler.get_work_group_size();
        converter_handler.dispatch(
            (dist.width() / wg_size.x).max(1),
            (dist.height() / wg_size.y).max(1),
            1,
        );
    }

    pub fn upload_texture(
        &mut self,
        target: &TTRenderTexture,
        data: &[u8],
        data_format: TexTransCoreTextureFormat,
    ) {
        let (target_format, target_channel) =
            TTRenderTexture::from_wgpu_texture_format(target.format()).unwrap();
        let pixel_par_byte = TTRenderTexture::to_wgpu_texture_format(data_format, target_channel)
            .block_copy_size(None)
            .unwrap();

        let data_size = target.width() * target.height() * pixel_par_byte;

        if data.len() as u32 != data_size {
            panic!("Data Size is Different")
        }

        let data_layout = wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(target.width() * pixel_par_byte),
            rows_per_image: None,
        };
        let data_size = wgpu::Extent3d {
            width: target.width(),
            height: target.height(),
            depth_or_array_layers: 1,
        };

        if data_format == target_format {
            self.send_command();
            self.engine
                .queue
                .write_texture(target.as_image_copy(), data, data_layout, data_size);
        } else {
            let copy_src = self.get_render_texture_with(&TTRtRequestDescriptor {
                width: target.width(),
                height: target.height(),
                format: RequestFormat::Manual(data_format, target_channel),
            });

            self.engine
                .queue
                .write_texture(copy_src.as_image_copy(), data, data_layout, data_size);

            self.convert_to_copy(target, &copy_src);
        }
    }

    pub async fn download_texture(
        &mut self,
        target: &TTRenderTexture,
        download_format: Option<TexTransCoreTextureFormat>,
    ) -> Option<wgpu::Buffer> {
        let (target_format, target_channel) =
            TTRenderTexture::from_wgpu_texture_format(target.format()).unwrap();

        let is_format_different = if let Some(f) = download_format {
            f != target_format
        } else {
            false
        };

        let download_pixel_par_byte = if let Some(df) = download_format {
            TTRenderTexture::to_wgpu_texture_format(df, target_channel)
                .block_copy_size(None)
                .unwrap()
        } else {
            TTRenderTexture::to_wgpu_texture_format(target_format, target_channel)
                .block_copy_size(None)
                .unwrap()
        };

        let read_back_buffer = self.engine.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("read-back-buffer"),
            size: (target.width() * target.height() * download_pixel_par_byte) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        if is_format_different {
            let mut desc = target.get_request_descriptor();
            desc.format = RequestFormat::Manual(download_format.unwrap(), target_channel);
            let convert_temp = self.get_render_texture_with(&desc);
            self.convert_to_copy(&convert_temp, target);
            self.download_impl(&convert_temp, &read_back_buffer, download_pixel_par_byte);
        } else {
            self.download_impl(target, &read_back_buffer, download_pixel_par_byte);
        };

        // let timer = Instant::now();
        let rb_buffer_slice = read_back_buffer.slice(..);
        let (sender, receiver) = tokio::sync::oneshot::channel();
        rb_buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
            sender.send(v).unwrap();
        });

        self.engine
            .device
            .poll(wgpu::Maintain::wait())
            .panic_on_timeout();

        if receiver.await.unwrap().is_ok() {
            // let end = timer.elapsed();
            // debug_log(&format!("readback-{}ms", end.as_millis()));
            Some(read_back_buffer)
        } else {
            None
        }
    }

    fn download_impl(
        &mut self,
        render_texture: &TTRenderTexture,
        read_back_buffer: &wgpu::Buffer,
        download_pixel_par_byte: u32,
    ) {
        let encoder = self.get_command_encoder_as_mut();
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: render_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: read_back_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(render_texture.width() * download_pixel_par_byte),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: render_texture.width(),
                height: render_texture.height(),
                depth_or_array_layers: render_texture.depth_or_array_layers(),
            },
        );
        self.send_command();
    }
}

impl TexTransCoreEngineContext<'_> {
    pub fn get_render_texture(
        &self,
        width: u32,
        height: u32,
        channel: TexTransCoreTextureChannel,
    ) -> TTRenderTexture {
        self.engine.create_render_texture(&TTRtRequestDescriptor {
            width,
            height,
            format: RequestFormat::AutoWithChannel(channel),
        })
    }
    pub(crate) fn get_render_texture_with(
        &self,
        arg_desc: &TTRtRequestDescriptor,
    ) -> TTRenderTexture {
        self.engine.create_render_texture(arg_desc)
    }
}
impl TexTransCoreEngineDevice {
    pub(crate) fn register_format_convertor(&mut self) {
        let mut bind_map = HashMap::new();
        bind_map.insert("SrcTex".to_string(), 0_u32);
        bind_map.insert("DistTex".to_string(), 1_u32);

        for cv in FORMAT_TABLE {
            let from_format = cv.from;
            let to_format = cv.to;
            let wgsl_str = FORMAT_CONVERTER_TEMPLATE
                .replace("$$$FROM$$$", from_format.as_type_str())
                .replace("$$$TO$$$", to_format.as_type_str());

            let cs_module = self
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("format convertor shade module"),
                    source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&wgsl_str)),
                });
            let compute_pipeline =
                self.device
                    .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some("format convertor compute pipeline"),
                        layout: None,
                        module: &cs_module,
                        entry_point: Some("CSMain"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        cache: None,
                    });

            let id = TTComputeShaderID::from(self.compute_shader.len() as u32);
            // println!("{id:?}-/ {wgsl_str}");

            self.compute_shader.push(TTComputeShader {
                module: cs_module,
                pipeline: compute_pipeline,
                binding_map: bind_map.clone(),
                work_group_size: WorkGroupSize { x: 16, y: 16, z: 1 },
            });

            self.converter_id.insert(*cv, id);
        }
    }
}

pub const FORMAT_TABLE: &[ConvertTextureFormat] = &[
    ConvertTextureFormat {
        from: wgpu::TextureFormat::Rgba8Unorm,
        to: wgpu::TextureFormat::Rgba16Unorm,
    },
    ConvertTextureFormat {
        from: wgpu::TextureFormat::Rgba8Unorm,
        to: wgpu::TextureFormat::Rgba16Float,
    },
    ConvertTextureFormat {
        from: wgpu::TextureFormat::Rgba8Unorm,
        to: wgpu::TextureFormat::Rgba32Float,
    },
    ConvertTextureFormat {
        from: wgpu::TextureFormat::Rgba16Unorm,
        to: wgpu::TextureFormat::Rgba8Unorm,
    },
    ConvertTextureFormat {
        from: wgpu::TextureFormat::Rgba16Unorm,
        to: wgpu::TextureFormat::Rgba16Float,
    },
    ConvertTextureFormat {
        from: wgpu::TextureFormat::Rgba16Unorm,
        to: wgpu::TextureFormat::Rgba32Float,
    },
    ConvertTextureFormat {
        from: wgpu::TextureFormat::Rgba16Float,
        to: wgpu::TextureFormat::Rgba8Unorm,
    },
    ConvertTextureFormat {
        from: wgpu::TextureFormat::Rgba16Float,
        to: wgpu::TextureFormat::Rgba16Unorm,
    },
    ConvertTextureFormat {
        from: wgpu::TextureFormat::Rgba16Float,
        to: wgpu::TextureFormat::Rgba32Float,
    },
    ConvertTextureFormat {
        from: wgpu::TextureFormat::Rgba32Float,
        to: wgpu::TextureFormat::Rgba8Unorm,
    },
    ConvertTextureFormat {
        from: wgpu::TextureFormat::Rgba32Float,
        to: wgpu::TextureFormat::Rgba16Unorm,
    },
    ConvertTextureFormat {
        from: wgpu::TextureFormat::Rgba32Float,
        to: wgpu::TextureFormat::Rgba16Float,
    },
];

pub const FORMAT_CONVERTER_TEMPLATE: &str = r#"
@group(0) @binding(0)
var SrcTex: texture_storage_2d<$$$FROM$$$,read>;
@group(0) @binding(1)
var DistTex: texture_storage_2d<$$$TO$$$,write>;

@compute @workgroup_size(16, 16, 1)
fn CSMain(@builtin(global_invocation_id) param: vec3<u32>) {
    let pos = param.xy;
    let col = textureLoad(SrcTex, pos);
    textureStore(DistTex, pos, col);
}
"#;
