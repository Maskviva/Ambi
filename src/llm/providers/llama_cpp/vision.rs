// src/llm/providers/llama_cpp/vision.rs
use crate::error::{AmbiError, Result};
use base64::{engine::general_purpose, Engine as _};
use log::info;
use std::path::Path;

#[cfg(feature = "mtmd")]
use llama_cpp_2::mtmd::{MtmdBitmap, MtmdContext, MtmdContextParams};

pub(crate) enum VisionContext {
    #[cfg(feature = "mtmd")]
    ExternalProjector {
        mtmd_ctx: MtmdContext,
    },
    Integrated,
}

unsafe impl Send for VisionContext {}
unsafe impl Sync for VisionContext {}

impl VisionContext {
    pub fn init(
        mmproj_path: Option<&String>,
        integrated: bool,
        model: &llama_cpp_2::model::LlamaModel,
    ) -> Result<Option<Self>> {
        if let Some(path) = mmproj_path {
            if !Path::new(path).exists() {
                return Err(AmbiError::EngineError(format!(
                    "Vision model not found: {}",
                    path
                )));
            }
            info!("Loading External Vision Projector (mmproj) from: {}", path);

            #[cfg(feature = "mtmd")]
            {
                let mut mtmd_params = MtmdContextParams::default();
                mtmd_params.print_timings = false;

                let mtmd_ctx =
                    MtmdContext::init_from_file(path, model, &mtmd_params).map_err(|e| {
                        AmbiError::EngineError(format!("Failed to init MTMD context: {}", e))
                    })?;
                Ok(Some(Self::ExternalProjector { mtmd_ctx }))
            }
            #[cfg(not(feature = "mtmd"))]
            {
                Err(AmbiError::EngineError(
                    "External projector support requires the 'mtmd' feature.".into(),
                ))
            }
        } else if integrated {
            info!("Vision capabilities are integrated natively into the main LLM.");
            Ok(Some(Self::Integrated))
        } else {
            Ok(None)
        }
    }

    #[cfg(feature = "mtmd")]
    pub fn create_bitmaps(&self, images: &[String]) -> Result<Vec<MtmdBitmap>> {
        match self {
            Self::ExternalProjector { mtmd_ctx } => images
                .iter()
                .map(|b64| {
                    let raw = Self::decode_base64_image(b64)?;
                    MtmdBitmap::from_buffer(mtmd_ctx, &raw).map_err(|e| {
                        AmbiError::EngineError(format!("Bitmap creation failed: {}", e))
                    })
                })
                .collect(),
            _ => Err(AmbiError::EngineError(
                "create_bitmaps called on non-ExternalProjector context".into(),
            )),
        }
    }

    #[cfg(not(feature = "mtmd"))]
    pub fn create_bitmaps(&self, _images: &[String]) -> Result<Vec<()>> {
        Err(AmbiError::EngineError("MTMD feature not enabled".into()))
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
