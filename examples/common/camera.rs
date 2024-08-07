use witer::{Key, Message};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
  pub eye: cgmath::Point3<f32>,
  pub target: cgmath::Point3<f32>,
  pub up: cgmath::Vector3<f32>,
  pub aspect: f32,
  pub fovy: f32,
  pub znear: f32,
  pub zfar: f32,
}

impl Camera {
  fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
    // 1.
    let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

    // 2.
    let proj =
      cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

    // 3.
    OPENGL_TO_WGPU_MATRIX * proj * view
  }
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
  // We can't use cgmath with bytemuck directly, so we'll have
  // to convert the Matrix4 into a 4x4 f32 array
  pub view_proj: [[f32; 4]; 4],
  pub pos: [f32; 3],
  pub padding: u32,
}

impl CameraUniform {
  pub fn new() -> Self {
    use cgmath::SquareMatrix;
    Self {
      view_proj: cgmath::Matrix4::identity().into(),
      pos: [0.0; 3],
      padding: 0,
    }
  }

  pub fn update_view_proj(&mut self, camera: &Camera) {
    self.view_proj = camera.build_view_projection_matrix().into();
    self.pos = camera.eye.into();
  }
}

pub struct CameraController {
  pub speed: f32,
  pub is_forward_pressed: bool,
  pub is_backward_pressed: bool,
  pub is_left_pressed: bool,
  pub is_right_pressed: bool,
}

impl CameraController {
  pub fn new(speed: f32) -> Self {
    Self {
      speed,
      is_forward_pressed: false,
      is_backward_pressed: false,
      is_left_pressed: false,
      is_right_pressed: false,
    }
  }

  pub fn process_events(&mut self, event: &Message) -> bool {
    match event {
      Message::Key { key, state, .. } => {
        let is_pressed = state.is_pressed();
        match key {
          Key::E | Key::Up => {
            self.is_forward_pressed = is_pressed;
            true
          }
          Key::S | Key::Left => {
            self.is_left_pressed = is_pressed;
            true
          }
          Key::D | Key::Down => {
            self.is_backward_pressed = is_pressed;
            true
          }
          Key::F | Key::Right => {
            self.is_right_pressed = is_pressed;
            true
          }
          _ => false,
        }
      }
      _ => false,
    }
  }

  pub fn update_camera(&self, camera: &mut Camera, delta_time: f32) {
    use cgmath::InnerSpace;
    let forward = camera.target - camera.eye;
    let forward_norm = forward.normalize();
    let forward_mag = forward.magnitude();

    let velocity = delta_time * self.speed;

    // Prevents glitching when the camera gets too close to the
    // center of the scene.
    if self.is_forward_pressed && forward_mag > velocity {
      camera.eye += forward_norm * velocity;
    }
    if self.is_backward_pressed {
      camera.eye -= forward_norm * velocity;
    }

    let right = forward_norm.cross(camera.up);

    // Redo radius calc in case the forward/backward is pressed.
    let forward = camera.target - camera.eye;
    let forward_mag = forward.magnitude();

    if self.is_right_pressed {
      // Rescale the distance between the target and the eye so
      // that it doesn't change. The eye, therefore, still
      // lies on the circle made by the target and eye.
      camera.eye = camera.target - (forward + right * velocity).normalize() * forward_mag;
    }
    if self.is_left_pressed {
      camera.eye = camera.target - (forward - right * velocity).normalize() * forward_mag;
    }
  }
}