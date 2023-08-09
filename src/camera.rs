use winit::event::{VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};

pub struct Camera{
    pub camera_pos: cgmath::Point3<f32>,
    pub camera_front: cgmath::Vector3<f32>,
    pub speed: f32,
    
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_right_pressed: bool,
    pub is_left_pressed: bool,
}

impl Camera {
    pub fn get_view(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::look_at_rh(self.camera_pos, self.camera_pos + self.camera_front, (0.0, 1.0, 0.0).into())
    }
    
    pub fn process_events(&mut self, event: &WindowEvent) -> bool{
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }    
    
    pub fn update_camera(&mut self) {
        if self.is_forward_pressed{
            self.camera_pos += self.camera_front * self.speed;
        }
        if self.is_backward_pressed{
            self.camera_pos -= self.camera_front * self.speed;
        }
        use cgmath::InnerSpace;
        let right = self.camera_front.cross((0.0, 1.0, 0.0).into()).normalize();
        if self.is_right_pressed{
            self.camera_pos += right * self.speed;
        }
        if self.is_left_pressed{
            self.camera_pos -= right * self.speed;
        }
    }
}

//(TODO): update the projection when the screen changes
pub struct Projection{
    pub aspect: f32,
    fov: f32,
    znear: f32,
    zfar: f32,
}

impl Projection{
    pub fn new(aspect: f32, fov: f32, znear: f32, zfar: f32) -> Self{
        Self{
            aspect,
            fov,
            znear,
            zfar,
        }
    }
    
    pub fn get_projection(&self) -> cgmath::Matrix4<f32>{
        cgmath::perspective(cgmath::Deg(self.fov), self.aspect, self.znear, self.zfar)
    }
}
