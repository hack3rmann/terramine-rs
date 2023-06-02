use {
    crate::{
        prelude::*,
        graphics::camera_resource::Camera,
    }
};

#[derive(Debug, Default)]
pub struct DirectionalLight {
    pub cam: Camera,
    pub relative_pos: vec3,
}
assert_impl_all!(DirectionalLight: Send, Sync);

impl DirectionalLight {
    pub fn spawn_control_window(&mut self, ui: &imgui::Ui) {
        use {
            crate::app::utils::graphics::ui::imgui_ext::make_window,
            std::f32::consts::PI,
        };

        make_window(ui, "Light").build(|| {
            static ANGLES: (AtomicF32, AtomicF32) = (AtomicF32::new(0.3), AtomicF32::new(5.7));

            let (mut horizontal, mut vertical) = (
                ANGLES.0.load(Acquire),
                ANGLES.1.load(Acquire),
            );

            ui.text("Rotation");
            ui.slider("Horizontal", 0.0, 2.0 * PI, &mut horizontal);
            ui.slider("Vertical",   0.0, 2.0 * PI, &mut vertical);

            ANGLES.0.store(horizontal, Release);
            ANGLES.1.store(vertical, Release);

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