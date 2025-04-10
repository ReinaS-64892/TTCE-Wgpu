mod compute_shader;
mod dxc_ctx;
mod render_texture;
mod storage_buffer;
mod tex_trans_core_engine;

use std::{ffi::c_void, ops::Deref, sync::Mutex};

use compute_shader::{TTComputeHandler, TTComputeShaderID};
use dxc_ctx::DirectXCompilerContext;
use once_cell::sync::OnceCell;
use render_texture::TTRenderTexture;
use storage_buffer::TTStorageBuffer;
use tex_trans_core_engine::{TexTransCoreEngineContext, TexTransCoreEngineDevice};
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
        .expect("tokio runtime initializing failed !?")
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

/// TexTransCoreEngineDevice を生成し、ポインターを得ることができる。
#[no_mangle]
pub extern "C" fn create_tex_trans_core_engine_device(
    preference: RequestDevicePreference,
) -> *mut c_void {
    let (device, queue) = get_tokio_runtime()
        .block_on(async move {
            let instance = wgpu::Instance::default();
            let adapter = match preference {
                RequestDevicePreference::Auto => instance
                    .enumerate_adapters(Backends::DX12 | Backends::VULKAN | Backends::METAL)// OpenGL 系を排除する
                    .into_iter()
                    .find(|_| true),// 何かもっといい手段ないのか？
                // Some(
                //     instance
                //         .request_adapter(&wgpu::RequestAdapterOptions::default())
                //         .await
                //         .expect("adapter request failed when ttce device creation"),
                // ),
                RequestDevicePreference::IntegratedGPUOrCPU => instance
                    .enumerate_adapters(Backends::all())
                    .into_iter()
                    .find(|a| {
                        let device_type = a.get_info().device_type;
                        device_type == DeviceType::IntegratedGpu || device_type == DeviceType::Cpu
                    }),
                RequestDevicePreference::DiscreteGPU => instance
                    .enumerate_adapters(Backends::all())
                    .into_iter()
                    .find(|a| {
                        let device_type = a.get_info().device_type;
                        device_type == DeviceType::DiscreteGpu
                    }),
            };
            let adapter = if let Some(adapter) = adapter {
                adapter
            } else {
                let request_adapter_option = wgpu::RequestAdapterOptions::default();
                instance
                    .request_adapter(&request_adapter_option)
                    .await
                    .expect("fallback adapter request failed when ttce device creation")
            };

            debug_log(&format!("Adapter : \n{:?}", adapter.get_info()));

            let device_feature = wgpu::DeviceDescriptor {
                required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                    | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM,
                required_limits: wgpu::Limits {
                    max_storage_textures_per_shader_stage: 8,
                    max_bind_groups: 1,
                    ..Default::default()
                },
                ..Default::default()
            };

            adapter.request_device(&device_feature, None).await
        })
        .unwrap();

    let dxc_ctx = DirectXCompilerContext::new().expect("DirectCompilerContext creation failed");

    let ttce = tex_trans_core_engine::TexTransCoreEngineDevice::new(device, queue, dxc_ctx);

    Box::into_raw(Box::new(ttce)) as *mut c_void
}

/// tex_trans_core_engine_ptr は TexTransCoreEngineDevice のポインターでないといけない。
/// TexTransCoreEngineDevice のポインターを受け取り、そのデバイスの内部で使われるデフォルトのフォーマットを指定する。
/// 初期化時に行うようか、 TexTransCoreEngineContext が一つもぶら下がっていないときに行うように。
#[no_mangle]
pub extern "C" fn set_default_texture_format(
    tex_trans_core_engine_ptr: *mut c_void,
    format: TexTransCoreTextureFormat,
) {
    let engine = unsafe {
        (tex_trans_core_engine_ptr as *mut TexTransCoreEngineDevice)
            .as_mut()
            .unwrap()
    };
    engine.set_default_texture_format(format);
}

/// # Safety
/// tex_trans_core_engine_ptr は TexTransCoreEngineDevice のポインターでないといけない。
/// Context や TTRenderTexture などぶら下がってる物をすべてドロップしてから呼ぶように。
#[no_mangle]
pub unsafe extern "C" fn drop_tex_trans_core_engine_device(tex_trans_core_engine_ptr: *mut c_void) {
    let _ = Box::from_raw(tex_trans_core_engine_ptr as *mut TexTransCoreEngineDevice);
}

