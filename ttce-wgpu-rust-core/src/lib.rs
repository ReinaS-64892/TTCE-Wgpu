mod compute_shader;
mod render_texture;
mod tex_trans_core_engine;

use std::{
    default,
    ffi::c_void,
    ops::Deref,
    sync::{atomic::AtomicPtr, Mutex},
};

use compute_shader::{TTComputeHandler, TTComputeShaderID};
use once_cell::sync::OnceCell;
use render_texture::TTRenderTexture;
use tex_trans_core_engine::{TexTransCoreEngin, TexTransCoreEngineContext};
use wgpu::{Backends, DeviceType};

static DEBUG_LOG: Mutex<Option<unsafe extern "C" fn(*const u16, i32) -> ()>> = Mutex::new(None);
#[no_mangle]
pub extern "C" fn set_debug_log_pointer(
    debug_log_fn_ptr: unsafe extern "C" fn(*const u16, i32) -> (),
) {
    *DEBUG_LOG.lock().unwrap() = if debug_log_fn_ptr as usize == 0 {
        None
    } else {
        Some(debug_log_fn_ptr)
    };
}
pub fn debug_log(str: &str) {
    let ptr = DEBUG_LOG.lock().unwrap();
    let Some(fn_ptr) = *ptr else {
        return;
    };

    let utf_16_str: Vec<_> = str.encode_utf16().collect();
    unsafe {
        fn_ptr(utf_16_str.as_ptr(), utf_16_str.len() as i32);
    }
}

static TOKIO_RUNTIME: OnceCell<tokio::runtime::Runtime> = OnceCell::new();
fn create_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
}
fn get_tokio_runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RUNTIME.get_or_init(create_tokio_runtime)
}

// TexTransCoreEngine
#[repr(u32)]
pub enum RequestDevicePreference {
    Auto,
    DiscreteGPU,
    IntegratedGPUOrCPU,
}

#[no_mangle]
pub extern "C" fn create_tex_trans_engine_device(
    preference: RequestDevicePreference,
) -> *mut c_void {
    let (device, queue) = get_tokio_runtime()
        .block_on(async move {
            let instance = wgpu::Instance::default();
            let adapter = match preference {
                RequestDevicePreference::Auto => Some(
                    instance
                        .request_adapter(&wgpu::RequestAdapterOptions::default())
                        .await
                        .unwrap(),
                ),
                RequestDevicePreference::IntegratedGPUOrCPU => instance
                    .enumerate_adapters(Backends::all())
                    .into_iter()
                    .find(|a| {
                        let device_type = a.get_info().device_type;
                        return device_type == DeviceType::IntegratedGpu
                            || device_type == DeviceType::Cpu;
                    }),
                RequestDevicePreference::DiscreteGPU => instance
                    .enumerate_adapters(Backends::all())
                    .into_iter()
                    .find(|a| {
                        let device_type = a.get_info().device_type;
                        return device_type == DeviceType::DiscreteGpu;
                    }),
            };
            let adapter = if let Some(adapter) = adapter {
                adapter
            } else {
                let request_adapter_option = wgpu::RequestAdapterOptions::default();
                instance
                    .request_adapter(&request_adapter_option)
                    .await
                    .unwrap()
            };

            debug_log(&format!("Adapter : \n{:?}", adapter.get_info()));

            let mut device_feature = wgpu::DeviceDescriptor::default();
            device_feature.required_features =
                wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                    | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM;
            adapter.request_device(&device_feature, None).await
        })
        .unwrap();

    let ttce = tex_trans_core_engine::TexTransCoreEngin::new(device, queue);

    Box::into_raw(Box::new(ttce)) as *mut c_void
}
#[no_mangle]
pub extern "C" fn set_default_texture_format(
    tex_trans_core_engine_ptr: *mut c_void,
    format: TexTransCoreTextureFormat,
) {
    let engine = unsafe {
        (tex_trans_core_engine_ptr as *mut TexTransCoreEngin)
            .as_mut()
            .unwrap()
    };
    engine.set_default_texture_format(format);
}
#[no_mangle]
pub unsafe extern "C" fn drop_tex_trans_engine_device(tex_trans_core_engine_ptr: *mut c_void) {
    let _ = Box::from_raw(tex_trans_core_engine_ptr as *mut TexTransCoreEngin);
}

#[no_mangle]
pub extern "C" fn register_format_convertor(tex_trans_core_engine_ptr: *mut c_void) {
    let engine = unsafe {
        (tex_trans_core_engine_ptr as *mut TexTransCoreEngin)
            .as_mut()
            .unwrap()
    };
    engine.register_format_convertor();
}

