// src/llm/providers/llama_cpp/vision.rs

use crate::error::{AmbiError, Result};
use base64::{engine::general_purpose, Engine as _};
use log::info;
use std::path::Path;

pub(crate) enum VisionContext {
    ExternalProjector {
        _clip_ctx_ptr: *mut std::ffi::c_void,
    },
    Integrated,
}

unsafe impl Send for VisionContext {}
unsafe impl Sync for VisionContext {}

impl VisionContext {
    pub fn init(mmproj_path: Option<&String>, integrated: bool) -> Result<Option<Self>> {
        if let Some(path) = mmproj_path {
            if !Path::new(path).exists() {
                return Err(AmbiError::EngineError(format!(
                    "Vision model not found: {}",
                    path
                )));
            }
            info!("Loading External Vision Projector (mmproj) from: {}", path);

            Ok(Some(Self::ExternalProjector {
                _clip_ctx_ptr: std::ptr::null_mut(),
            }))
        } else if integrated {
            info!("Vision capabilities are integrated natively into the main LLM.");
            Ok(Some(Self::Integrated))
        } else {
            Ok(None)
        }
    }

    pub fn decode_base64_image(base64_img: &str) -> Result<Vec<u8>> {
        let clean_b64 = if let Some(idx) = base64_img.find("base64,") {
            &base64_img[idx + 7..]
        } else {
            base64_img
        };

        general_purpose::STANDARD
            .decode(clean_b64)
            .map_err(|e| AmbiError::EngineError(format!("Failed to decode base64 image: {}", e)))
    }
}

impl Drop for VisionContext {
    fn drop(&mut self) {
        match self {
            Self::ExternalProjector { _clip_ctx_ptr } => {}
            Self::Integrated => {}
        }
    }
}
