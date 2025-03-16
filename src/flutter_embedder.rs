#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

include!(concat!(env!("OUT_DIR"), "/flutter_embedder_bindings.rs"));

use libloading::{Library, Symbol};
use std::path::Path;
use tracing::error;

pub enum FlutterEngineMode {
    JIT,
    AOT,
}

static FLUTTER_ENGINE_JIT_PATH: &str =
    "C:/libs/flutter/engine/src/out/host_debug/flutter_engine.dll";
static FLUTTER_ENGINE_AOT_PATH: &str =
    "C:/libs/flutter/engine/src/out/host_release/flutter_engine.dll";

pub fn load_flutter_engine(mode: FlutterEngineMode) -> Result<Library, Box<dyn std::error::Error>> {
    let engine_path = match mode {
        FlutterEngineMode::JIT => std::path::Path::new(FLUTTER_ENGINE_JIT_PATH),
        FlutterEngineMode::AOT => std::path::Path::new(FLUTTER_ENGINE_AOT_PATH),
    };

    if !engine_path.exists() {
        error!("Engine not found at path: {}", engine_path.display());
        return Err(format!("Engine not found at path: {}", engine_path.display()).into());
    }

    let lib = unsafe { Library::new(engine_path)? };
    Ok(lib)
}
