use crate::{Framebuffer, Uniforms};
use nalgebra_glm::{Vec3, Vec4};
use image::open;

pub struct Skybox {
    texture: image::RgbaImage,
    width: u32,
    height: u32,
}

impl Skybox {
    pub fn new() -> Self {
        let img = open("assets/image/front.png").expect("Failed to load skybox texture")
            .to_rgba8();
        let width = img.width();
        let height = img.height();
        
        Skybox { 
            texture: img,
            width,
            height,
        }
    }

    pub fn render(&self, framebuffer: &mut Framebuffer, uniforms: &Uniforms) {
        for y in 0..framebuffer.height {
            for x in 0..framebuffer.width {
                // Convert screen coordinates to texture coordinates
                let tex_x = ((x as f32 / framebuffer.width as f32) * self.width as f32) as u32;
                let tex_y = ((y as f32 / framebuffer.height as f32) * self.height as f32) as u32;

                if let Some(pixel) = self.texture.get_pixel_checked(tex_x, tex_y) {
                    let r = pixel[0] as u32;
                    let g = pixel[1] as u32;
                    let b = pixel[2] as u32;
                    let color = (r << 16) | (g << 8) | b;
                    
                    framebuffer.set_current_color(color);
                    framebuffer.point(x, y, 1000.0); // Render behind everything else
                }
            }
        }
    }
}
