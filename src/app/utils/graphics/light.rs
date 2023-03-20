use {
    crate::app::utils::{
        user_io::Keyboard,
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
    #[allow(dead_code)]
    pub fn new() -> Self { Self::default() }

    pub fn spawn_control_window(&mut self, ui: &imgui::Ui, keyboard: &Keyboard) {
        use {
            crate::app::utils::graphics::ui::imgui_constructor::make_window,
            std::f32::consts::PI,
        };

        make_window(ui, "Light", keyboard)
            .build(|| {
                static ANGLES: (AtomicF32, AtomicF32) = (AtomicF32::new(0.0), AtomicF32::new(0.0));

                let (mut horizontal, mut vertical) = (
                    ANGLES.0.load(Ordering::SeqCst),
                    ANGLES.1.load(Ordering::SeqCst),
                );

                ui.text("Rotation");
                let h_edited = ui.slider("Horizontal", 0.0, 2.0 * PI, &mut horizontal);
                let v_edited = ui.slider("Vertical",   0.0, 2.0 * PI, &mut vertical);

                ui.separator();
                ui.text("Position");
                ui.slider("X", -128.0, 128.0, &mut self.relative_pos.x);
                ui.slider("Y", -128.0, 128.0, &mut self.relative_pos.y);
                ui.slider("Z", -128.0, 128.0, &mut self.relative_pos.z);

                if h_edited || v_edited {
                    ANGLES.0.store(horizontal, Ordering::SeqCst);
                    ANGLES.1.store(vertical, Ordering::SeqCst);

                    self.cam.front = vec3::new(
                        f32::cos(vertical) * f32::cos(horizontal),
                        f32::sin(vertical),
                        f32::cos(vertical) * f32::sin(horizontal),
                    );
                }
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