/// tex_trans_core_engine_ptr は TexTransCoreEngineDevice のポインターでないといけない。
/// TexTransCoreEngineDevice に内部的に使用するフォーマットコンバータを生成させる。
/// set_default_texture_format と同様、処理を始める前やしていないタイミングで行うように。
#[no_mangle]
pub extern "C" fn register_format_convertor(tex_trans_core_engine_ptr: *mut c_void) {
    let engine = unsafe {
        (tex_trans_core_engine_ptr as *mut TexTransCoreEngineDevice)
            .as_mut()
            .unwrap()
    };
    engine.register_format_convertor();
}

// retune of tt_compute_shader_id

/// # Safety
/// tex_trans_core_engine_ptr は TexTransCoreEngineDevice のポインターでないといけない。
/// 任意の HLSL を UTF16 (C# string) をコンピュートシェーダーとして登録させることができ、hlsl_path_source は null pointer でもよい。
/// 戻り値の値は result が true の時しか使用してはならない。 false の場合は何らかの理由で失敗している。ログに出力されたものを見るように。
#[no_mangle]
pub unsafe extern "C" fn register_compute_shader_from_hlsl(
    tex_trans_core_engine_ptr: *mut c_void,
    hlsl_path: *const u16,
    hlsl_path_str_len: i32,
    hlsl_path_source: *const u16,
    hlsl_path_source_str_len: i32,
) -> RegisterCSResult {
    let engine = (tex_trans_core_engine_ptr as *mut TexTransCoreEngineDevice)
        .as_mut()
        .unwrap();

    let hlsl_path_rust_string = String::from_utf16(std::slice::from_raw_parts(
        hlsl_path,
        hlsl_path_str_len as usize,
    ))
    .unwrap();

    let source_slice_rust_string_opt = (!hlsl_path_source.is_null()).then(|| {
        String::from_utf16(std::slice::from_raw_parts(
            hlsl_path_source,
            hlsl_path_source_str_len as usize,
        ))
        .unwrap()
    });

    let try_id = engine.register_compute_shader_from_hlsl(
        hlsl_path_rust_string.as_str(),
        source_slice_rust_string_opt.as_deref(),
    );

    match try_id {
        Ok(id) => RegisterCSResult {
            result: true,
            compute_shader_id: *id.deref(),
        },
        Err(err) => {
            debug_log(err.to_string().as_str());
            RegisterCSResult {
                result: false,
                compute_shader_id: 0,
            }
        }
    }
}
#[repr(C)]
pub struct RegisterCSResult {
    result: bool,
    compute_shader_id: u32,
}

// TexTransCoreEngineContext

/// # Safety
/// tex_trans_core_engine_ptr は TexTransCoreEngineDevice のポインターでないといけない。
/// TexTransCoreEngineContext を生成し、それのポインターを得ることができる。
/// 処理が始まる前に行うべきことを行ってから作ることを推奨。
#[no_mangle]
pub unsafe extern "C" fn get_ttce_context(tex_trans_core_engine_ptr: *const c_void) -> *mut c_void {
    let engine = (tex_trans_core_engine_ptr as *const TexTransCoreEngineDevice)
        .as_ref()
        .unwrap();

    Box::into_raw(Box::from(engine.create_ctx())) as *mut c_void
}

/// # Safety
/// ttce_context_ptr は TexTransCoreEngineContext でないといけない。
#[no_mangle]
pub unsafe extern "C" fn drop_ttce_context(ttce_context_ptr: *mut c_void) {
    let _ = Box::from_raw(ttce_context_ptr as *mut TexTransCoreEngineContext);
}

/// # Safety
/// ttce_context_ptr は TexTransCoreEngineContext のポインターでないといけない。
/// TTRenderTexture のポインターを得る事ができる。
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

/// # Safety
///  TTRenderTexture のポインターでないといけない。
#[no_mangle]
pub unsafe extern "C" fn drop_render_texture(render_texture_ptr: *mut c_void) {
    let _ = Box::from_raw(render_texture_ptr as *mut TTRenderTexture);
}

