fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("ttce_wgpu_rust_core")
        .csharp_namespace("net.rs64.TexTransCoreEngineForWgpu")
        .csharp_class_name("NativeMethod")
        .generate_csharp_file("../TTCE-Wgpu/TTCEWgpuRustCore.g.cs")
        .unwrap();
}
