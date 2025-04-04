use std::{
    env,
    error::Error,
    ffi::OsStr,
    fmt::{Debug, Display},
    fs::{self},
    path::PathBuf,
};

use hassle_rs::{Dxc, DxcIncludeHandler, HassleError};

pub struct DirectXCompilerContext {
    dxc: Dxc,
    dxc_lib: hassle_rs::DxcLibrary,
    dxc_compiler: hassle_rs::DxcCompiler,
}

impl DirectXCompilerContext {
    pub fn new() -> Result<Self, DirectXCompilerContextError> {
        let lib_path_name = libloading::library_filename("dxcompiler");
        let current_dir = env::current_dir().unwrap();
        let Some(dll_path) = find_reclusive(current_dir, lib_path_name.as_os_str()) else {
            return Err(DirectXCompilerContextError::LibraryNotFound);
        };

        let Ok(dxc) = Dxc::new(Some(dll_path)) else {
            return Err(DirectXCompilerContextError::LibraryNotFound);
        };

        let dxc_lib = match dxc.create_library() {
            Ok(dxc_lib) => dxc_lib,
            Err(hassle_lib_error) => {
                return Err(DirectXCompilerContextError::HassleError(hassle_lib_error))
            }
        };
        let dxc_compiler = match dxc.create_compiler() {
            Ok(dxc_c) => dxc_c,
            Err(hassle_lib_error) => {
                return Err(DirectXCompilerContextError::HassleError(hassle_lib_error))
            }
        };

        Ok(DirectXCompilerContext {
            dxc,
            dxc_lib,
            dxc_compiler,
        })
    }

    pub fn compile_hlsl(
        &self,
        source_name: &str,
        shader_text: &str,
        entry_point: &str,
        target_profile: &str,
        args: &[&str],
        defines: &[(&str, Option<&str>)],
    ) -> Result<Vec<u8>, DirectXCompilerCompilingError> {
        let blob = match self.dxc_lib.create_blob_with_encoding_from_str(shader_text) {
            Ok(blob) => blob,
            Err(error) => return Err(DirectXCompilerCompilingError::HassleError(error)),
        };

        let result = self.dxc_compiler.compile(
            &blob,
            source_name,
            entry_point,
            target_profile,
            args,
            Some(&mut TTCEDefaultIncludeHandler {}),
            defines,
        );

        match result {
            Ok(compile_result) => {
                let op_result = match compile_result.get_result() {
                    Ok(op_result) => op_result,
                    Err(e) => return Err(DirectXCompilerCompilingError::HassleError(e)),
                };
                return Ok(op_result.to_vec());
            }
            Err((compile_result, _)) => {
                let compile_error = match compile_result.get_error_buffer() {
                    Ok(ce) => ce,
                    Err(e) => return Err(DirectXCompilerCompilingError::HassleError(e)),
                };

                Err(
                    match self.dxc_lib.get_blob_as_string(&compile_error.into()) {
                        Ok(str) => DirectXCompilerCompilingError::CompileError(str),
                        Err(err) => DirectXCompilerCompilingError::HassleError(err),
                    },
                )
            }
        }
    }
}

fn find_reclusive(dir: PathBuf, target_file_name: &OsStr) -> Option<PathBuf> {
    let Ok(open_dir) = dir.read_dir() else {
        return None;
    };
    for entry in open_dir.into_iter().filter_map(Result::ok) {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let entry_path = entry.path();
        if file_type.is_dir() {
            if let Some(rec_find_result) = find_reclusive(entry_path, target_file_name) {
                return Some(rec_find_result);
            }
        } else if file_type.is_file() {
            let Some(file_name) = entry_path.file_name() else {
                continue;
            };
            if file_name == target_file_name {
                return Some(entry_path);
            }
        }
    }
    return None;
}

#[derive(Debug)]
pub enum DirectXCompilerContextError {
    LibraryNotFound,
    HassleError(HassleError),
}
#[derive(Debug)]
pub enum DirectXCompilerCompilingError {
    CompileError(String),
    HassleError(HassleError),
}
impl Display for DirectXCompilerCompilingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self));
        Ok(())
    }
}
impl Error for DirectXCompilerCompilingError {}
impl Debug for DirectXCompilerContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectXCompilerContext")
            .field("dxc", &self.dxc)
            .finish()
    }
}

struct TTCEDefaultIncludeHandler {}

impl DxcIncludeHandler for TTCEDefaultIncludeHandler {
    fn load_source(&mut self, filename: String) -> Option<String> {
        match fs::read_to_string(filename) {
            Ok(file_contents) => return Some(file_contents),
            Err(_) => None,
        }
    }
}
