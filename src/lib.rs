use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod camera;
mod chunk;
mod render;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

struct State {
    camera: camera::Camera,
    camera_controller: camera::CameraController,
    projection: camera::Projection,
    render: render::Render,
}

impl State {
    async fn new(window: Window) -> Self {
        //positive Z points away from the screen
        let camera = camera::Camera {
            camera_pos: (0.0, 0.0, 3.0).into(),
            camera_front: (0.0, 0.0, -1.0).into(),
            speed: 5.0,
            angular_speed: 2.0,
            yaw: -45.0,
            pitch: 0.0,
        };
        let render = render::Render::new(window).await;

        let camera_controller = camera::CameraController::new();
        let projection =
            camera::Projection::new(render.width() / render.height(), 45.0, 0.1, 100.0);

        Self {
            camera,
            camera_controller,
            projection,
            render,
        }
    }

    pub fn window(&self) -> &Window {
        self.render.window()
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if self.render.resize(new_size) {
            self.projection.aspect = self.render.width() / self.render.height();
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event);
        self.render.process_events(event);
        false
    }

    fn update(&mut self, dt: instant::Duration) {
        self.camera.update_camera(&self.camera_controller, dt);
        self.render.update(&self.camera, &self.projection);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(window).await;
    let mut last_render_time = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);
                match state.render.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.render.resize(state.render.get_size());
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}
