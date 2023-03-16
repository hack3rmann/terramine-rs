use {
    crate::app::utils::{
        user_io::Keyboard,
        graphics::camera::Camera,
    },
    math_linear::prelude::*,
    std::sync::Mutex,
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
                static ANGLES: Mutex<(f32, f32)> = Mutex::new((0.0, 0.0));

                let mut angles = ANGLES.lock()
                    .expect("mutex should be not poisoned");

                ui.text("Rotation");
                let h_edited = ui.slider("Horizontal", 0.0, 2.0 * PI, &mut angles.0);
                let v_edited = ui.slider("Vertical",   0.0, 2.0 * PI, &mut angles.1);

                ui.separator();
                ui.text("Position");
                ui.slider("X", -128.0, 128.0, &mut self.relative_pos.x);
                ui.slider("Y", -128.0, 128.0, &mut self.relative_pos.y);
                ui.slider("Z", -128.0, 128.0, &mut self.relative_pos.z);

                let (horizontal, vertical) = *angles;

                if h_edited || v_edited {
                    self.cam.front = vec3::new(
                        f32::cos(vertical) * f32::cos(horizontal),
                        f32::sin(vertical),
                        f32::cos(vertical) * f32::sin(horizontal),
                    );
                }
            });
    }

    pub fn update(&mut self, cam_pos: vec3, cam_view_dir: vec3) {
        // FIXME:
        const Y_PLANE: f32 = 32.0;
        let interest_pos = cam_view_dir * ((Y_PLANE - cam_pos.y) / cam_view_dir.y) + cam_pos;
        
        let height = self.relative_pos.y;
        let absolute_pos = self.cam.front * ((height - interest_pos.y) / self.cam.front.y) + interest_pos;

        let (x, y, z) = absolute_pos.as_tuple();
        self.cam.set_position(x, y, z);
    }
}