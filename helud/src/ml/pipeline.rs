use ort::session::Session;
use image::DynamicImage;
use ndarray::Array1;

use ort::value::Tensor;
use ndarray::Array4;
use image::imageops::FilterType;

pub fn frame_to_embedding(
    session: &mut Session,
    frame: &DynamicImage,
) -> anyhow::Result<Array1<f32>> {
    // MobileFaceNet input: 112x112 RGB
    let resized = frame.resize_exact(112, 112, FilterType::Triangle).to_rgb8();

    // Convert to normalized tensor [-1, 1]
    let mut tensor = Array4::<f32>::zeros((1, 3, 112, 112));
    for (x, y, pixel) in resized.enumerate_pixels() {
        tensor[[0, 0, y as usize, x as usize]] = (pixel[0] as f32 - 127.5) / 128.0;
        tensor[[0, 1, y as usize, x as usize]] = (pixel[1] as f32 - 127.5) / 128.0;
        tensor[[0, 2, y as usize, x as usize]] = (pixel[2] as f32 - 127.5) / 128.0;
    }

    // Run inference
    let inputs = ort::inputs![Tensor::from_array(tensor.into_dyn())?];
    let outputs = session.run(inputs)?;

    // Extract output
    let output_tensor = outputs[0].try_extract_tensor::<f32>()?;
    let slice = output_tensor.1;
    let mut embedding = Array1::from_vec(slice.to_vec());

    // L2 normalize
    let l2_norm = embedding.dot(&embedding).sqrt();
    if l2_norm > 0.0 {
        embedding /= l2_norm;
    }

    Ok(embedding)
}

pub fn cosine_similarity(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
    a.dot(b)
}

pub fn is_match(embedding: &Array1<f32>, stored: &[Array1<f32>], threshold: f32) -> bool {
    stored.iter().any(|e| cosine_similarity(embedding, e) >= threshold)
}
