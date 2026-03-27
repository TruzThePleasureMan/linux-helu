use std::path::Path;
use anyhow::Result;
use ort::session::{Session, builder::GraphOptimizationLevel};
use ort::execution_providers::{CUDAExecutionProvider, TensorRTExecutionProvider, CPUExecutionProvider};

pub fn load_session(model_path: &Path) -> Result<Session> {
    let builder = Session::builder()
        .map_err(|e| anyhow::anyhow!("Failed to create Session builder: {}", e))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| anyhow::anyhow!("Failed to set optimization level: {}", e))?;

    // Try CUDA first, then TensorRT, then CPU
    let session = builder
        .with_execution_providers([
            CUDAExecutionProvider::default().build(),
            TensorRTExecutionProvider::default().build(),
            CPUExecutionProvider::default().build(),
        ])
        .map_err(|e| anyhow::anyhow!("Failed to build session with providers: {}", e))?
        .commit_from_file(model_path)
        .map_err(|e| anyhow::anyhow!("Failed to commit session from file: {}", e))?;

    Ok(session)
}
