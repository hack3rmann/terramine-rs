use crate::{
    prelude::*,
    graphics::camera_resource::Camera,
};



#[derive(Debug, Default)]
pub struct DirectionalLight {
    pub cam: Camera,
    pub relative_pos: Vec3,
}
assert_impl_all!(DirectionalLight: Send, Sync);

impl DirectionalLight {
    // FIXME:
    // pub fn spawn_control_window(&mut self, ui: &imgui::Ui) {
    //     use {
    //         crate::app::utils::graphics::ui::imgui_ext::make_window,
    //         std::f32::consts::PI,
    //     };

    //     make_window(ui, "Light").build(|| {
    //         macros::atomic_static! {
    //             static HORIZONTAL: f32 = 0.3;
    //             static VERTICAL: f32 = 5.7;
    //         }

    //         let (mut horizontal, mut vertical) = macros::load!(Acquire: HORIZONTAL, VERTICAL);

    //         ui.text("Rotation");
    //         ui.slider("Horizontal", 0.0, 2.0 * PI, &mut horizontal);
    //         ui.slider("Vertical",   0.0, 2.0 * PI, &mut vertical);

    //         macros::store!(Release: HORIZONTAL = horizontal, VERTICAL = vertical);

    //         self.cam.front = vec3::new(
    //             f32::cos(vertical) * f32::cos(horizontal),
    //             f32::sin(vertical),
    //             f32::cos(vertical) * f32::sin(horizontal),
    //         );
    //     });
    // }

    pub fn update(&mut self, cam_pos: Vec3) {
        let interest_pos = cam_pos;
        
        let height = self.relative_pos.y;
        let front = Vec3::from_array(self.cam.front.as_array());
        let absolute_pos = front * ((height - interest_pos.y) / front.y) + interest_pos;

        let [x, y, z] = absolute_pos.to_array();
        self.cam.set_position(x, y, z);
    }
}