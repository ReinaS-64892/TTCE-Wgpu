// mod compute_shader;
// mod render_texture;
// mod tex_trans_core_engine;

// use compute_shader::TTComputeShaderID;
// use image::ImageReader;
// use render_texture::TTRenderTexture;
// use tex_trans_core_engine::{TexTransCoreEngin, TexTransCoreEngineContext};
// use tokio;
// use wgpu::{self};

// #[tokio::main]
// async fn main() {
//     // let mut get_desc = wgpu::InstanceDescriptor::default();
//     // get_desc.backends = wgpu::Backends::DX12;
//     // let instance = wgpu::Instance::new(get_desc);
//     let instance = wgpu::Instance::default();

//     // let adapters = instance.enumerate_adapters(wgpu::Backends::all());

//     // for adp in adapters.iter() {
//     //     println!("{:?}", adp.get_info());
//     // }

//     println!("---");

//     // let adapter = adapters
//     //     .into_iter()
//     //     .find(|a| a.get_info().name.contains("2060"))
//     //     .unwrap();

//     let request_adapter_option = wgpu::RequestAdapterOptions::default();
//     // request_adapter_option.power_preference = PowerPreference::default();
//     // request_adapter_option.compatible_surface = None;
//     let adapter = instance
//         .request_adapter(&request_adapter_option)
//         .await
//         .unwrap();

//     println!("{:?}", adapter.get_info());

//     let mut device_feature = wgpu::DeviceDescriptor::default();
//     device_feature.required_features = wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
//         | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM;

//     let (device, queue) = adapter.request_device(&device_feature, None).await.unwrap();

//     let mut ttce = TexTransCoreEngin::new(device, queue);
//     ttce.register_format_convertor();
//     let level_adjustment_compute_id  = ttce.register_compute_shader_from_hlsl(r#"F:\unityproject\Lime - ReinaSEdit\Packages\TexTransTool\TexTransCore\ShaderAssets\GrabBlend\LevelAdjustment.ttcomp"#,None).unwrap();

//     let mut ctx = ttce.create_ctx();

//     // let operator_name = ttce.register_compute_operator_from_hlsl(r#"F:\unityproject\Lime - ReinaSEdit\Packages\TexTransTool\TexTransCore\ShaderAssets\GrabBlend\LevelAdjustment.ttcomp"#).unwrap();
//     // println!("{:?}", ttce.compute_operators.get(&operator_name));
//     // let operator_holder = ttce.create_compute_operator_from_hlsl(r#"F:\unityproject\Lime - ReinaSEdit\Packages\TexTransTool\TexTransCore\ShaderAssets\FillAlpha.ttcomp"#).unwrap();
//     // let operator_holder = ttce.create_compute_operator_from_hlsl(r#"F:\unityproject\Lime - ReinaSEdit\Packages\TexTransTool\TexTransCore\ShaderAssets\GrabBlend\HSVAdjustment.ttcomp"#).unwrap();

//     // let operator_name = ttce.register_compute_operator_from_hlsl(r#"F:\unityproject\Lime - ReinaSEdit\Packages\TexTransTool\TexTransCore\ShaderAssets\AlphaCopy.ttcomp"#).unwrap();
//     // println!("{:?}", ttce.compute_operators.get(&operator_name));

//     // let path = r#"F:\unityproject\Lime - ReinaSEdit\Packages\TexTransTool\TexTransCore\ShaderAssets\Blendings\StdLinearLight.ttblend"#;
//     // let mut hlsl_blend_code = String::new();
//     // File::open(path)
//     //     .unwrap()
//     //     .read_to_string(&mut hlsl_blend_code)
//     //     .unwrap();

//     // hlsl_blend_code.push_str(compute_shader::BLENDING_SHADER_TEMPLATE);

//     // let try_blending_cs_holder = ctx
//     //     .engine
//     //     .create_compute_shader_from_hlsl(path, Some(hlsl_blend_code.as_str()));

//     // if try_blending_cs_holder.is_err() {
//     //     println!("{}", try_blending_cs_holder.unwrap_err());
//     //     panic!();
//     // }

//     // let blending_cs_holder = try_blending_cs_holder.unwrap();

//     let img = ImageReader::open(r#"D:\Rs\TTCE-Wgpu\TestData\0-Hair.png"#).unwrap();
//     // let img2 = ImageReader::open(r#"D:\Rs\ttce-wgpu\src\0-SingleGradationDecal.png"#).unwrap();

//     let dyn_image = img.decode().unwrap();
//     // let dyn_image2 = img2.decode().unwrap();

//     let image_bytes = dyn_image.as_bytes();
//     // let image2_bytes = dyn_image2.as_bytes();

//     let rt = ctx.get_render_texture(2048, 2048, false);
//     // let mut rt2 = ctx.get_render_texture(&rt_desc);

