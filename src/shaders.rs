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
        "Moon" => moon_shader(fragment, uniforms, sun_position),
        _ => default_shader(fragment, uniforms, sun_position),
    }
}

pub fn star_fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Slower pulsing effect with increased base brightness
    let time_factor = 0.95 + 0.15 * ((uniforms.time as f32 * 0.02).sin());
    
    // Add noise for surface detail
    let zoom = 30.0;
    let surface_noise = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom + uniforms.time as f32 * 0.01,
        fragment.tex_coords.y * zoom
    );
    
    // Add plasma-like effect
    let plasma_noise = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom * 0.5 - uniforms.time as f32 * 0.015,
        fragment.tex_coords.y * zoom * 0.5 + uniforms.time as f32 * 0.015
    );

    // Increased base values for more brightness
    let red = 1.2;  // Increased from 1.0
    let green = 0.9 + 0.2 * surface_noise;  // Increased base from 0.84
    let blue = 0.3 + 0.15 * plasma_noise;   // Increased both base and variation

    let gradient = (1.0 - fragment.world_position.magnitude() * 0.04).max(0.0);  // Reduced falloff
    let noise_factor = 0.9 + 0.2 * (surface_noise + plasma_noise);  // Increased variation
    
    Color::from_float(
        (red * gradient * time_factor * noise_factor).min(1.0),
        (green * gradient * time_factor * noise_factor).min(1.0),
        (blue * gradient * time_factor * noise_factor).min(1.0),
    )
}

pub fn mercury_shader(fragment: &Fragment, uniforms: &Uniforms, sun_position: Vec3) -> Color {
    let zoom = 60.0;
    let time = uniforms.time as f32 * 0.02;
    
    // Create swirling patterns for the purple surface
    let surface_pattern = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom + time,
        fragment.tex_coords.y * zoom - time * 0.5
    );
    
    // Create crystalline/mineral effects
    let crystal_pattern = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom * 2.0 - time * 0.8,
        fragment.tex_coords.y * zoom * 2.0 + time * 0.3
    );

    // Deep purple surface with crystalline variations
    let surface = Color::from_float(
        0.5 + 0.2 * crystal_pattern,  // Purple-red component
        0.2 + 0.1 * surface_pattern,  // Minimal green for depth
        0.8 + 0.2 * crystal_pattern   // Strong blue for purple tint
    );

    apply_enhanced_lighting(fragment, uniforms, sun_position, surface, 1.4)
}

pub fn venus_shader(fragment: &Fragment, uniforms: &Uniforms, sun_position: Vec3) -> Color {
    let zoom = 70.0;
    let time = uniforms.time as f32 * 0.015;
    
    // Create dense sulfuric cloud patterns
    let cloud_pattern = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom + time,
        fragment.tex_coords.y * zoom - time * 0.7
    );
    
    // Create turbulent atmospheric flows
    let turbulence = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom * 1.5 - time * 0.5,
        fragment.tex_coords.y * zoom * 1.5 + time * 0.3
    );
    
    // Add heat distortion effect
    let heat_pattern = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom * 2.0 + time * 0.2,
        fragment.tex_coords.y * zoom * 2.0 - time * 0.2
    );

    // Golden-orange sulfuric atmosphere with variations
    let atmosphere = Color::from_float(
        0.85 + 0.15 * turbulence,     // Strong golden-red
        0.65 + 0.15 * cloud_pattern,  // Medium orange
        0.2 + 0.1 * heat_pattern     // Slight yellow tint
    );

    // Use lower intensity for more saturated colors
    apply_enhanced_lighting(fragment, uniforms, sun_position, atmosphere, 1.3)
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

    let water_color = Color::from_float(0.1, 0.5, 1.0);  // Made water more vibrant
    let land_color = Color::from_float(0.2, 0.9, 0.2);   // Made land more vibrant
    let mountain_color = Color::from_float(0.8, 0.6, 0.5); // Brighter mountains

    let base_color = if noise_value > 0.6 {
        mountain_color * (1.0 + mountain_noise * 0.5)
    } else if noise_value > 0.2 {
        land_color
    } else {
        water_color
    };

    // Enhanced cloud and atmosphere effects
    let cloud_zoom = 30.0;
    let cloud_noise = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * cloud_zoom + uniforms.time as f32 * 0.01,
        fragment.tex_coords.y * cloud_zoom + uniforms.time as f32 * 0.01,
    );
    let cloud_alpha = (cloud_noise * 0.5 + 0.5).clamp(0.0, 1.0);
    let cloud_color = Color::from_float(1.2, 1.2, 1.2);

    let atmosphere_factor = (1.0 - fragment.normal.dot(&Vec3::new(0.0, 1.0, 0.0))).powi(2);
    let atmosphere_color = Color::from_float(0.6, 0.8, 1.2);
    let final_color =
        base_color * (1.0 - atmosphere_factor) + atmosphere_color * atmosphere_factor * 0.4;

    let mixed_color = final_color * (1.0 - cloud_alpha) + cloud_color * cloud_alpha;

    apply_enhanced_lighting(fragment, uniforms, sun_position, mixed_color, 1.8)
}