// retune of tt_compute_shader_id
#[no_mangle]
pub unsafe extern "C" fn register_compute_shader_from_hlsl(
    tex_trans_core_engine_ptr: *mut c_void,
    hlsl_path: *const u16,
    hlsl_path_str_len: i32,
    hlsl_path_source: *const u16,
    hlsl_path_source_str_len: i32,
) -> u32 {
    let engine = (tex_trans_core_engine_ptr as *mut TexTransCoreEngin)
        .as_mut()
        .unwrap();

    let path_slice = std::slice::from_raw_parts(hlsl_path, hlsl_path_str_len as usize);
    let hlsl_path_rust_string = String::from_utf16(path_slice).unwrap();

    let source_slice_rust_string_opt = if std::ptr::null() != hlsl_path_source {
        let source_slice =
            std::slice::from_raw_parts(hlsl_path_source, hlsl_path_source_str_len as usize);
        let source_slice_rust_string = String::from_utf16(source_slice).unwrap();

        Some(source_slice_rust_string)
    } else {
        None
    };

    let source_slice_rust_str_opt = if let Some(string) = source_slice_rust_string_opt.as_ref() {
        Some(string.as_str())
    } else {
        None
    };

    let id = engine
        .register_compute_shader_from_hlsl(
            hlsl_path_rust_string.as_str(),
            source_slice_rust_str_opt,
        )
        .unwrap();

    *id.deref()
}

// TexTransCoreEngineContext