/// # Safety
///  TTRenderTexture のポインターでないといけない。
#[no_mangle]
pub unsafe extern "C" fn get_width(render_texture_ptr: *mut c_void) -> u32 {
    let from_render_texture = (render_texture_ptr as *const TTRenderTexture)
        .as_ref()
        .unwrap();

    from_render_texture.width()
}

/// # Safety
///  TTRenderTexture のポインターでないといけない。
#[no_mangle]
pub unsafe extern "C" fn get_height(render_texture_ptr: *mut c_void) -> u32 {
    let from_render_texture = (render_texture_ptr as *const TTRenderTexture)
        .as_ref()
        .unwrap();

    from_render_texture.height()
}

//Upload Download to render texture
#[derive(Clone, Copy, PartialEq, Eq)]
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

/// # Safety
/// 二つの TTRenderTexture のポインターでなければならない。
/// 形式の変換や解像度のリサイズなどは一切行えないので注意。
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

    engine_ctx.copy_texture(dist_render_texture, source_render_texture);
}

/// # Safety
/// ttce_context_ptr は TexTransCoreEngineContext
/// render_texture_ptr は TTRenderTexture
/// data は 配列の先頭 のポインター
/// data_len を format と 書き込み先の解像度と正しく長さが合うようにしなければならない。
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

    engine_ctx.upload_texture(render_texture, data_slice, format);
}

/// # Safety
/// ttce_context_ptr は TexTransCoreEngineContext
/// render_texture_ptr は TTRenderTexture
/// write_data は 配列の先頭 のポインター
/// write_data_len を format と 書き込み先の解像度と正しく長さが合うようにしなければならない。
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

/// # Safety
/// tt_compute_handler_ptr は TTComputeHandler のポインターを割り当てるように。
/// TTStorageBuffer への pointer が得られる。
#[no_mangle]
pub unsafe extern "C" fn allocate_storage_buffer(
    ttce_context_ptr: *const c_void,
    buffer_len: i32,
    downloadable: bool,
) -> *mut c_void {
    let engine_ctx = (ttce_context_ptr as *const TexTransCoreEngineContext)
        .as_ref()
        .unwrap();

    Box::into_raw(Box::from(
        engine_ctx.allocate_storage_buffer(buffer_len, downloadable),
    )) as *mut c_void
}
/// # Safety
/// tt_compute_handler_ptr は TTComputeHandler のポインター、 buffer は アップロードしたい 配列の先頭の のポインターでないといけない。
/// TTStorageBuffer への pointer が得られる。
#[no_mangle]
pub unsafe extern "C" fn upload_storage_buffer(
    ttce_context_ptr: *const c_void,
    buffer: *const u8,
    buffer_len: i32,
    downloadable: bool,
) -> *mut c_void {
    let engine_ctx = (ttce_context_ptr as *const TexTransCoreEngineContext)
        .as_ref()
        .unwrap();

    let buffer = std::slice::from_raw_parts(buffer, buffer_len as usize);

    Box::into_raw(Box::from(
        engine_ctx.upload_storage_buffer(buffer, downloadable),
    )) as *mut c_void
}
/// # Safety
/// tt_compute_handler_ptr は TTComputeHandler のポインターでないといけない。
#[no_mangle]
pub unsafe extern "C" fn drop_storage_buffer(storage_buffer_ptr: *mut c_void) {
    let _ = Box::from_raw(storage_buffer_ptr as *mut TTStorageBuffer);
}

/// # Safety
/// tt_compute_handler_ptr は TTComputeHandler のポインターでないといけない。
#[no_mangle]
pub unsafe extern "C" fn download_storage_buffer(
    ttce_context_ptr: *mut c_void,
    buffer: *mut u8,
    buffer_len: i32,
    storage_buffer_ptr: *const c_void,
) {
    let engine_ctx = (ttce_context_ptr as *mut TexTransCoreEngineContext)
        .as_mut()
        .unwrap();
    let storage_buffer = (storage_buffer_ptr as *mut TTStorageBuffer)
        .as_ref()
        .unwrap();

    let buffer = std::slice::from_raw_parts_mut(buffer, buffer_len as usize);

    get_tokio_runtime()
        .block_on(engine_ctx.download_storage_buffer(storage_buffer))
        .unwrap();

    let storage_buffer_slice = storage_buffer
        .buffer
        .slice(..storage_buffer.buffer.size().min(buffer_len as u64));
    let storage_buffer_mapped = storage_buffer_slice.get_mapped_range();

    buffer.copy_from_slice(&storage_buffer_mapped);
}

