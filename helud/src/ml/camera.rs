use image::{DynamicImage, RgbImage};
use nokhwa::Camera as NokhwaCamera;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::pixel_format::RgbFormat;

pub struct Camera {
    nokhwa: Option<NokhwaCamera>,
    mock: bool,
}

impl Camera {
    pub fn open(index: u32, mock: bool) -> anyhow::Result<Self> {
        if mock {
            return Ok(Self {
                nokhwa: None,
                mock: true,
            });
        }

        let index = CameraIndex::Index(index);
        let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        let mut nokhwa_cam = NokhwaCamera::new(index, format)?;
        nokhwa_cam.open_stream()?;

        Ok(Self {
            nokhwa: Some(nokhwa_cam),
            mock: false,
        })
    }

    pub fn capture_frame(&mut self) -> anyhow::Result<DynamicImage> {
        if self.mock {
            // Return a dummy 112x112 image
            let img = RgbImage::new(112, 112);
            return Ok(DynamicImage::ImageRgb8(img));
        }

        if let Some(cam) = &mut self.nokhwa {
            let frame = cam.frame()?;
            let decoded = frame.decode_image::<RgbFormat>()?;
            Ok(DynamicImage::ImageRgb8(decoded))
        } else {
            anyhow::bail!("Camera not initialized")
        }
    }
}

#[allow(dead_code)]
pub fn camera_available(index: u32) -> bool {
    NokhwaCamera::new(
        CameraIndex::Index(index),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
    ).is_ok()
}
