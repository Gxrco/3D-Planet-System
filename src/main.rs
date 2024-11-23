use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{look_at, perspective, Mat4, Vec3, Vec4, make_vec4};
use std::f32::consts::PI;
use image::{codecs::gif::GifDecoder, AnimationDecoder};

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
// Add WarpState to the camera imports
use camera::{Camera, WarpState};
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

struct Ship {
    position: Vec3,
    rotation: Vec3,
    scale: f32,
    obj: Obj,
    vertex_arrays: Vec<Vertex>,
    initial_rotation: Vec3,  // Add this new field
}

impl Ship {
    fn new() -> Self {
        let obj = Obj::load("assets/models/model.obj").expect("Failed to load ship model");
        let vertex_arrays = obj.get_vertex_array();
        let initial_rotation = Vec3::new(0.0, PI, 0.0);  // Store initial rotation
        
        Ship {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: initial_rotation,
            scale: 0.1, // Decrease the scale to make the ship smaller
            obj,
            vertex_arrays,
            initial_rotation,  // Add this field
        }
    }

    fn update_position(&mut self, camera: &Camera) {
        // Calculate camera direction vectors
        let forward = (camera.center - camera.eye).normalize();
        let up = camera.up;
        
        // Position ship in front of camera
        self.position = camera.eye +
            (forward * 4.0) +    // Decrease distance in front
            (up * -0.8);         // Slightly below

        // Calculate the angle between camera position and center
        let delta = camera.eye - camera.center;
        let yaw = delta.x.atan2(delta.z);
        
        // Set ship rotation to counteract camera rotation
        self.rotation.y = yaw;
        self.rotation.x = 0.0;
        self.rotation.z = 0.0;
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

    
    let obj = Obj::load("assets/models/sphere.obj").expect("Failed to load sphere obj");
    let vertex_arrays = obj.get_vertex_array();
    
    // Load ring model
    let ring_obj = Obj::load("assets/models/ring.obj").expect("Failed to load ring obj");
    let ring_vertex_arrays = ring_obj.get_vertex_array();

    let ring = Ring {
        obj: ring_obj,
        vertex_arrays: ring_vertex_arrays,
        scale: 1.0,  // Ring scale relative to planet
        rotation: Vec3::new(0.4, 0.0, 0.0),  // Match Saturn's tilt
    };

    let mut celestial_bodies = vec![
        
        CelestialBody {
            name: "Sun".to_string(),
            position: Vec3::new(0.0, 0.0, 0.0),
            scale: 2.0,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Star,
            visible: true,
            orbital_speed: 0.0,
            axial_speed: 0.001,
            orbital_radius: 0.0,
            orbital_offset: 0.0,
            ring: None,
            trail: Vec::with_capacity(50),
            orbital_angle: 0.0,
            orbit_complete: false,
        },
        
        CelestialBody {
            name: "Mercury".to_string(),
            position: Vec3::new(5.0, 0.0, 0.0),  // Changed position
            scale: 0.5,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Mercury,
            visible: true,
            orbital_speed: 0.02,
            axial_speed: 0.004,
            orbital_radius: 5.0,  // Increased radius
            orbital_offset: 0.0,
            ring: None,
            trail: Vec::with_capacity(50),
            orbital_angle: 0.0,
            orbit_complete: false,
        },
        
        CelestialBody {
            name: "Venus".to_string(),
            position: Vec3::new(-9.0, 0.0, 0.0),  // Changed position
            scale: 0.6,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Venus,
            visible: true,
            orbital_speed: 0.015,
            axial_speed: 0.002,
            orbital_radius: 9.0,  // Increased radius
            orbital_offset: 0.0,
            ring: None,
            trail: Vec::with_capacity(50),
            orbital_angle: 0.0,
            orbit_complete: false,
        },
        
        CelestialBody {
            name: "Earth".to_string(),
            position: Vec3::new(13.0, 0.0, 0.0),  // Changed position
            scale: 0.6,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Earth,
            visible: true,
            orbital_speed: 0.01,
            axial_speed: 0.003,
            orbital_radius: 13.0,  // Increased radius
            orbital_offset: 0.0,
            ring: None,
            trail: Vec::with_capacity(50),
            orbital_angle: 0.0,
            orbit_complete: false,
        },
        
        CelestialBody {
            name: "Mars".to_string(),
            position: Vec3::new(-17.0, 0.0, 0.0),  // Changed position
            scale: 0.5,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Mars,
            visible: true,
            orbital_speed: 0.008,
            axial_speed: 0.003,
            orbital_radius: 17.0,  // Increased radius
            orbital_offset: 0.0,
            ring: None,
            trail: Vec::with_capacity(50),
            orbital_angle: 0.0,
            orbit_complete: false,
        },
        
        CelestialBody {
            name: "Jupiter".to_string(),
            position: Vec3::new(22.0, 0.0, 0.0),  // Changed position
            scale: 1.5,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Jupiter,
            visible: true,
            orbital_speed: 0.004,
            axial_speed: 0.004,
            orbital_radius: 22.0,  // Increased radius
            orbital_offset: 0.0,
            ring: None,
            trail: Vec::with_capacity(50),
            orbital_angle: 0.0,
            orbit_complete: false,
        },
        
        CelestialBody {
            name: "Saturn".to_string(),
            position: Vec3::new(-28.0, 0.0, 0.0),  // Changed position
            scale: 2.0,     // Increased scale further
            rotation: Vec3::new(0.4, 0.0, 0.0),  // More pronounced tilt
            shader_type: ShaderType::Saturn,
            visible: true,
            orbital_speed: 0.003,
            axial_speed: 0.003,
            orbital_radius: 28.0,  // Increased radius
            orbital_offset: 0.0,
            ring: Some(ring),  // Add the ring to Saturn
            trail: Vec::with_capacity(50),
            orbital_angle: 0.0,
            orbit_complete: false,
        },
        
        CelestialBody {
            name: "Moon".to_string(),
            position: Vec3::new(13.8, 0.0, 0.0),  // Near Earth, same altitude
            scale: 0.16,                         // Much smaller than Earth
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Moon,
            visible: true,
            orbital_speed: 0.0,
            axial_speed: 0.0,
            orbital_radius: 0.0,
            orbital_offset: 0.0,
            ring: None,
            trail: Vec::with_capacity(50),
            orbital_angle: 0.0,
            orbit_complete: false,
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
        Vec3::new(0.0, 25.0, 40.0),  // Higher and further back
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    // Load and decode portal animation frames
    let file = std::fs::File::open("assets/image/portal.gif").expect("Failed to open portal.gif");
    let decoder = GifDecoder::new(file).expect("Failed to create GIF decoder");
    let frames = decoder.into_frames().collect_frames().expect("Failed to collect frames");
    let mut current_frame = 0;

    let skybox = skybox::Skybox::new();

    let mut ship = Ship::new();
    let mut ship_mode = false;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        time += 1;

        // Toggle ship mode
        if window.is_key_pressed(Key::Tab, minifb::KeyRepeat::No) {
            ship_mode = !ship_mode;
        }

        // Use the same camera controls regardless of ship mode
        handle_input(&window, &mut camera, &mut celestial_bodies, &uniforms);

        // Update ship position if in ship mode
        if ship_mode {
            ship.update_position(&camera);
        }

        framebuffer.clear();

        // Render skybox first
        skybox.render(&mut framebuffer, &uniforms);

        // Then render everything else
        uniforms.view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        uniforms.time = time;

        let sun_position = Vec3::new(0.0, 0.0, 0.0);

        for body in &celestial_bodies {
            if body.visible {
                // Render trails first
                for trail_point in &body.trail {
                    let trail_color = match body.shader_type {
                        ShaderType::Mercury => 0x666666,
                        ShaderType::Venus => 0xFFBB22,
                        ShaderType::Earth => 0x2266FF,
                        ShaderType::Mars => 0xFF6622,
                        ShaderType::Jupiter => 0xFFBB66,
                        ShaderType::Saturn => 0xFFDD66,
                        _ => 0x555555,
                    };
                    
                    let pos = trail_point.position;
                    let pos_vec4 = make_vec4(&[pos.x, pos.y, pos.z, 1.0]);
                    let view_pos = uniforms.view_matrix * pos_vec4;
                    let proj_pos = uniforms.projection_matrix * view_pos;
                    
                    if proj_pos[3] != 0.0 {
                        // Perspective division
                        let ndc_x = proj_pos[0] / proj_pos[3];
                        let ndc_y = proj_pos[1] / proj_pos[3];
                        
                        // NDC to screen coordinates
                        let x = ((ndc_x + 1.0) * framebuffer.width as f32 / 2.0) as usize;
                        let y = ((-ndc_y + 1.0) * framebuffer.height as f32 / 2.0) as usize;
                        
                        if x < framebuffer.width && y < framebuffer.height {
                            framebuffer.set_current_color(trail_color);
                            framebuffer.point(x, y, proj_pos[2] / proj_pos[3]);
                        }
                    }
                }

                uniforms.model_matrix = create_model_matrix(body.position, body.scale, body.rotation);
                render(
                    &mut framebuffer,
                    &uniforms,
                    &vertex_arrays,
                    &body.shader_type,
                    sun_position,
                );

                // Render ring if present
                if let Some(ring) = &body.ring {
                    uniforms.model_matrix = create_model_matrix(
                        body.position,
                        body.scale * ring.scale,
                        ring.rotation,
                    );
                    render(
                        &mut framebuffer,
                        &uniforms,
                        &ring.vertex_arrays,
                        &body.shader_type,  // Use same shader as planet
                        sun_position,
                    );
                }
            }
        }

        // Always render ship when in ship mode - moved after celestial bodies but before portal effect
        if ship_mode {
            uniforms.model_matrix = create_model_matrix(ship.position, ship.scale, ship.rotation);
            render(
                &mut framebuffer,
                &uniforms,
                &ship.vertex_arrays,
                &ShaderType::RockyPlanet,
                sun_position,
            );
        }

        // Render portal effect if warping
        if let Some(_) = camera.update_warp() {
            let frame = &frames[current_frame].buffer();
            
            // Draw portal effect covering the entire screen
            for y in 0..framebuffer_height {
                for x in 0..framebuffer_width {
                    let tex_x = ((x as f32 / framebuffer_width as f32) * frame.width() as f32) as u32;
                    let tex_y = ((y as f32 / framebuffer_height as f32) * frame.height() as f32) as u32;
                    
                    if let Some(pixel) = frame.get_pixel_checked(tex_x, tex_y) {
                        let r = pixel[0] as u32;
                        let g = pixel[1] as u32;
                        let b = pixel[2] as u32;
                        let color = (r << 16) | (g << 8) | b;
                        framebuffer.set_current_color(color);
                        framebuffer.point(x, y, 0.0);
                    }
                }
            }

            // Advance to next frame
            current_frame = (current_frame + 1) % frames.len();
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
        camera.orbit(rotation_speed, 0.0, celestial_bodies);
    }
    if window.is_key_down(Key::Right) {
        camera.orbit(-rotation_speed, 0.0, celestial_bodies);
    }
    if window.is_key_down(Key::W) {
        camera.orbit(0.0, -rotation_speed, celestial_bodies);
    }
    if window.is_key_down(Key::S) {
        camera.orbit(0.0, rotation_speed, celestial_bodies);
    }

    let mut movement = Vec3::new(0.0, 0.0, 0.0);
    if window.is_key_down(Key::A) {
        movement.x -= movement_speed;
    }
    if window.is_key_down(Key::D) {
        movement.x += movement_speed;
    }

    if movement.magnitude() > 0.0 {
        camera.move_center(movement, celestial_bodies);
    }

    if window.is_key_down(Key::Up) {
        camera.zoom(zoom_speed, celestial_bodies);
    }
    if window.is_key_down(Key::Down) {
        camera.zoom(-zoom_speed, celestial_bodies);
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

    // Handle warping only if not already warping
    if !camera.warping {
        if window.is_key_pressed(Key::F1, minifb::KeyRepeat::No) {
            camera.start_warp(celestial_bodies[0].position); // Sun
        }
        if window.is_key_pressed(Key::F2, minifb::KeyRepeat::No) {
            camera.start_warp(celestial_bodies[1].position); // Mercury
        }
        if window.is_key_pressed(Key::F3, minifb::KeyRepeat::No) {
            camera.start_warp(celestial_bodies[2].position); // Venus
        }
        if window.is_key_pressed(Key::F4, minifb::KeyRepeat::No) {
            camera.start_warp(celestial_bodies[3].position); // Earth
        }
        if window.is_key_pressed(Key::F5, minifb::KeyRepeat::No) {
            camera.start_warp(celestial_bodies[4].position); // Mars
        }
        if window.is_key_pressed(Key::F6, minifb::KeyRepeat::No) {
            camera.start_warp(celestial_bodies[5].position); // Jupiter
        }
        if window.is_key_pressed(Key::F7, minifb::KeyRepeat::No) {
            camera.start_warp(celestial_bodies[6].position); // Saturn
        }
        if window.is_key_pressed(Key::F8, minifb::KeyRepeat::No) {
            camera.start_warp(celestial_bodies[7].position); // Moon
        }
    }

    // Reset camera position with R key - fix condition
    if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) && matches!(camera.warp_state, WarpState::None) {
        camera.reset_position();
    }

    // Add bird's eye view with 'B' key
    if window.is_key_pressed(Key::B, minifb::KeyRepeat::No) && !camera.warping {
        camera.bird_eye_view();
    }

    // Update warp animation
    camera.update_warp();

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

    // Update planet positions and rotations
    for body in celestial_bodies.iter_mut() {
        if body.name != "Moon" {  // Handle moon separately since it orbits Earth
            // Update orbital position
            let angle = (uniforms.time as f32 * body.orbital_speed) + body.orbital_offset;
            body.position.x = body.orbital_radius * angle.cos();
            body.position.z = body.orbital_radius * angle.sin();
            
            // Update axial rotation
            body.rotation.y += body.axial_speed;

            // Add new trail point every few frames
            if uniforms.time % 2 == 0 {
                body.trail.push(TrailPoint {
                    position: body.position,
                });
            }
        }
    }
}

// Simplify TrailPoint struct - remove age field
struct TrailPoint {
    position: Vec3,
}

struct Ring {
    obj: Obj,
    vertex_arrays: Vec<Vertex>,
    scale: f32,
    rotation: Vec3,
}


pub struct CelestialBody {
    name: String,
    position: Vec3,
    scale: f32,
    rotation: Vec3,
    shader_type: ShaderType,
    visible: bool,
    orbital_speed: f32,  // Speed of orbit around the sun
    axial_speed: f32,   // Speed of rotation around own axis
    orbital_radius: f32, // Distance from the sun
    orbital_offset: f32, // Initial angle offset
    ring: Option<Ring>,  // New field for optional ring
    trail: Vec<TrailPoint>,
    orbital_angle: f32,   // Track the accumulated orbital angle
    orbit_complete: bool, // Flag to indicate if a full orbit is completed
}
