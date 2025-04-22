use crate::app::gui_framework::EguiFramework;
use crate::app::renderer::Renderer;
use crate::raytracer::camera::Camera;
use crate::raytracer::world::World;
use crate::raytracer::Scene;
use pollster::FutureExt;
use rand::prelude::SliceRandom;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::time::{Duration, Instant};
use winit::dpi::Size;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

mod gui;
mod gui_framework;
mod renderer;
mod winit_app;

struct RenderState {
    pixel_render_order: Vec<usize>,
    current_render_pixel: usize,
    canvas: Vec<u8>,
}

impl RenderState {
    fn new(width: u32, height: u32) -> Self {
        let len = (width * height) as usize;
        let mut pixel_render_order: Vec<_> = (0..len).collect();
        pixel_render_order.shuffle(&mut rand::rng());

        let canvas = vec![0; len * 4];

        Self {
            pixel_render_order,
            current_render_pixel: 0,
            canvas,
        }
    }

    fn is_finished(&self) -> bool {
        self.current_render_pixel == self.pixel_render_order.len()
    }

    fn missing_pixels(&self) -> usize {
        self.pixel_render_order.len() - self.current_render_pixel
    }

    fn restore_canvas(&mut self) {
        self.canvas.fill(0);
        self.current_render_pixel = 0;
    }

    fn progress(&self) -> f32 {
        self.current_render_pixel as f32 / (self.pixel_render_order.len() as f32)
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        let len = (width * height) as usize;
        self.pixel_render_order.clear();
        self.pixel_render_order.extend(0..len);
        self.pixel_render_order.shuffle(&mut rand::rng());

        self.canvas.resize(len * 4, 0);
        self.canvas.fill(0);
        self.current_render_pixel = 0;
    }
}

struct AppState {
    render_state: RenderState,
    samples_per_pixel: u32,
    max_ray_depth: u32,
    time_budget_ms: u64,
    scene: Scene,
    last_fps_update: (Instant, f64),
    last_frame: Instant,
    renderer: Renderer,
    egui_framework: EguiFramework,
}

