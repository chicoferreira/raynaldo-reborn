use crate::app::AppState;
use egui::Ui;
use egui::emath::Numeric;
use glam::Vec3;

impl AppState {
    pub fn prepare_egui(&mut self, window: &winit::window::Window) {
        self.egui_framework.prepare(&window, |egui_ctx| {
            egui::Window::new("Hello, egui!").show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Camera position:");
                    if dragged_vec3(ui, &mut self.scene.camera.position, 0.01) {
                        self.render_state.restore_canvas();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Yaw and Pitch:");
                    let mut dir = [&mut self.scene.camera.yaw, &mut self.scene.camera.pitch];
                    if dragged_any(ui, &mut dir, 0.5) {
                        self.scene.camera.update_pixel_constants();
                        self.render_state.restore_canvas();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("FOV:");
                    if ui
                        .add(
                            egui::DragValue::new(&mut self.scene.camera.fov)
                                .speed(0.1)
                                .range(0.0..=180.0),
                        )
                        .changed()
                    {
                        self.scene.camera.update_pixel_constants();
                        self.render_state.restore_canvas();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Defocus angle:");
                    let defocus_angle = egui::DragValue::new(&mut self.scene.camera.defocus_angle)
                        .speed(0.1)
                        .range(0.0..=180.0);
                    if ui.add(defocus_angle).changed() {
                        self.scene.camera.update_pixel_constants();
                        self.render_state.restore_canvas();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Focus distance:");
                    let focus_distance =
                        egui::DragValue::new(&mut self.scene.camera.focus_distance)
                            .speed(0.1)
                            .range(0.1..=100.0);
                    if ui.add(focus_distance).changed() {
                        self.scene.camera.update_pixel_constants();
                        self.render_state.restore_canvas();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Time budget:");
                    ui.add(
                        egui::DragValue::new(&mut self.time_budget_ms)
                            .speed(0.1)
                            .range(0.1..=200.0)
                            .suffix("ms"),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Samples per pixel:");
                    let drag_samples_per_pixel = egui::DragValue::new(&mut self.samples_per_pixel)
                        .speed(1.0)
                        .range(1..=1000);
                    if ui.add(drag_samples_per_pixel).changed() {
                        self.render_state.restore_canvas();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Max ray depth:");
                    let drag_max_depth = egui::DragValue::new(&mut self.max_ray_depth)
                        .speed(1.0)
                        .range(1..=1000);
                    if ui.add(drag_max_depth).changed() {
                        self.render_state.restore_canvas();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Render progress:");
                    let progress = self.render_state.progress(self.samples_per_pixel);
                    ui.add(
                        egui::ProgressBar::new(progress)
                            .show_percentage()
                            .animate(true)
                            .desired_width(150.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Frame rate:");
                    ui.label(format!("{:.2} fps", self.last_fps_update.1));
                });
            });
        });
    }
}

pub fn dragged_any<N: Numeric>(ui: &mut Ui, vec: &mut [&mut N], speed: f32) -> bool {
    let mut changed = false;
    for v in vec.iter_mut() {
        if ui.add(egui::DragValue::new(*v).speed(speed)).changed() {
            changed = true;
        }
    }
    changed
}

pub fn dragged_vec3(ui: &mut Ui, vec: &mut Vec3, speed: f32) -> bool {
    dragged_any(ui, &mut [&mut vec.x, &mut vec.y, &mut vec.z], speed)
}