//     ctx.upload_texture(
//         image_bytes,
//         render_texture::TexTransCoreTextureFormat::Rgba8Unorm,
//         &rt,
//     );
//     // ctx.upload_texture(image2_bytes, &mut rt2);

//     // blending(&mut ctx, &blending_cs_holder, &mut rt, &mut rt2);
//     level_adjustment(&mut ctx, &level_adjustment_compute_id, &rt);

//     let buffer = ctx
//         .download_texture(
//             &rt,
//             Some(render_texture::TexTransCoreTextureFormat::Rgba8Unorm),
//         )
//         .await
//         .unwrap();

//     // ctx.retune_render_texture(rt);

//     let buffer_slice = buffer.slice(..);
//     let buffer_mapped = buffer_slice.get_mapped_range();

//     let mut out_image = image::RgbaImage::new(2048, 2048);
//     out_image.copy_from_slice(&buffer_mapped);
//     out_image
//         .save(r#"D:\Rs\TTCE-Wgpu\TestData\0-Result.png"#)
//         .unwrap();

//     /*

//     let hlsl = include_str!("FillRed.hlsl");
//     let spv = hassle_rs::compile_hlsl(
//         "FillRed.hlsl",
//         hlsl,
//         "CSMain",
//         "cs_6_0",
//         &vec!["-spirv"],
//         &vec![],
//     )
//     .unwrap();
//     // let spv = include_bytes!("FillRed.spv");
//     let naga_ir =
//         naga::front::spv::parse_u8_slice(&spv, &naga::front::spv::Options::default()).unwrap();

//     let mut validator = naga::valid::Validator::new(
//         naga::valid::ValidationFlags::empty(),
//         naga::valid::Capabilities::empty(),
//     );
//     let validate_info = validator.validate(&naga_ir).unwrap();
//     let wgsl_string = naga::back::wgsl::write_string(
//         &naga_ir,
//         &validate_info,
//         naga::back::wgsl::WriterFlags::empty(),
//     )
//     .unwrap();

//     let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//         label: Some("fill red"),
//         source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&wgsl_string)),
//     });

//     let tex_desc = wgpu::TextureDescriptor {
//         size: wgpu::Extent3d {
//             width: 1024,
//             height: 1024,
//             depth_or_array_layers: 1,
//         },
//         dimension: wgpu::TextureDimension::D2,
//         format: wgpu::TextureFormat::Rgba32Float,
//         usage: wgpu::TextureUsages::TEXTURE_BINDING
//             | wgpu::TextureUsages::STORAGE_BINDING
//             | wgpu::TextureUsages::COPY_SRC,
//         mip_level_count: 1,
//         sample_count: 1,
//         label: Some("ttce-test-rt"),
//         view_formats: &[wgpu::TextureFormat::Rgba32Float],
//     };

//     let render_texture = device.create_texture(&tex_desc);

//     let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//         label: Some("ttce-test-compute-bind-group"),
//         layout: &compute_pipeline.get_bind_group_layout(0),
//         entries: &[wgpu::BindGroupEntry {
//             binding: 0,
//             resource: wgpu::BindingResource::TextureView(
//                 &render_texture.create_view(&wgpu::TextureViewDescriptor::default()),
//             ),
//         }],
//     });

//     let read_back_buffer = device.create_buffer(&wgpu::BufferDescriptor {
//         label: Some("read-back-buffer"),
//         size: (render_texture.width() * render_texture.height() * 4 * 4) as u64,
//         usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
//         mapped_at_creation: false,
//     });

//     let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

//     {
//         //Computing!!!
//         let mut pass: wgpu::ComputePass<'_> =
//             encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());

//         pass.set_pipeline(&compute_pipeline);
//         pass.set_bind_group(0, &bind_group, &[]);
//         pass.dispatch_workgroups(render_texture.width() / 16, render_texture.height() / 16, 1);
//     }

//     encoder.copy_texture_to_buffer(
//         wgpu::ImageCopyTextureBase {
//             texture: &render_texture,
//             mip_level: 0,
//             origin: wgpu::Origin3d::ZERO,
//             aspect: wgpu::TextureAspect::All,
//         },
//         wgpu::ImageCopyBufferBase {
//             buffer: &read_back_buffer,
//             layout: wgpu::ImageDataLayout {
//                 offset: 0,
//                 bytes_per_row: Some(render_texture.width() * 4 * 4),
//                 rows_per_image: None,
//             },
//         },
//         wgpu::Extent3d {
//             width: render_texture.width(),
//             height: render_texture.height(),
//             depth_or_array_layers: render_texture.depth_or_array_layers(),
//         },
//     );

//     queue.submit(Some(encoder.finish()));

//     let rb_buffer_slice = read_back_buffer.slice(..);
//     let (s, r) = tokio::sync::oneshot::channel();
//     rb_buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
//         s.send(v).unwrap();
//     });

//     device.poll(wgpu::Maintain::wait()).panic_on_timeout();

