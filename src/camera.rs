use winit::event::{VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};

pub struct Camera{
    pub camera_pos: cgmath::Point3<f32>,
    pub camera_front: cgmath::Vector3<f32>,
    pub speed: f32,
    pub angular_speed: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn get_view(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::look_at_rh(self.camera_pos, self.camera_pos + self.camera_front, (0.0, 1.0, 0.0).into())
    }
    
    
    pub fn update_camera(&mut self, controller: &CameraController, dt: instant::Duration) {
        use cgmath::InnerSpace;
        let dt = dt.as_secs_f32();
        
        if controller.is_e_pressed{
            self.yaw += self.angular_speed * dt;
        }
        if controller.is_q_pressed{
            self.yaw -= self.angular_speed * dt;
        }
        if controller.is_up_arrow_pressed{
            self.pitch += self.angular_speed * dt;
        }
        if controller.is_down_arrow_pressed{
            self.pitch -= self.angular_speed * dt;
        }
        let mut direction = cgmath::Vector3::<f32>::new(0.0, 0.0, 0.0);
        direction.x = cgmath::Rad(self.yaw).0.cos() * cgmath::Rad(self.pitch).0.cos();
        direction.y = cgmath::Rad(self.pitch).0.sin();
        direction.z = cgmath::Rad(self.yaw).0.sin() * cgmath::Rad(self.pitch).0.cos();
        self.camera_front = direction.normalize();
        
        if controller.is_forward_pressed{
            self.camera_pos += self.camera_front * self.speed * dt;
        }
        if controller.is_backward_pressed{
            self.camera_pos -= self.camera_front * self.speed * dt;
        }
        
        let right = self.camera_front.cross((0.0, 1.0, 0.0).into()).normalize();
        if controller.is_right_pressed{
            self.camera_pos += right * self.speed * dt;
        }
        if controller.is_left_pressed{
            self.camera_pos -= right * self.speed * dt;
        }
        
        let camera_up = right.cross(self.camera_front);
        if controller.is_up_pressed{
            self.camera_pos += camera_up * self.speed * dt;
        }
        if controller.is_down_pressed{
            self.camera_pos -= camera_up * self.speed * dt;
        }
        
    }
}

pub struct CameraController{
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_right_pressed: bool,
    pub is_left_pressed: bool,
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
    pub is_q_pressed: bool,
    pub is_e_pressed: bool,
    pub is_up_arrow_pressed: bool,
    pub is_down_arrow_pressed: bool,
}

impl CameraController{
    pub fn new() -> Self{
        Self{
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_right_pressed: false,
            is_left_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_q_pressed: false,
            is_e_pressed: false,
            is_up_arrow_pressed: false,
            is_down_arrow_pressed: false,
        }
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
                    VirtualKeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::LControl => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Q => {
                        self.is_q_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::E => {
                        self.is_e_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Up => {
                        self.is_up_arrow_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Down => {
                        self.is_down_arrow_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
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
