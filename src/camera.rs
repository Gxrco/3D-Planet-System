use nalgebra_glm::{Vec3, rotate_vec3, lerp, distance};
use std::f32::consts::PI;
use crate::CelestialBody;  // Add this import at the top

#[derive(PartialEq)]
pub enum WarpState {
    None,
    PortalOpening(f32),  // progress 0.0-1.0
    Overview(f32),       // New state for top-down view
    Approaching(f32),    // New state for moving closer
    PortalClosing(f32),  // progress 0.0-1.0
}

pub struct Camera {
  pub eye: Vec3,
  pub center: Vec3,
  pub up: Vec3,
  pub has_changed: bool,
  min_zoom: f32,
  max_zoom: f32,
  max_center_distance: f32,
  pub warping: bool,
  warp_progress: f32,
  warp_start_eye: Vec3,
  warp_start_center: Vec3,
  warp_target_eye: Vec3,
  warp_target_center: Vec3,
  pub warp_state: WarpState,
  portal_radius: f32,
  overview_target: Vec3,
  final_target: Vec3,
  initial_eye: Vec3,
  initial_center: Vec3,
  initial_up: Vec3,
}

impl Camera {
  pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
    Camera {
      eye,
      center,
      up,
      has_changed: true,
      min_zoom: 10.0,        // Increased minimum distance (was 5.0)
      max_zoom: 100.0,       // Increased maximum distance (was 40.0)
      max_center_distance: 50.0,  // Increased maximum center movement (was 15.0)
      warping: false,
      warp_progress: 0.0,
      warp_start_eye: eye,
      warp_start_center: center,
      warp_target_eye: eye,
      warp_target_center: center,
      warp_state: WarpState::None,
      portal_radius: 0.0,
      overview_target: eye,
      final_target: eye,
      initial_eye: eye,
      initial_center: center,
      initial_up: up,
    }
  }

  pub fn basis_change(&self, vector: &Vec3) -> Vec3 {
    let forward = (self.center - self.eye).normalize();
    let right = forward.cross(&self.up).normalize();
    let up = right.cross(&forward).normalize();

    let rotated = 
    vector.x * right +
    vector.y * up +
    - vector.z * forward;

    rotated.normalize()
  }

  fn check_collision(&self, bodies: &[CelestialBody], new_position: Vec3) -> bool {
    for body in bodies {
        let distance = distance(&new_position, &body.position);
        let min_distance = body.scale * 1.2; // Add some padding around objects
        
        if distance < min_distance {
            return true; // Collision detected
        }
    }
    false
  }

  pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32, bodies: &[CelestialBody]) {
    let radius_vector = self.eye - self.center;
    let radius = radius_vector.magnitude();

    let current_yaw = radius_vector.z.atan2(radius_vector.x);

    let radius_xz = (radius_vector.x * radius_vector.x + radius_vector.z * radius_vector.z).sqrt();
    let current_pitch = (-radius_vector.y).atan2(radius_xz);

    let new_yaw = (current_yaw + delta_yaw) % (2.0 * PI);
    let new_pitch = (current_pitch + delta_pitch).clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);

    let new_eye = self.center + Vec3::new(
      radius * new_yaw.cos() * new_pitch.cos(),
      -radius * new_pitch.sin(),
      radius * new_yaw.sin() * new_pitch.cos()
    );

    // Only update if no collision
    if !self.check_collision(bodies, new_eye) {
        self.eye = new_eye;
        self.has_changed = true;
    }
  }

  pub fn zoom(&mut self, delta: f32, bodies: &[CelestialBody]) {
    let direction = (self.center - self.eye).normalize();
    let current_distance = (self.center - self.eye).magnitude();
    let new_distance = (current_distance - delta).clamp(self.min_zoom, self.max_zoom);
    
    let new_eye = self.center - direction * new_distance;
    
    // Only update if no collision
    if !self.check_collision(bodies, new_eye) {
        self.eye = new_eye;
        self.has_changed = true;
    }
  }

  pub fn move_center(&mut self, direction: Vec3, bodies: &[CelestialBody]) {
    let radius_vector = self.center - self.eye;
    let radius = radius_vector.magnitude();

    let angle_x = direction.x * 0.05; // Adjust this factor to control rotation speed
    let angle_y = direction.y * 0.05;

    let rotated = rotate_vec3(&radius_vector, angle_x, &Vec3::new(0.0, 1.0, 0.0));

    let right = rotated.cross(&self.up).normalize();
    let final_rotated = rotate_vec3(&rotated, angle_y, &right);

    let new_center = self.eye + final_rotated.normalize() * radius;
    
    // Only update if no collision and within bounds
    if new_center.magnitude() <= self.max_center_distance && !self.check_collision(bodies, self.eye) {
        self.center = new_center;
        self.has_changed = true;
    }
  }

  pub fn check_if_changed(&mut self) -> bool {
    if self.has_changed {
      self.has_changed = false;
      true
    } else {
      false
    }
  }

  pub fn start_warp(&mut self, target_position: Vec3) {
    // Only start warping if we're not already warping
    if self.warp_state == WarpState::None {
        self.warping = true;
        self.warp_start_eye = self.eye;
        self.warp_start_center = self.center;
        
        // Set intermediate overview position (high up)
        let overview_height = 25.0;
        let overview_pos = target_position + Vec3::new(0.0, overview_height, 0.0);
        
        // Set final viewing position (closer, angled view)
        let final_height = 12.0;
        let final_offset = Vec3::new(8.0, final_height, 8.0);
        
        self.warp_target_eye = overview_pos;
        self.warp_target_center = target_position;
        
        self.warp_state = WarpState::PortalOpening(0.0);
        self.portal_radius = 0.0;
        self.overview_target = overview_pos;
        self.final_target = target_position + final_offset;
    }
  }

  pub fn update_warp(&mut self) -> Option<f32> {
    match self.warp_state {
        WarpState::None => {
            self.warping = false;
            None
        },
        
        WarpState::PortalOpening(ref mut progress) => {
            *progress += 0.05;
            self.portal_radius = *progress;
            
            if *progress >= 1.0 {
              self.warp_state = WarpState::Overview(0.0);
            }
            Some(self.portal_radius)
        }

        WarpState::Overview(ref mut progress) => {
            *progress += 0.02;
            
            if *progress >= 1.0 {
              self.warp_state = WarpState::Approaching(0.0);
              self.warp_start_eye = self.eye;
              self.warp_target_eye = self.final_target;
            } else {
              let t = (1.0 - (*progress * PI).cos()) * 0.5;
              self.eye = lerp(&self.warp_start_eye, &self.overview_target, t);
              self.center = lerp(&self.warp_start_center, &self.warp_target_center, t);
            }
            Some(self.portal_radius)
        }

        WarpState::Approaching(ref mut progress) => {
            *progress += 0.015; // Slower approach
            
            if *progress >= 1.0 {
              self.warp_state = WarpState::PortalClosing(0.0);
            } else {
              let t = (1.0 - (*progress * PI).cos()) * 0.5;
              self.eye = lerp(&self.warp_start_eye, &self.warp_target_eye, t);
            }
            Some(self.portal_radius)
        }

        WarpState::PortalClosing(ref mut progress) => {
            *progress += 0.05;
            self.portal_radius = 1.0 - *progress;
            
            if *progress >= 1.0 {
              self.warp_state = WarpState::None;
              self.warping = false;
              None
            } else {
              Some(self.portal_radius)
            }
        }
    }
  }

  pub fn reset_position(&mut self) {
    self.eye = self.initial_eye;
    self.center = self.initial_center;
    self.up = self.initial_up;
  }

  pub fn bird_eye_view(&mut self) {
    // Only start if not already warping
    if self.warp_state == WarpState::None {
        self.warping = true;
        self.warp_start_eye = self.eye;
        self.warp_start_center = self.center;
        
        // Position high above and slightly back for better perspective
        let target_position = Vec3::new(0.0, 0.0, 0.0); // Center of solar system
        let overview_height = 80.0; // Much higher altitude
        let overview_pos = Vec3::new(-40.0, overview_height, 40.0); // Angled position for better view
        
        self.warp_target_eye = overview_pos;
        self.warp_target_center = target_position;
        
        self.warp_state = WarpState::PortalOpening(0.0);
        self.portal_radius = 0.0;
        self.overview_target = overview_pos;
        self.final_target = overview_pos;
    }
  }
}