pub fn mars_shader(fragment: &Fragment, uniforms: &Uniforms, sun_position: Vec3) -> Color {
    let zoom = 50.0;
    let time = uniforms.time as f32 * 0.02;
    
    // Create base rocky terrain
    let rock_pattern = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom * 2.0,
        fragment.tex_coords.y * zoom * 2.0
    ).abs();
    
    // Add larger rock formations
    let large_rocks = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom,
        fragment.tex_coords.y * zoom
    ).abs();
    
    // Create canyons and valleys
    let canyons = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom * 3.0,
        fragment.tex_coords.y * zoom * 3.0
    ).abs();
    
    // Dynamic dust storms with time variation
    let dust_storm = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom + time,
        fragment.tex_coords.y * zoom - time
    );

    // Combine different terrain features
    let terrain = (rock_pattern * 0.4 + large_rocks * 0.4 + canyons * 0.2)
        .max(0.0)
        .min(1.0);

    // Create color variations for different terrain features
    let base_red = 0.8 + 0.2 * terrain;  // Brighter red for highlands
    let base_brown = 0.3 + 0.2 * large_rocks; // Brown variations for rocks
    let dust_color = 0.1 + 0.1 * dust_storm;  // Subtle dust effect

    let base_color = Color::from_float(
        base_red,     // Strong red base
        base_brown,   // Brown/orange mix
        dust_color    // Dust influence
    );

    // Apply lighting with enhanced shadows for rocky appearance
    apply_enhanced_lighting(fragment, uniforms, sun_position, base_color, 1.4)
}

pub fn jupiter_shader(fragment: &Fragment, uniforms: &Uniforms, sun_position: Vec3) -> Color {
    let time = uniforms.time as f32 * 0.02;
    let latitude = fragment.tex_coords.y * PI;
    
    // Enhanced band patterns
    let band_pattern = (latitude * 12.0).sin() * 0.5 + 0.5;
    let secondary_bands = (latitude * 20.0).sin() * 0.3;
    
    // Dynamic storm patterns
    let storm = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * 30.0 + time,
        fragment.tex_coords.y * 30.0
    );

    let base_color = Color::from_float(
        0.9 + 0.3 * band_pattern,           // Warm orange-brown
        0.7 + 0.2 * band_pattern + secondary_bands,  // Varied yellows
        0.4 + 0.4 * storm                   // Storm highlights
    );

    apply_enhanced_lighting(fragment, uniforms, sun_position, base_color, 1.7)
}

pub fn saturn_shader(fragment: &Fragment, uniforms: &Uniforms, sun_position: Vec3) -> Color {
    let time = uniforms.time as f32 * 0.01;
    let zoom = 60.0;
    
    // Calculate ring parameters based on world position
    let ring_y = fragment.world_position.y.abs();  // Use absolute Y for ring detection
    let ring_distance = (fragment.world_position.x.powi(2) + fragment.world_position.z.powi(2)).sqrt();
    
    // Ring parameters
    let inner_radius = 1.2;
    let outer_radius = 2.5;
    let ring_thickness = 0.15;  // Maximum thickness of rings
    
    // Determine if we're in the ring region
    let is_ring = if ring_distance > inner_radius && ring_distance < outer_radius && ring_y < ring_thickness {
        // Calculate ring intensity based on distance from center and y-position
        let distance_factor = 1.0 - ((ring_distance - inner_radius) / (outer_radius - inner_radius));
        let height_factor = 1.0 - (ring_y / ring_thickness);
        (distance_factor * height_factor).max(0.0)
    } else {
        0.0
    };

    // Create base planet color
    let base_noise = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom,
        fragment.tex_coords.y * zoom
    );
    
    let surface_color = Color::from_float(
        0.9 + 0.1 * base_noise,    // Golden tone
        0.7 + 0.2 * base_noise,    // Warm yellow
        0.5 + 0.1 * base_noise     // Less blue
    );

    // Create ring bands pattern
    let ring_pattern = (ring_distance * 8.0).sin() * 0.5 + 0.5;  // Create circular bands
    let ring_noise = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * 120.0 + time,
        fragment.tex_coords.y * 120.0
    ) * 0.3;

    // Enhanced ring color with more contrast and variation
    let ring_color = Color::from_float(
        1.0 * (0.8 + 0.2 * ring_pattern + ring_noise),  // Brighter base
        0.95 * (0.7 + 0.3 * ring_pattern + ring_noise), // Slight golden tint
        0.9 * (0.6 + 0.4 * ring_pattern + ring_noise)   // Warmer tone
    );

    // Mix planet and ring colors with enhanced contrast
    let final_color = if is_ring > 0.0 {
        let ring_intensity = is_ring * (0.8 + 0.2 * ring_pattern);  // Vary ring intensity
        Color::from_float(
            ring_color.r as f32 / 255.0 * ring_intensity,
            ring_color.g as f32 / 255.0 * ring_intensity,
            ring_color.b as f32 / 255.0 * ring_intensity
        )
    } else {
        surface_color
    };

    apply_enhanced_lighting(fragment, uniforms, sun_position, final_color, 2.0)
}