#[no_mangle]
pub unsafe extern "C" fn get_ttce_context(tex_trans_core_engine_ptr: *const c_void) -> *mut c_void {
    let engine = (tex_trans_core_engine_ptr as *const TexTransCoreEngin)
        .as_ref()
        .unwrap();

    Box::into_raw(Box::from(engine.create_ctx())) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn drop_ttce_context(ttce_context_ptr: *mut c_void) {
    let _ = Box::from_raw(ttce_context_ptr as *mut TexTransCoreEngineContext);
}

#[no_mangle]
pub unsafe extern "C" fn get_render_texture(
    ttce_context_ptr: *mut c_void,
    width: u32,
    height: u32,
    channel: TexTransCoreTextureChannel,
) -> *mut c_void {
    let engine_ctx = (ttce_context_ptr as *mut TexTransCoreEngineContext)
        .as_mut()
        .unwrap();

    Box::into_raw(Box::from(
        engine_ctx.get_render_texture(width, height, channel),
    )) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn drop_render_texture(render_texture_ptr: *mut c_void) {
    let _ = Box::from_raw(render_texture_ptr as *mut TTRenderTexture);
}

#[no_mangle]
pub unsafe extern "C" fn get_width(render_texture_ptr: *mut c_void) -> u32 {
    let from_render_texture = (render_texture_ptr as *const TTRenderTexture)
        .as_ref()
        .unwrap();

    from_render_texture.width()
}

#[no_mangle]
pub unsafe extern "C" fn get_height(render_texture_ptr: *mut c_void) -> u32 {
    let from_render_texture = (render_texture_ptr as *const TTRenderTexture)
        .as_ref()
        .unwrap();

    from_render_texture.height()
}

//Upload Download to render texture
#[derive(Clone, Copy)]
#[repr(i32)]
pub enum TexTransCoreTextureChannel {
    R = 1,
    RG = 2,
    // RGB = 3,
    RGBA = 4,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum TexTransCoreTextureFormat {
    Byte = 0,
    UShort = 1,
    Half = 2,
    Float = 3,
}

#[no_mangle]
pub unsafe extern "C" fn copy_texture(
    ttce_context_ptr: *mut c_void,
    dist_render_texture_ptr: *const c_void,
    source_render_texture_ptr: *const c_void,
) {
    let engine_ctx = (ttce_context_ptr as *mut TexTransCoreEngineContext)
        .as_mut()
        .unwrap();
    let dist_render_texture = (dist_render_texture_ptr as *const TTRenderTexture)
        .as_ref()
        .unwrap();
    let source_render_texture = (source_render_texture_ptr as *const TTRenderTexture)
        .as_ref()
        .unwrap();

    engine_ctx.copy_texture(&dist_render_texture, &source_render_texture);
}
#[no_mangle]
pub unsafe extern "C" fn upload_texture(
    ttce_context_ptr: *mut c_void,
    render_texture_ptr: *const c_void,
    data: *const u8,
    data_len: i32,
    format: TexTransCoreTextureFormat,
) {
    let engine_ctx = (ttce_context_ptr as *mut TexTransCoreEngineContext)
        .as_mut()
        .unwrap();
    let data_slice = std::slice::from_raw_parts(data, data_len as usize);
    let render_texture = (render_texture_ptr as *const TTRenderTexture)
        .as_ref()
        .unwrap();

    engine_ctx.upload_texture(&render_texture, data_slice, format);
}
#[no_mangle]
pub unsafe extern "C" fn download_texture(
    ttce_context_ptr: *mut c_void,
    write_data: *mut u8,
    write_data_len: i32,
    format: TexTransCoreTextureFormat,
    render_texture_ptr: *const c_void,
) {
    let engine_ctx = (ttce_context_ptr as *mut TexTransCoreEngineContext)
        .as_mut()
        .unwrap();
    let data_slice = std::slice::from_raw_parts_mut(write_data, write_data_len as usize);
    let render_texture = (render_texture_ptr as *const TTRenderTexture)
        .as_ref()
        .unwrap();

    get_tokio_runtime().block_on(async move {
        let buffer = engine_ctx
            .download_texture(render_texture, Some(format))
            .await
            .unwrap();

        let buffer_slice = buffer.slice(..);
        let buffer_mapped = buffer_slice.get_mapped_range();

        data_slice.copy_from_slice(&buffer_mapped);
    });
}

// TTComputeHandler
#[no_mangle]
pub unsafe extern "C" fn get_compute_handler(
    ttce_context_ptr: *mut c_void,
    tt_compute_shader_id: u32,
) -> *mut c_void {
    let engine_ctx = (ttce_context_ptr as *mut TexTransCoreEngineContext)
        .as_mut()
        .unwrap();

    Box::into_raw(Box::from(
        engine_ctx
            .get_compute_handler(&TTComputeShaderID::from(tt_compute_shader_id))
            .unwrap(),
    )) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn drop_compute_handler(tt_compute_handler_ptr: *mut c_void) {
    let _ = Box::from_raw(tt_compute_handler_ptr as *mut TTComputeHandler);
}
#[no_mangle]
pub unsafe extern "C" fn get_bind_index(
    tt_compute_handler_ptr: *const c_void,
    name_ptr: *const u16,
    name_ptr_len: i32,
) -> GetBindIndexResult {
    let compute_handler = (tt_compute_handler_ptr as *mut TTComputeHandler)
        .as_mut()
        .unwrap();

    let name_slice = std::slice::from_raw_parts(name_ptr, name_ptr_len as usize);
    let name_rust_string = String::from_utf16(name_slice).unwrap();

    if let Some(i) = compute_handler.get_bind_index(name_rust_string.as_str()) {
        GetBindIndexResult {
            result: true,
            bind_index: i,
        }
    } else {
        GetBindIndexResult {
            result: false,
            bind_index: 0,
        }
    }
}
#[repr(C)]
pub struct GetBindIndexResult {
    result: bool,
    bind_index: u32,
}

#[no_mangle]
pub unsafe extern "C" fn set_render_texture(
    tt_compute_handler_ptr: *mut c_void,
    bind_index: u32,
    render_texture_ptr: *const c_void,
) {
    let compute_handler = (tt_compute_handler_ptr as *mut TTComputeHandler)
        .as_mut()
        .unwrap();

    let render_texture = (render_texture_ptr as *const TTRenderTexture)
        .as_ref()
        .unwrap();

    compute_handler.set_render_texture(bind_index, render_texture);
}

#[no_mangle]
pub unsafe extern "C" fn upload_constants_buffer(
    tt_compute_handler_ptr: *mut c_void,
    bind_index: u32,
    buffer: *const u8,
    buffer_len: i32,
) {
    let compute_handler = (tt_compute_handler_ptr as *mut TTComputeHandler)
        .as_mut()
        .unwrap();

    let buffer = std::slice::from_raw_parts(buffer, buffer_len as usize);

    compute_handler.upload_buffer(bind_index, buffer, true);
}
#[no_mangle]
pub unsafe extern "C" fn upload_storage_buffer(
    tt_compute_handler_ptr: *mut c_void,
    bind_index: u32,
    buffer: *const u8,
    buffer_len: i32,
) {
    let compute_handler = (tt_compute_handler_ptr as *mut TTComputeHandler)
        .as_mut()
        .unwrap();

    let buffer = std::slice::from_raw_parts(buffer, buffer_len as usize);

    compute_handler.upload_buffer(bind_index, buffer, false);
}

#[repr(C)]
pub struct WorkGroupSize {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
impl WorkGroupSize {
    fn from(wgs: compute_shader::WorkGroupSize) -> Self {
        WorkGroupSize {
            x: wgs.x,
            y: wgs.y,
            z: wgs.z,
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_work_group_size(
    tt_compute_handler_ptr: *const c_void,
) -> WorkGroupSize {
    let compute_handler = (tt_compute_handler_ptr as *const TTComputeHandler)
        .as_ref()
        .unwrap();
    WorkGroupSize::from(compute_handler.get_work_group_size())
}

#[no_mangle]
pub unsafe extern "C" fn dispatch(tt_compute_handler_ptr: *mut c_void, x: u32, y: u32, z: u32) {
    let compute_handler = (tt_compute_handler_ptr as *mut TTComputeHandler)
        .as_mut()
        .unwrap();

    compute_handler.dispatch(x, y, z);
}
