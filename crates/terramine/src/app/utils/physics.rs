use crate::{prelude::*, transform::{Translation, Transform}};



#[derive(Clone, Debug, ConstDefault)]
pub struct KinematicState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
}
assert_impl_all!(KinematicState: Send, Sync);



#[derive(Clone, SmartDefault)]
pub struct PhysicalComponent {
    pub kin_state: KinematicState,

    #[default = 1.0]
    pub mass: f32,

    /// In this model, **acceleration** is the function
    /// of **position**, **velocity** and previous **acceleration**.
    #[default(Arc::new(|_, _, a| a))]
    pub acceleration_function: Arc<dyn Fn(Vec3, Vec3, Vec3) -> Vec3 + Send + Sync>,
}
assert_impl_all!(PhysicalComponent: Send, Sync, Component);

impl PhysicalComponent {
    pub fn from_mass(mass: f32) -> Self {
        Self { mass, ..default() }
    }

    pub fn from_kinematic_state(mass: f32, kin_state: KinematicState) -> Self {
        Self { mass, kin_state, ..default() }
    }

    pub fn step(&mut self, ts: TimeStep) {
        let dt = ts.as_secs_f32();

        self.kin_state.acceleration = (self.acceleration_function)(
            self.kin_state.position,
            self.kin_state.velocity,
            self.kin_state.acceleration,
        );
        
        self.kin_state.velocity += self.kin_state.acceleration * dt;

        self.kin_state.position += self.kin_state.velocity * dt + 0.5 * self.kin_state.acceleration * dt * dt;
    }

    pub fn force(&self) -> Vec3 {
        self.kin_state.acceleration * self.mass
    }

    pub fn apply_force(&mut self, force: Vec3) {
        self.accelerate(force / self.mass);
    }

    pub fn accelerate(&mut self, accel: Vec3) {
        self.kin_state.acceleration += accel;
    }

    pub fn momentum(&self) -> Vec3 {
        self.kin_state.velocity * self.mass
    }

    pub fn apply_momentum(&mut self, momentum: Vec3) {
        self.add_velocity(momentum / self.mass)
    }

    pub fn add_velocity(&mut self, velocity: Vec3) {
        self.kin_state.velocity += velocity;
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.kin_state.velocity.length_squared()
    }

    pub fn update_all(world: &World) -> AnyResult<()> {
        let mut query = world.query::<&mut PhysicalComponent>();
        let ts = world.resource::<&Timer>()?.time_step();

        for (_entity, state) in query.into_iter() {
            state.step(ts);
        }

        Ok(())
    }

    pub fn affect_translation(&self, translation: &mut Translation) {
        translation.position = self.kin_state.position;
    }

    pub fn affect_transform(&self, transform: &mut Transform) {
        self.affect_translation(&mut transform.translation);
    }
}

impl Debug for PhysicalComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalComponent")
            .field("kin_state", &self.kin_state)
            .field("mass", &self.mass)
            .finish_non_exhaustive()
    }
}