//     if let Ok(m) = r.await.unwrap() {
//         println!("{:?}", m);
//         let mut out_file = File::create("./Out.bin").unwrap();
//         out_file
//             .write_all(&rb_buffer_slice.get_mapped_range())
//             .unwrap();
//     }

//      */
// }

// fn blending<'ctx, 'rf, 'cs>(
//     ctx: &'rf mut TexTransCoreEngineContext<'ctx>,
//     blend_compute_id: &TTComputeShaderID,
//     dist_rt: &TTRenderTexture,
//     add_rt: &TTRenderTexture,
// ) {
//     let mut compute_holder = ctx.get_compute_handler(blend_compute_id).unwrap();

//     compute_holder.set_render_texture("DistTex", dist_rt);
//     compute_holder.set_render_texture("AddTex", add_rt);

//     let wg_size = compute_holder.get_work_group_size();
//     compute_holder.dispatch(
//         (dist_rt.texture.width() / wg_size.x).max(1),
//         (dist_rt.texture.height() / wg_size.y).max(1),
//         1,
//     );

//     /*
//     let tex_view_desc = wgpu::TextureViewDescriptor::default();

//     let add_tex_view = add_rt.texture.create_view(&tex_view_desc);
//     let dist_tex_view = dist_rt.texture.create_view(&tex_view_desc);
//     let add_bind_entry: wgpu::BindGroupEntry<'_> = blend_compute
//         .create_binding_entry("AddTex", wgpu::BindingResource::TextureView(&add_tex_view))
//         .unwrap();
//     let dist_bind_entry = blend_compute
//         .create_binding_entry(
//             "DistTex",
//             wgpu::BindingResource::TextureView(&dist_tex_view),
//         )
//         .unwrap();

//     let bind_group = ctx
//         .engine
//         .device
//         .create_bind_group(&wgpu::BindGroupDescriptor {
//             label: Some("blending binding"),
//             layout: &blend_compute.pipeline.get_bind_group_layout(0),
//             entries: &[add_bind_entry, dist_bind_entry],
//         });

//     let encoder = ctx.get_command_encoder_as_mut();
//     {
//         let mut compute_pass = encoder.begin_compute_pass(&Default::default());

//         compute_pass.set_pipeline(&blend_compute.pipeline);
//         compute_pass.set_bind_group(0, &bind_group, &[]);
//         compute_pass.dispatch_workgroups(
//             (dist_rt.texture.width() / 16).max(1),
//             (dist_rt.texture.height() / 16).max(1),
//             1,
//         );
//     }
//     */
// }

// fn level_adjustment<'ctx, 'rf, 'cs>(
//     ctx: &'rf mut TexTransCoreEngineContext<'ctx>,
//     level_compute_id: &TTComputeShaderID,
//     dist_rt: &TTRenderTexture,
// ) {
//     let mut compute_holder = ctx.get_compute_handler(level_compute_id).unwrap();

//     compute_holder.set_render_texture("Tex", dist_rt);

//     let input_floor = 0_f32;
//     let input_ceiling = 1_f32;
//     let gamma = 0.3_f32;
//     let output_floor = 0_f32;
//     let output_ceiling = 0.9_f32;
//     let r = 1_f32;
//     let g = 0_f32;
//     let b = 1_f32;

//     let mut data = [0_u8; 32];

//     data[0..(0 + 4)].copy_from_slice(&input_floor.to_le_bytes());
//     data[4..(4 + 4)].copy_from_slice(&input_ceiling.to_le_bytes());
//     data[8..(8 + 4)].copy_from_slice(&gamma.to_le_bytes());
//     data[12..(12 + 4)].copy_from_slice(&output_floor.to_le_bytes());
//     data[16..(16 + 4)].copy_from_slice(&output_ceiling.to_le_bytes());
//     data[20..(20 + 4)].copy_from_slice(&r.to_le_bytes());
//     data[24..(24 + 4)].copy_from_slice(&g.to_le_bytes());
//     data[28..(28 + 4)].copy_from_slice(&b.to_le_bytes());

//     compute_holder.set_constants_buffer("gv", &data);

//     let wg_size = compute_holder.get_work_group_size();
//     compute_holder.dispatch(
//         (dist_rt.texture.width() / wg_size.x).max(1),
//         (dist_rt.texture.height() / wg_size.y).max(1),
//         1,
//     );

//     data[20..(20 + 4)].copy_from_slice(&0_f32.to_le_bytes());
//     data[24..(24 + 4)].copy_from_slice(&1_f32.to_le_bytes());
//     data[28..(28 + 4)].copy_from_slice(&0_f32.to_le_bytes());

//     compute_holder.set_constants_buffer("gv", &data);

//     let wg_size = compute_holder.get_work_group_size();
//     compute_holder.dispatch(
//         (dist_rt.texture.width() / wg_size.x).max(1),
//         (dist_rt.texture.height() / wg_size.y).max(1),
//         1,
//     );
// }

fn main() {}
