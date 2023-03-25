use {
    crate::app::utils::{
        graphics::camera::Camera,
    },
    math_linear::prelude::*,
    portable_atomic::AtomicF32,
    std::sync::atomic::Ordering,
};

#[derive(Debug, Default)]
pub struct DirectionalLight {
    pub cam: Camera,
    pub relative_pos: vec3,
}

impl DirectionalLight {
    pub fn spawn_control_window(&mut self, ui: &imgui::Ui) {
        use {
            crate::app::utils::graphics::ui::imgui_constructor::make_window,
            std::f32::consts::PI,
        };

        make_window(ui, "Light").build(|| {
            static ANGLES: (AtomicF32, AtomicF32) = (AtomicF32::new(0.3), AtomicF32::new(5.7));

            let (mut horizontal, mut vertical) = (
                ANGLES.0.load(Ordering::SeqCst),
                ANGLES.1.load(Ordering::SeqCst),
            );

            ui.text("Rotation");
            ui.slider("Horizontal", 0.0, 2.0 * PI, &mut horizontal);
            ui.slider("Vertical",   0.0, 2.0 * PI, &mut vertical);

            ANGLES.0.store(horizontal, Ordering::SeqCst);
            ANGLES.1.store(vertical, Ordering::SeqCst);

            self.cam.front = vec3::new(
                f32::cos(vertical) * f32::cos(horizontal),
                f32::sin(vertical),
                f32::cos(vertical) * f32::sin(horizontal),
            );
        });
    }

    pub fn update(&mut self, cam_pos: vec3) {
        let interest_pos = cam_pos;
        
        let height = self.relative_pos.y;
        let absolute_pos = self.cam.front * ((height - interest_pos.y) / self.cam.front.y) + interest_pos;

        let (x, y, z) = absolute_pos.as_tuple();
        self.cam.set_position(x, y, z);
    }
}