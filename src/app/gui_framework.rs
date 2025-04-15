use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub struct EguiFramework {
    egui_context: egui::Context,
    egui_state: egui_winit::State,
    screen_descriptor: egui_wgpu::ScreenDescriptor,
    renderer: egui_wgpu::Renderer,
    paint_jobs: Vec<egui::ClippedPrimitive>,
    textures: egui::TexturesDelta,
}

impl EguiFramework {
    pub fn new(
        device: &wgpu::Device,
        render_texture_format: wgpu::TextureFormat,
        event_loop: &ActiveEventLoop,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Self {
        let max_texture_size = device.limits().max_texture_dimension_2d as usize;

        let egui_context = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_context.clone(),
            egui::ViewportId::ROOT,
            event_loop,
            Some(scale_factor),
            Some(winit::window::Theme::Dark),
            Some(max_texture_size),
        );
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };
        let renderer = egui_wgpu::Renderer::new(device, render_texture_format, None, 1, false);
        let textures = egui::TexturesDelta::default();

        Self {
            egui_context,
            egui_state,
            screen_descriptor,
            renderer,
            paint_jobs: Vec::new(),
            textures,
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> bool {
        self.egui_state.on_window_event(window, event).consumed
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.size_in_pixels = [width, height];
        }
    }

    pub fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.pixels_per_point = scale_factor as f32;
    }

    pub fn prepare<EguiFn>(&mut self, window: &Window, mut egui_fn: EguiFn)
    where
        EguiFn: FnMut(&egui::Context),
    {
        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_context.run(raw_input, |egui_ctx| {
            egui_fn(egui_ctx);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, output.platform_output);
        self.paint_jobs = self
            .egui_context
            .tessellate(output.shapes, self.screen_descriptor.pixels_per_point);
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
    ) {
        for (id, image_delta) in &self.textures.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }
        self.renderer.update_buffers(
            device,
            queue,
            encoder,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        {
            let mut rpass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: render_target,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                })
                .forget_lifetime();

            self.renderer
                .render(&mut rpass, &self.paint_jobs, &self.screen_descriptor);
        }

        // Cleanup
        let textures = std::mem::take(&mut self.textures);
        for id in &textures.free {
            self.renderer.free_texture(id);
        }
    }
}
