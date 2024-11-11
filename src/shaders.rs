use crate::color::Color;
use crate::fragment::Fragment;
use crate::vertex::Vertex;
use crate::Uniforms;
use fastnoise_lite::{FastNoiseLite, NoiseType};
use nalgebra_glm::{mat4_to_mat3, Mat3, Vec3, Vec4};

fn create_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::Perlin));
    noise.set_frequency(Some(0.05));
    noise
}

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::new(vertex.position.x, vertex.position.y, vertex.position.z, 1.0);

    let world_position = uniforms.model_matrix * position;

    let clip_position = uniforms.projection_matrix * uniforms.view_matrix * world_position;

    let w = clip_position.w;
    let ndc_position = Vec4::new(
        clip_position.x / w,
        clip_position.y / w,
        clip_position.z / w,
        1.0,
    );

    let screen_position = uniforms.viewport_matrix * ndc_position;

    let model_mat3 = mat4_to_mat3(&uniforms.model_matrix);
    let normal_matrix = model_mat3
        .transpose()
        .try_inverse()
        .unwrap_or(Mat3::identity());
    let transformed_normal = normal_matrix * vertex.normal;

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal,
        world_position: Vec3::new(world_position.x, world_position.y, world_position.z),
    }
}

pub fn planet_fragment_shader(
    fragment: &Fragment,
    uniforms: &Uniforms,
    planet_type: &str,
    sun_position: Vec3,
) -> Color {
    match planet_type {
        "Sun" => star_fragment_shader(fragment, uniforms),
        "Mercury" => mercury_shader(fragment, uniforms, sun_position),
        "Venus" => venus_shader(fragment, uniforms, sun_position),
        "Earth" => earth_shader(fragment, uniforms, sun_position),
        "Mars" => mars_shader(fragment, uniforms, sun_position),
        "Jupiter" => jupiter_shader(fragment, uniforms, sun_position),
        "Saturn" => saturn_shader(fragment, uniforms, sun_position),
        _ => default_shader(fragment, uniforms, sun_position),
    }
}

pub fn star_fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let time_factor = 0.8 + 0.2 * ((uniforms.time as f32 * 0.05).sin());

    let red = 1.0;
    let green = 0.84;
    let blue = 0.2;

    let gradient = (1.0 - fragment.world_position.magnitude() * 0.05).max(0.0);
    Color::from_float(
        red * gradient * time_factor,
        green * gradient * time_factor,
        blue * gradient * time_factor,
    )
}

pub fn mercury_shader(fragment: &Fragment, uniforms: &Uniforms, sun_position: Vec3) -> Color {
    let zoom = 40.0;
    let noise1 = uniforms
        .noise
        .get_noise_2d(fragment.tex_coords.x * zoom, fragment.tex_coords.y * zoom);
    let noise2 = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom * 2.0,
        fragment.tex_coords.y * zoom * 2.0,
    );

    let combined_noise = (noise1 + noise2 * 0.5).max(0.0);
    let base_color = Color::from_float(0.5 + 0.4 * combined_noise, 0.5 + 0.3 * combined_noise, 0.4);

    apply_lighting(fragment, uniforms, sun_position, base_color)
}

pub fn venus_shader(fragment: &Fragment, uniforms: &Uniforms, sun_position: Vec3) -> Color {
    let zoom = 100.0;
    let noise1 = uniforms
        .noise
        .get_noise_2d(fragment.tex_coords.x * zoom, fragment.tex_coords.y * zoom);
    let turbulence = (uniforms.noise.get_noise_2d(
        fragment.world_position.x * 10.0,
        uniforms.time as f32 * 0.05,
    ))
    .abs();

    let base_color = Color::from_float(1.0, 0.7 + 0.2 * noise1, 0.2 + 0.1 * turbulence);

    apply_lighting(fragment, uniforms, sun_position, base_color)
}

pub fn earth_shader(fragment: &Fragment, uniforms: &Uniforms, sun_position: Vec3) -> Color {
    let zoom = 80.0;
    let noise_value = uniforms
        .noise
        .get_noise_2d(fragment.tex_coords.x * zoom, fragment.tex_coords.y * zoom);
    let mountain_noise = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom * 2.0,
        fragment.tex_coords.y * zoom * 2.0,
    );

    let water_color = Color::from_float(0.0, 0.3, 0.7);
    let land_color = Color::from_float(0.2, 0.6, 0.2);
    let mountain_color = Color::from_float(0.5, 0.4, 0.3);

    let base_color = if noise_value > 0.6 {
        mountain_color * (1.0 + mountain_noise * 0.5)
    } else if noise_value > 0.2 {
        land_color
    } else {
        water_color
    };

    let cloud_zoom = 30.0;
    let cloud_noise = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * cloud_zoom + uniforms.time as f32 * 0.01,
        fragment.tex_coords.y * cloud_zoom + uniforms.time as f32 * 0.01,
    );
    let cloud_alpha = (cloud_noise * 0.5 + 0.5).clamp(0.0, 1.0);
    let cloud_color = Color::from_float(1.0, 1.0, 1.0);

    let atmosphere_factor = (1.0 - fragment.normal.dot(&Vec3::new(0.0, 1.0, 0.0))).powi(2);
    let atmosphere_color = Color::from_float(0.6, 0.8, 1.0);
    let final_color =
        base_color * (1.0 - atmosphere_factor) + atmosphere_color * atmosphere_factor * 0.3;

    let mixed_color = final_color * (1.0 - cloud_alpha) + cloud_color * cloud_alpha;

    apply_lighting(fragment, uniforms, sun_position, mixed_color)
}