// TTComputeHandler

/// # Safety
/// ttce_context_ptr は TexTransCoreEngineContext のポインターでないといけないし、
/// tt_compute_shader_id は register_compute_shader_from_hlsl から得られる u32 でないといけない。
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

/// # Safety
/// tt_compute_handler_ptr は TTComputeHandler のポインターでないといけない。
#[no_mangle]
pub unsafe extern "C" fn drop_compute_handler(tt_compute_handler_ptr: *mut c_void) {
    let _ = Box::from_raw(tt_compute_handler_ptr as *mut TTComputeHandler);
}

/// # Safety
/// tt_compute_handler_ptr は TTComputeHandler のポインターでないといけない。
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
/// # Safety
/// tt_compute_handler_ptr は TTComputeHandler のポインター、render_texture_ptr は TTRenderTexture のポインターでないといけない。
/// bind_index は get_bind_index から得た値を使うように。
#[no_mangle]
pub unsafe extern "C" fn set_render_texture(
    tt_compute_handler_ptr: *mut c_void,
    bind_index: u32,
    render_texture_ptr: *const c_void,
) -> bool {
    let compute_handler = (tt_compute_handler_ptr as *mut TTComputeHandler)
        .as_mut()
        .unwrap();

    let render_texture = (render_texture_ptr as *const TTRenderTexture)
        .as_ref()
        .unwrap();

    let result = compute_handler.set_render_texture(bind_index, render_texture);

    if let Err(e) = result {
        debug_log(format!("{:?}", e).as_str());
    }

    result.is_ok()
}

/// # Safety
/// tt_compute_handler_ptr は TTComputeHandler のポインター、 buffer は アップロードしたい 配列の先頭の のポインターでないといけない。
/// bind_index は get_bind_index から得た値を使うように。
#[no_mangle]
pub unsafe extern "C" fn upload_constants_buffer(
    tt_compute_handler_ptr: *mut c_void,
    bind_index: u32,
    buffer: *const u8,
    buffer_len: i32,
) -> bool {
    let compute_handler = (tt_compute_handler_ptr as *mut TTComputeHandler)
        .as_mut()
        .unwrap();

    let buffer = std::slice::from_raw_parts(buffer, buffer_len as usize);

    let result = compute_handler.upload_constants_buffer(bind_index, buffer);

    if let Err(e) = result {
        debug_log(format!("{:?}", e).as_str());
    }

    result.is_ok()
}

/// # Safety
/// tt_compute_handler_ptr は TTComputeHandler のポインター、 buffer は アップロードしたい 配列の先頭の のポインターでないといけない。
/// bind_index は get_bind_index から得た値を使うように。
#[no_mangle]
pub unsafe extern "C" fn set_storage_buffer(
    tt_compute_handler_ptr: *mut c_void,
    bind_index: u32,
    storage_buffer_ptr: *mut c_void,
) -> bool {
    let compute_handler = (tt_compute_handler_ptr as *mut TTComputeHandler)
        .as_mut()
        .unwrap();

    let storage_buffer = (storage_buffer_ptr as *mut TTStorageBuffer)
        .as_mut()
        .unwrap();

    let result = compute_handler.set_storage_buffer(bind_index, storage_buffer);

    if let Err(e) = result {
        debug_log(format!("{:?}", e).as_str());
    }

    result.is_ok()
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

/// # Safety
/// tt_compute_handler_ptr は TTComputeHandler のポインター
#[no_mangle]
pub unsafe extern "C" fn get_work_group_size(
    tt_compute_handler_ptr: *const c_void,
) -> WorkGroupSize {
    let compute_handler = (tt_compute_handler_ptr as *const TTComputeHandler)
        .as_ref()
        .unwrap();
    WorkGroupSize::from(compute_handler.get_work_group_size())
}

/// # Safety
/// tt_compute_handler_ptr は TTComputeHandler のポインター
#[no_mangle]
pub unsafe extern "C" fn dispatch(tt_compute_handler_ptr: *mut c_void, x: u32, y: u32, z: u32) {
    let compute_handler = (tt_compute_handler_ptr as *mut TTComputeHandler)
        .as_mut()
        .unwrap();

    compute_handler.dispatch(x, y, z);
}
