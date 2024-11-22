use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{look_at, perspective, Mat4, Vec3};
use std::f32::consts::PI;

mod camera;
mod color;
mod fragment;
mod framebuffer;
mod normal_map;
mod obj;
mod shaders;
mod skybox;
mod texture;
mod triangle;
mod vertex;

use crate::shaders::{
    earth_shader, gas_giant_fragment_shader, jupiter_shader, mars_shader, mercury_shader,
    moon_shader, rocky_planet_fragment_shader, saturn_shader, star_fragment_shader, 
    venus_shader, vertex_shader, ShaderType,
};
use camera::Camera;
use fastnoise_lite::{FastNoiseLite, NoiseType};
use framebuffer::Framebuffer;
use obj::Obj;
use triangle::triangle;
use vertex::Vertex;

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    noise: FastNoiseLite,
}

fn create_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::Cellular));
    noise.set_frequency(Some(0.1));
    noise
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0, 0.0, 0.0, 0.0, 
        0.0, cos_x, -sin_x, 0.0, 
        0.0, sin_x, cos_x, 0.0, 
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y, 0.0, sin_y, 0.0, 
        0.0, 1.0, 0.0, 0.0, 
        -sin_y, 0.0, cos_y, 0.0, 
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0, 
        sin_z, cos_z, 0.0, 0.0, 
        0.0, 0.0, 1.0, 0.0, 
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let scale_matrix = Mat4::new(
        scale, 0.0, 0.0, 0.0, 
        0.0, scale, 0.0, 0.0, 
        0.0, 0.0, scale, 0.0, 
        0.0, 0.0, 0.0, 1.0,
    );

    let translation_matrix = Mat4::new(
        1.0,
        0.0,
        0.0,
        translation.x, 
        0.0,
        1.0,
        0.0,
        translation.y, 
        0.0,
        0.0,
        1.0,
        translation.z, 
        0.0,
        0.0,
        0.0,
        1.0,
    );

    translation_matrix * rotation_matrix * scale_matrix
}

fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 1000.0;

    perspective(fov, aspect_ratio, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0,
        0.0,
        0.0,
        width / 2.0, 
        0.0,
        -height / 2.0,
        0.0,
        height / 2.0, 
        0.0,
        0.0,
        1.0,
        0.0, 
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    shader_type: &ShaderType,
    sun_position: Vec3,
) {
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    let mut fragments = Vec::new();
    for tri in transformed_vertices.chunks(3) {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            let shaded_color = match shader_type {
                ShaderType::Star => star_fragment_shader(&fragment, uniforms),
                ShaderType::Mercury => mercury_shader(&fragment, uniforms, sun_position),
                ShaderType::Venus => venus_shader(&fragment, uniforms, sun_position),
                ShaderType::Earth => earth_shader(&fragment, uniforms, sun_position),
                ShaderType::Mars => mars_shader(&fragment, uniforms, sun_position),
                ShaderType::Jupiter => jupiter_shader(&fragment, uniforms, sun_position),
                ShaderType::Saturn => saturn_shader(&fragment, uniforms, sun_position),
                ShaderType::Moon => moon_shader(&fragment, uniforms, sun_position),
                ShaderType::RockyPlanet => {
                    rocky_planet_fragment_shader(&fragment, uniforms, sun_position)
                }
                ShaderType::GasGiant => {
                    gas_giant_fragment_shader(&fragment, uniforms, sun_position)
                }
                ShaderType::Custom(shader_fn) => shader_fn(&fragment, uniforms),
            };

            framebuffer.set_current_color(shaded_color.to_hex());
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Rust Graphics - Renderer Example",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x000010);

    
    let mut celestial_bodies = vec![
        
        CelestialBody {
            name: "Sun".to_string(),
            position: Vec3::new(0.0, 0.0, 0.0),
            scale: 2.0,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Star,
            visible: true,
        },
        
        CelestialBody {
            name: "Mercury".to_string(),
            position: Vec3::new(3.0, 1.0, -1.5),
            scale: 0.5,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Mercury,
            visible: true,
        },
        
        CelestialBody {
            name: "Venus".to_string(),
            position: Vec3::new(-4.5, -1.0, 1.0),
            scale: 0.6,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Venus,
            visible: true,
        },
        
        CelestialBody {
            name: "Earth".to_string(),
            position: Vec3::new(6.0, 0.5, -2.0),
            scale: 0.6,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Earth,
            visible: true,
        },
        
        CelestialBody {
            name: "Mars".to_string(),
            position: Vec3::new(-7.0, -0.5, 1.5),
            scale: 0.5,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Mars,
            visible: true,
        },
        
        CelestialBody {
            name: "Jupiter".to_string(),
            position: Vec3::new(9.0, 1.5, -3.0),
            scale: 1.5,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Jupiter,
            visible: true,
        },
        
        CelestialBody {
            name: "Saturn".to_string(),
            position: Vec3::new(-12.0, -1.5, 2.0),
            scale: 2.0,     // Increased scale further
            rotation: Vec3::new(0.4, 0.0, 0.0),  // More pronounced tilt
            shader_type: ShaderType::Saturn,
            visible: true,
        },
        
        CelestialBody {
            name: "Moon".to_string(),
            position: Vec3::new(6.8, 0.7, -2.2), // Slightly offset from Earth
            scale: 0.16,                         // Much smaller than Earth
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Moon,
            visible: true,
        },
    ];

    
    let obj = Obj::load("assets/models/sphere.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array();
    let mut time = 0;

    
    let noise = create_noise();
    let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
    let viewport_matrix =
        create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);
    let mut uniforms = Uniforms {
        model_matrix: Mat4::identity(),
        view_matrix: Mat4::identity(),
        projection_matrix,
        viewport_matrix,
        time: 0,
        noise,
    };

    
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 20.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        time += 1;

        handle_input(&window, &mut camera, &mut celestial_bodies, &uniforms);

        framebuffer.clear();

        
        uniforms.view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        uniforms.time = time;

        let sun_position = Vec3::new(0.0, 0.0, 0.0);

        for body in &celestial_bodies {
            if body.visible {
                uniforms.model_matrix = create_model_matrix(body.position, body.scale, body.rotation);
                render(
                    &mut framebuffer,
                    &uniforms,
                    &vertex_arrays,
                    &body.shader_type,
                    sun_position,
                );
            }
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();
    }
}

fn handle_input(window: &Window, camera: &mut Camera, celestial_bodies: &mut Vec<CelestialBody>, uniforms: &Uniforms) {
    let movement_speed = 1.0;
    let rotation_speed = PI / 50.0;
    let zoom_speed = 0.1;

    
    if window.is_key_down(Key::Left) {
        camera.orbit(rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Right) {
        camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::W) {
        camera.orbit(0.0, -rotation_speed);
    }
    if window.is_key_down(Key::S) {
        camera.orbit(0.0, rotation_speed);
    }

    
    let mut movement = Vec3::new(0.0, 0.0, 0.0);
    if window.is_key_down(Key::A) {
        movement.x -= movement_speed;
    }
    if window.is_key_down(Key::D) {
        movement.x += movement_speed;
    }
    if window.is_key_down(Key::Q) {
        movement.y += movement_speed;
    }
    if window.is_key_down(Key::E) {
        movement.y -= movement_speed;
    }
    if movement.magnitude() > 0.0 {
        camera.move_center(movement);
    }

    
    if window.is_key_down(Key::Up) {
        camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::Down) {
        camera.zoom(-zoom_speed);
    }

    if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) {
        celestial_bodies[0].visible = !celestial_bodies[0].visible;
    }
    if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) {
        celestial_bodies[1].visible = !celestial_bodies[1].visible;
    }
    if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) {
        celestial_bodies[2].visible = !celestial_bodies[2].visible;
    }
    if window.is_key_pressed(Key::Key4, minifb::KeyRepeat::No) {
        celestial_bodies[3].visible = !celestial_bodies[3].visible;
    }
    if window.is_key_pressed(Key::Key5, minifb::KeyRepeat::No) {
        celestial_bodies[4].visible = !celestial_bodies[4].visible;
    }
    if window.is_key_pressed(Key::Key6, minifb::KeyRepeat::No) {
        celestial_bodies[5].visible = !celestial_bodies[5].visible;
    }
    if window.is_key_pressed(Key::Key7, minifb::KeyRepeat::No) {
        celestial_bodies[6].visible = !celestial_bodies[6].visible;
    }
    if window.is_key_pressed(Key::Key8, minifb::KeyRepeat::No) {
        celestial_bodies[7].visible = !celestial_bodies[7].visible; // Toggle Moon visibility
    }

    // Update Moon position to orbit around Earth
    let earth_position = celestial_bodies[3].position;
    let orbit_speed = 0.02;
    let orbit_radius = 0.8;
    let moon = &mut celestial_bodies[7];
    
    moon.position = Vec3::new(
        earth_position.x + orbit_radius * (uniforms.time as f32 * orbit_speed).cos(),
        earth_position.y + 0.2 * (uniforms.time as f32 * orbit_speed * 0.5).sin(),
        earth_position.z + orbit_radius * (uniforms.time as f32 * orbit_speed).sin()
    );
}

struct CelestialBody {
    name: String,
    position: Vec3,
    scale: f32,
    rotation: Vec3,
    shader_type: ShaderType,
    visible: bool,
}
