use crate::{Framebuffer, Uniforms};
use nalgebra_glm::{Vec3, Vec4};
use rand::prelude::*;
use std::f32::consts::PI;

pub struct Star {
    position: Vec3,
    brightness: f32,
    size: u8,
}

pub struct Skybox {
    stars: Vec<Star>,
}

impl Skybox {
    pub fn new(star_count: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut stars = Vec::with_capacity(star_count);

        for _ in 0..star_count {
            let theta = rng.gen::<f32>() * 2.0 * PI;
            let phi = rng.gen::<f32>() * PI;
            let radius = 100.0;

            let x = radius * phi.sin() * theta.cos();
            let y = radius * phi.cos();
            let z = radius * phi.sin() * theta.sin();

            let brightness = rng.gen::<f32>();
            let size: u8 = rng.gen_range(1..=3);

            stars.push(Star {
                position: Vec3::new(x, y, z),
                brightness,
                size,
            });
        }

        Skybox { stars }
    }

    pub fn render(
        &self,
        framebuffer: &mut Framebuffer,
        uniforms: &Uniforms,
        camera_position: Vec3,
    ) {
        for star in &self.stars {
            let position = star.position + camera_position;

            let pos_vec4 = Vec4::new(position.x, position.y, position.z, 1.0);
            let projected = uniforms.projection_matrix * uniforms.view_matrix * pos_vec4;

            if projected.w <= 0.0 {
                continue;
            }
            let ndc = projected / projected.w;

            let screen_pos = uniforms.viewport_matrix * Vec4::new(ndc.x, ndc.y, ndc.z, 1.0);

            if screen_pos.z < 0.0 {
                continue;
            }

            let x = screen_pos.x as usize;
            let y = screen_pos.y as usize;

            if x < framebuffer.width && y < framebuffer.height {
                let intensity = (star.brightness * 255.0) as u8;
                let color = (intensity as u32) << 16 | (intensity as u32) << 8 | intensity as u32;

                framebuffer.set_current_color(color);

                match star.size {
                    1 => framebuffer.point(x, y, 1000.0),
                    2 => {
                        framebuffer.point(x, y, 1000.0);
                        framebuffer.point(x + 1, y, 1000.0);
                        framebuffer.point(x, y + 1, 1000.0);
                        framebuffer.point(x + 1, y + 1, 1000.0);
                    }
                    3 => {
                        framebuffer.point(x, y, 1000.0);
                        framebuffer.point(x - 1, y, 1000.0);
                        framebuffer.point(x + 1, y, 1000.0);
                        framebuffer.point(x, y - 1, 1000.0);
                        framebuffer.point(x, y + 1, 1000.0);
                    }
                    _ => {}
                }
            }
        }
    }
}