pub fn moon_shader(fragment: &Fragment, uniforms: &Uniforms, sun_position: Vec3) -> Color {
    let zoom = 40.0;
    
    // Create large crater effects
    let large_craters = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom,
        fragment.tex_coords.y * zoom,
    ).abs();
    
    // Create smaller, more numerous craters
    let small_craters = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom * 4.0,
        fragment.tex_coords.y * zoom * 4.0,
    ).abs();
    
    // Create surface texture variations
    let surface_texture = uniforms.noise.get_noise_2d(
        fragment.tex_coords.x * zoom * 2.0,
        fragment.tex_coords.y * zoom * 2.0,
    );

    // Combine crater effects
    let crater_depth = (large_craters * 0.7 + small_craters * 0.3)
        .max(0.0)
        .min(1.0);

    // Create mare (dark areas) effect
    let mare_effect = surface_texture.abs() * 0.3;

    // Base colors for light and dark areas
    let light_color = Color::from_float(0.8, 0.8, 0.85);  // Slightly bluish white
    let dark_color = Color::from_float(0.3, 0.3, 0.35);   // Dark gray
    let mare_color = Color::from_float(0.2, 0.2, 0.25);   // Darker gray for maria

    // Mix colors based on crater depth and mare
    let mixed_color = if mare_effect > 0.2 {
        mare_color
    } else {
        let crater_factor = 1.0 - crater_depth * 0.5;
        Color::from_float(
            light_color.r as f32 / 255.0 * crater_factor,
            light_color.g as f32 / 255.0 * crater_factor,
            light_color.b as f32 / 255.0 * crater_factor
        )
    };

    // Apply enhanced lighting with reduced intensity for more contrast
    apply_enhanced_lighting(fragment, uniforms, sun_position, mixed_color, 1.2)
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

fn apply_enhanced_lighting(
    fragment: &Fragment,
    uniforms: &Uniforms,
    sun_position: Vec3,
    base_color: Color,
    intensity_multiplier: f32,
) -> Color {
    let light_dir = (sun_position - fragment.world_position).normalize();
    let diffuse = fragment.normal.dot(&light_dir).max(0.0);
    let diffuse_intensity = 2.0 * diffuse * intensity_multiplier;

    let view_dir = (-fragment.world_position).normalize();
    let reflect_dir = (2.0 * fragment.normal.dot(&light_dir) * fragment.normal - light_dir).normalize();
    let specular = reflect_dir.dot(&view_dir).max(0.0).powi(8);  // Reduced power for broader highlights
    let specular_intensity = 0.5 * specular * intensity_multiplier;

    let distance_to_sun = (sun_position - fragment.world_position).magnitude();
    let attenuation = 1.0 / (1.0 + 0.003 * distance_to_sun * distance_to_sun);  // Reduced attenuation

    // Add ambient light to prevent completely dark areas
    let ambient = 0.2;

    let mut r = base_color.r as f32 * (ambient + diffuse_intensity * attenuation + specular_intensity);
    let mut g = base_color.g as f32 * (ambient + diffuse_intensity * attenuation + specular_intensity);
    let mut b = base_color.b as f32 * (ambient + diffuse_intensity * attenuation + specular_intensity);

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
    Moon,
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