pub(crate) fn run(world: World, camera_settings: CameraSettings) {
    let app = winit_app::WinitApp::new(
        |event_loop| {
            event_loop
                .create_window(
                    winit::window::WindowAttributes::default()
                        .with_title("Raynaldo")
                        .with_min_inner_size(Size::Physical((100, 100).into())),
                )
                .unwrap()
        },
        |event_loop, window| {
            let scale_factor = window.scale_factor() as f32;
            let renderer = Renderer::new(window).block_on();
            let egui_framework = EguiFramework::new(
                &renderer.device,
                renderer.surface_format,
                event_loop,
                renderer.width(),
                renderer.height(),
                scale_factor,
            );

            let state = AppState {
                render_state: RenderState::new(renderer.width(), renderer.height()),
                samples_per_pixel: 5,
                max_ray_depth: 10,
                time_budget_ms: 10,
                scene: Scene::new(
                    camera_settings.to_camera(renderer.width(), renderer.height(), 2.0),
                    world.clone(),
                ),
                last_fps_update: (Instant::now(), 0.0),
                last_frame: Instant::now(),
                renderer,
                egui_framework,
            };

            state
        },
        |_window, state, input, event_loop| {
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                event_loop.exit();
            }

            if let Some(size) = input.window_resized() {
                if size.width != 0 && size.height != 0 {
                    state.renderer.update_size(size.width, size.height);
                    state.egui_framework.resize(size.width, size.height);
                    state.scene.update_screen_size(size.width, size.height);
                    state.render_state.on_resize(size.width, size.height);
                }
            }

            if let Some(scale_factor) = input.scale_factor() {
                state.egui_framework.scale_factor(scale_factor);
            }

            if let Some(delta_time) = input.delta_time() {
                let delta_time = delta_time.as_secs_f32();
                let right_input =
                    input.key_held(KeyCode::KeyD) as i32 - input.key_held(KeyCode::KeyA) as i32;
                let up_input = input.key_held(KeyCode::Space) as i32
                    - input.key_held(KeyCode::ShiftLeft) as i32;
                let forward_input =
                    input.key_held(KeyCode::KeyW) as i32 - input.key_held(KeyCode::KeyS) as i32;
                let (yaw_input, pitch_input) = if input.mouse_held(MouseButton::Left) {
                    input.mouse_diff()
                } else {
                    (0.0, 0.0)
                };

                if right_input != 0
                    || up_input != 0
                    || forward_input != 0
                    || yaw_input != 0.0
                    || pitch_input != 0.0
                {
                    state.scene.camera.process_input(
                        right_input as f32,
                        up_input as f32,
                        forward_input as f32,
                        yaw_input,
                        pitch_input,
                        delta_time,
                    );
                    state.render_state.restore_canvas();
                }
            }
        },
        |state, window, event| !state.egui_framework.handle_event(window, event),
        |window, state, _event_loop| {
            let width = state.renderer.width();

            state.prepare_egui(&window);

            const PIXEL_BATCH_SIZE: usize = 10000;

            let instant = Instant::now();
            while !state.render_state.is_finished()
                && instant.elapsed() < Duration::from_millis(state.time_budget_ms)
            {
                #[derive(Copy, Clone)]
                struct BufferWrapper(*mut u8);

                // SAFETY: this is safe because no simultaneous access to the same index happens
                unsafe impl Send for BufferWrapper {}
                unsafe impl Sync for BufferWrapper {}

                let buffer_ptr = BufferWrapper(state.render_state.canvas.as_mut_ptr());

                let size = PIXEL_BATCH_SIZE.min(state.render_state.missing_pixels());
                let end = state.render_state.current_render_pixel + size;

                state.render_state.pixel_render_order[state.render_state.current_render_pixel..end]
                    .par_iter()
                    .for_each(|&index_in_buffer| {
                        let x = index_in_buffer as u32 % width;
                        let y = index_in_buffer as u32 / width;
                        let color = state.scene.render_pixel(
                            x,
                            y,
                            state.samples_per_pixel,
                            state.max_ray_depth,
                        );

                        let index = index_in_buffer * 4;

                        // SAFETY: this is safe because pixel_render_order
                        // has unique indices to the buffer
                        unsafe {
                            // copy the buffer, otherwise the compiler only sees that only
                            // buffer_ptr.0 is being used, and captures that instead
                            let buffer_ptr = buffer_ptr;
                            *buffer_ptr.0.add(index) = (color.x * 255.0) as u8;
                            *buffer_ptr.0.add(index + 1) = (color.y * 255.0) as u8;
                            *buffer_ptr.0.add(index + 2) = (color.z * 255.0) as u8;
                            *buffer_ptr.0.add(index + 3) = (color.w * 255.0) as u8;
                        }
                    });

                state.render_state.current_render_pixel += size;
            }

            state
                .renderer
                .render_with(&state.render_state.canvas, |renderer, encoder, view| {
                    state
                        .egui_framework
                        .render(&renderer.device, &renderer.queue, encoder, view);
                });

            if state.last_fps_update.0.elapsed() > Duration::from_millis(100) {
                let fps = 1.0 / state.last_frame.elapsed().as_secs_f64();
                state.last_fps_update = (Instant::now(), fps);
            }
            state.last_frame = Instant::now();
        },
    );

    app.run().unwrap()
}

#[derive(Clone)]
pub struct CameraSettings {
    pub position: glam::Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub focus_distance: f32,
    pub defocus_angle: f32,
}

impl CameraSettings {
    fn to_camera(&self, width: u32, height: u32, sensibility: f32) -> Camera {
        Camera::new(
            width,
            height,
            self.fov,
            self.position,
            self.yaw,
            self.pitch,
            sensibility,
            self.focus_distance,
            self.defocus_angle,
        )
    }
}
