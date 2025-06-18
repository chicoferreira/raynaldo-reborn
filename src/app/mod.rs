use crate::app::gui_framework::EguiFramework;
use crate::app::renderer::Renderer;
use crate::raytracer::Scene;
use crate::raytracer::camera::Camera;
use crate::raytracer::world::World;
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
    /// The pixel render orders are used to randomize the order in which pixels are rendered.
    /// There are multiple orders to avoid generating each frame when moving the camera.
    /// Instead we just switch between them.
    pixel_render_orders: Vec<Vec<usize>>,
    /// Tracks the current pixel render order.
    current_pixel_render_order: usize,
    /// The current pixel being rendered.
    current_render_pixel: usize,
    /// The canvas that holds the accumulated values: (r_sum, g_sum, b_sum, sample_count)
    canvas: Vec<(f32, f32, f32, u32)>,
    /// The total number of samples that have been rendered.
    /// This needs to be equal to height * width * samples_per_pixel to be considered finished.
    total_rendered_pixel_samples: usize,
    /// The number of samples per pixel that have been rendered.
    rendered_samples: u32,
}

impl RenderState {
    fn generate_pixel_render_orders(len: usize) -> Vec<Vec<usize>> {
        const NUMBER_OF_ORDERS: usize = 5;

        (0..NUMBER_OF_ORDERS)
            .map(|_| {
                let mut pixel_render_order: Vec<_> = (0..len).collect();
                pixel_render_order.shuffle(&mut rand::rng());
                pixel_render_order
            })
            .collect()
    }

    fn new(width: u32, height: u32) -> Self {
        let len = (width * height) as usize;
        let pixel_render_orders = Self::generate_pixel_render_orders(len);
        let canvas = vec![(0.0, 0.0, 0.0, 0); len];

        Self {
            pixel_render_orders,
            current_pixel_render_order: 0,
            current_render_pixel: 0,
            total_rendered_pixel_samples: 0,
            rendered_samples: 0,
            canvas,
        }
    }

    fn get_current_pixel_order(&self) -> &[usize] {
        &self.pixel_render_orders[self.current_pixel_render_order]
    }

    fn is_finished(&self, samples_per_pixel: u32) -> bool {
        self.missing_pixels(samples_per_pixel) == 0
    }

    fn is_finished_current_order(&self) -> bool {
        self.missing_pixels_in_current_order() == 0
    }

    fn missing_pixels_in_current_order(&self) -> usize {
        self.get_current_pixel_order().len() - self.current_render_pixel
    }

    fn total_pixels_to_render(&self, samples_per_pixel: u32) -> usize {
        self.canvas.len() * samples_per_pixel as usize
    }

    fn missing_pixels(&self, samples_per_pixel: u32) -> usize {
        self.total_pixels_to_render(samples_per_pixel) - self.total_rendered_pixel_samples
    }

    fn restore_canvas(&mut self) {
        self.canvas.fill((0.0, 0.0, 0.0, 0));
        self.current_render_pixel = 0;
        self.current_pixel_render_order =
            (self.current_pixel_render_order + 1) % self.pixel_render_orders.len();
        self.rendered_samples = 0;
        self.total_rendered_pixel_samples = 0;
    }

    fn progress(&self, samples_per_pixel: u32) -> f32 {
        self.total_rendered_pixel_samples as f32
            / self.total_pixels_to_render(samples_per_pixel) as f32
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.canvas.len() * 4);
        for (r_sum, g_sum, b_sum, sample_count) in &self.canvas {
            if *sample_count > 0 {
                let r = ((r_sum / *sample_count as f32) * 255.0).clamp(0.0, 255.0) as u8;
                let g = ((g_sum / *sample_count as f32) * 255.0).clamp(0.0, 255.0) as u8;
                let b = ((b_sum / *sample_count as f32) * 255.0).clamp(0.0, 255.0) as u8;
                let a = 255u8;
                bytes.extend_from_slice(&[r, g, b, a]);
            } else {
                bytes.extend_from_slice(&[0, 0, 0, 255]);
            }
        }
        bytes
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        let len = (width * height) as usize;
        for pixel_render_order in self.pixel_render_orders.iter_mut() {
            pixel_render_order.clear();
            pixel_render_order.extend(0..len);
            pixel_render_order.shuffle(&mut rand::rng());
        }

        self.canvas.resize(len, (0.0, 0.0, 0.0, 0));
        self.restore_canvas();
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
            while !state.render_state.is_finished(state.samples_per_pixel)
                && instant.elapsed() < Duration::from_millis(state.time_budget_ms)
            {
                #[derive(Copy, Clone)]
                struct BufferWrapper(*mut (f32, f32, f32, u32));

                // SAFETY: this is safe because no simultaneous access to the same index happens
                unsafe impl Send for BufferWrapper {}
                unsafe impl Sync for BufferWrapper {}

                let buffer_ptr = BufferWrapper(state.render_state.canvas.as_mut_ptr());

                let size =
                    PIXEL_BATCH_SIZE.min(state.render_state.missing_pixels_in_current_order());
                let end = state.render_state.current_render_pixel + size;

                state.render_state.get_current_pixel_order()
                    [state.render_state.current_render_pixel..end]
                    .par_iter()
                    .for_each(|&index_in_buffer| {
                        let x = index_in_buffer as u32 % width;
                        let y = index_in_buffer as u32 / width;
                        let color = state.scene.render_sample(x, y, state.max_ray_depth);

                        // SAFETY: this is safe because pixel_render_order
                        // has unique indices to the buffer
                        unsafe {
                            // accumulate to the buffer, otherwise the compiler only sees that only
                            // buffer_ptr.0 is being used, and captures that instead
                            let buffer_ptr = buffer_ptr;
                            let buffer_ptr = buffer_ptr.0.add(index_in_buffer);

                            (*buffer_ptr).0 += color.x;
                            (*buffer_ptr).1 += color.y;
                            (*buffer_ptr).2 += color.z;
                            (*buffer_ptr).3 += 1;
                        }
                    });

                state.render_state.current_render_pixel += size;
                state.render_state.total_rendered_pixel_samples += size;

                // Reset the pixel render order if the render is not finished (number of samples for each pixel has not been reached)
                if state.render_state.is_finished_current_order()
                    && !state.render_state.is_finished(state.samples_per_pixel)
                {
                    state.render_state.current_render_pixel = 0;
                    state.render_state.rendered_samples += 1;
                    // TODO: change the pixel render order
                }
            }

            let canvas_bytes = state.render_state.to_bytes();
            state
                .renderer
                .render_with(&canvas_bytes, |renderer, encoder, view| {
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
