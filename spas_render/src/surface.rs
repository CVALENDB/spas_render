use glam::UVec2;
use spas_render_types::{TextureFormat, TextureUsages};
use wgpu::{CompositeAlphaMode, Device, SurfaceColorSpace, SurfaceConfiguration};

use crate::types::{PresentMode, Surface};

/// Convinence function that re-configures the surface with the expected usages.
pub fn configure_surface(
    surface: &Surface,
    device: &Device,
    format: TextureFormat,
    size: UVec2,
    present_mode: PresentMode,
) {
    surface.configure(
        device,
        &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: SurfaceColorSpace::Auto,
            width: size.x,
            height: size.y,
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        },
    )
}