pub fn mars_shader(fragment: &Fragment, uniforms: &Uniforms, sun_position: Vec3) -> Color {
    let zoom = 50.0;
    let noise_value = uniforms
        .noise
        .get_noise_2d(fragment.tex_coords.x * zoom, fragment.tex_coords.y * zoom);
    let dust_noise = uniforms.noise.get_noise_2d(
        fragment.world_position.x * 5.0,
        fragment.world_position.z * 5.0,
    );

    let base_color = Color::from_float(0.8 + 0.2 * dust_noise, 0.4 * noise_value, 0.3);

    apply_lighting(fragment, uniforms, sun_position, base_color)
}

pub fn jupiter_shader(fragment: &Fragment, uniforms: &Uniforms, sun_position: Vec3) -> Color {
    let latitude = fragment.tex_coords.y * PI;
    let band_pattern = (latitude * 10.0).sin() * 0.5 + 0.5;
    let noise_value = uniforms
        .noise
        .get_noise_2d(fragment.tex_coords.x * 20.0, uniforms.time as f32 * 0.05);
    let storm_noise = (uniforms
        .noise
        .get_noise_2d(fragment.world_position.x * 5.0, uniforms.time as f32 * 0.1))
    .abs();

    let base_color = Color::from_float(
        0.9 * band_pattern,
        0.7 * band_pattern,
        0.4 * noise_value + 0.2 * storm_noise,
    );

    apply_lighting(fragment, uniforms, sun_position, base_color)
}

pub fn saturn_shader(fragment: &Fragment, uniforms: &Uniforms, sun_position: Vec3) -> Color {
    let latitude = fragment.tex_coords.y * PI;
    let band_pattern = (latitude * 15.0).sin() * 0.5 + 0.5;

    let ring_distance =
        (fragment.world_position.x.powi(2) + fragment.world_position.z.powi(2)).sqrt();

    let ring_effect = (ring_distance - 1.8).abs();

    let is_ring = if ring_effect < 0.1 { 1.0 } else { 0.0 };

    let planet_color = Color::from_float(0.8 * band_pattern, 0.7 * band_pattern, 0.5);

    let ring_color = Color::from_float(0.9, 0.8, 0.6);

    let final_color = if is_ring > 0.0 {
        ring_color
    } else {
        planet_color
    };

    apply_lighting(fragment, uniforms, sun_position, final_color)
}

pub fn default_shader(fragment: &Fragment, uniforms: &Uniforms, sun_position: Vec3) -> Color {
    let base_color = Color::new(100, 100, 100);
    apply_lighting(fragment, uniforms, sun_position, base_color)
}

fn apply_lighting(
    fragment: &Fragment,
    uniforms: &Uniforms,
    sun_position: Vec3,
    base_color: Color,
) -> Color {
    let light_dir = (sun_position - fragment.world_position).normalize();
    let diffuse = fragment.normal.dot(&light_dir).max(0.0);
    let diffuse_intensity = 1.5 * diffuse;

    let view_dir = (-fragment.world_position).normalize();
    let reflect_dir =
        (2.0 * fragment.normal.dot(&light_dir) * fragment.normal - light_dir).normalize();
    let specular = reflect_dir.dot(&view_dir).max(0.0).powi(16);
    let specular_intensity = 0.3 * specular;

    let distance_to_sun = (sun_position - fragment.world_position).magnitude();
    let attenuation = 1.0 / (1.0 + 0.005 * distance_to_sun * distance_to_sun);

    let mut r = base_color.r as f32 * (diffuse_intensity * attenuation + specular_intensity);
    let mut g = base_color.g as f32 * (diffuse_intensity * attenuation + specular_intensity);
    let mut b = base_color.b as f32 * (diffuse_intensity * attenuation + specular_intensity);

    r = r.clamp(0.0, 255.0);
    g = g.clamp(0.0, 255.0);
    b = b.clamp(0.0, 255.0);

    Color::new(r as u8, g as u8, b as u8)
}

use std::f32::consts::PI;

pub enum ShaderType {
    Star,
    Mercury,
    Venus,
    Earth,
    Mars,
    Jupiter,
    Saturn,
    RockyPlanet,
    GasGiant,
    Custom(fn(&Fragment, &Uniforms) -> Color),
}

pub fn rocky_planet_fragment_shader(
    fragment: &Fragment,
    uniforms: &Uniforms,
    sun_position: Vec3,
) -> Color {
    let diffuse = fragment
        .normal
        .dot(&(sun_position - fragment.world_position).normalize())
        .max(0.0);
    let base_color = Color::new(200, 100, 50);
    base_color * diffuse
}

pub fn gas_giant_fragment_shader(
    fragment: &Fragment,
    uniforms: &Uniforms,
    sun_position: Vec3,
) -> Color {
    let band_noise = uniforms
        .noise
        .get_noise_2d(fragment.tex_coords.x * 5.0, uniforms.time as f32 * 0.1);
    let base_color = Color::from_float(0.8, 0.5, 1.0);
    base_color * (0.5 + 0.5 * band_noise)
}
