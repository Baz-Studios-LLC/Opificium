//! The bench camera: orbit, zoom and pan, tuned to the game's hand-feel.

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

#[derive(Resource)]
pub struct OrbitRig {
    pub focus: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Default for OrbitRig {
    fn default() -> Self {
        OrbitRig {
            focus: Vec3::ZERO,
            yaw: 0.6,
            pitch: 0.7,
            distance: 16.0,
        }
    }
}

impl OrbitRig {
    fn eye(&self) -> Vec3 {
        let flat = Vec2::from_angle(self.yaw) * self.pitch.cos();
        self.focus + Vec3::new(flat.x, self.pitch.sin(), flat.y) * self.distance
    }

    fn ground_forward(&self) -> Vec3 {
        let eye_to_focus = self.focus - self.eye();
        Vec3::new(eye_to_focus.x, 0.0, eye_to_focus.z).normalize_or_zero()
    }

    fn ground_right(&self) -> Vec3 {
        // forward x up is the true right hand; the negation that used to
        // sit here swapped A and D and dragged the bench backwards.
        self.ground_forward().cross(Vec3::Y)
    }
}

/// What the eye turns about, at the benches that lock it there.
///
/// Stated by the bench rather than fixed here, because the thing being looked at decides:
/// a body's chest is at a metre, and a model's middle is at half of whatever the model
/// happens to be. The eye that orbits it needs to be told.
#[derive(Resource)]
pub struct Centre(pub Vec3);

impl Default for Centre {
    fn default() -> Self {
        // Chest height on a person, which is the right guess before anything stands.
        Self(Vec3::new(0.0, 1.0, 0.0))
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrbitRig>()
            .init_resource::<Centre>()
            .add_systems(Startup, spawn_camera)
            // After the panes have had the wheel: they decide whether the
            // cursor is over a list, and a stale answer made the zoom
            // steal scrolls on half the frames.
            .add_systems(
                Update,
                (steer, place).chain().after(crate::look::scroll_panes),
            );
    }
}

fn spawn_camera(mut commands: Commands, rig: Res<OrbitRig>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(rig.eye()).looking_at(rig.focus, Vec3::Y),
    ));
}

fn steer(
    keys: Res<ButtonInput<KeyCode>>,
    naming: Res<crate::builder::Naming>,
    dims: Res<crate::builder::DimsEntry>,
    bench: Res<crate::Bench>,
    over_pane: Res<crate::look::OverPane>,
    centre: Res<Centre>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
    mut rig: ResMut<OrbitRig>,
) {
    let terrain = *bench == crate::Bench::Terrain;
    // At the terrain bench, SHIFT is the camera. Both mouse buttons there are
    // tools - the left lays the brush down and the right takes it back off - so
    // the eye needs a modifier rather than a button of its own, and a maker
    // lowering a hill must never also swing the view out from under themselves.
    let shift = terrain && keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    let turning = if terrain {
        shift && buttons.pressed(MouseButton::Left)
    } else {
        buttons.pressed(MouseButton::Right)
    };
    if turning {
        rig.yaw += motion.delta.x * 0.008;
        rig.pitch = (rig.pitch + motion.delta.y * 0.008).clamp(0.12, 1.55);
    }

    // While a name or a measure is being typed, the keyboard belongs
    // to the pen.
    if naming.0.is_some() || dims.0.is_some() {
        return;
    }

    // Snapped views on the number row: the drafting angles. Front is the
    // door side - the gold sill - and overhead is the plan view. At the terrain
    // bench the bare row holds the tools, so these move onto Shift with the
    // rest of the camera.
    let pi = std::f32::consts::PI;
    let drafting = !terrain || shift;
    for (key, yaw, pitch) in [
        (KeyCode::Digit1, 0.0, 0.32),       // front, the door side
        (KeyCode::Digit2, pi * 0.5, 0.32),  // right side
        (KeyCode::Digit3, pi, 0.32),        // back
        (KeyCode::Digit4, -pi * 0.5, 0.32), // left side
        (KeyCode::Digit5, 0.0, 1.55),       // overhead, the plan
        (KeyCode::Digit6, 0.6, 0.7),        // the working perch
    ] {
        if drafting && keys.just_pressed(key) {
            rig.yaw = yaw;
            rig.pitch = pitch;
        }
    }

    // The rig bench turns about the body and nothing else. Brett: "lock the
    // camera to the center so that you can orbit the cahracter and zoom but not
    // pan". A body is one thing standing in one place - there is nowhere to pan
    // TO - and a bench that let the eye wander off it would only ever be
    // something to correct.
    if *bench == crate::Bench::Rig {
        if scroll.delta.y != 0.0 && !over_pane.0 {
            // Closer than a body ever needed. A model can be a fifth of a metre tall,
            // and a floor of 1.2m would hold the eye further out than the whole thing.
            rig.distance = (rig.distance * (1.0 - scroll.delta.y * 0.08)).clamp(0.15, 20.0);
        }
        if rig.focus.distance(centre.0) > 1e-4 {
            rig.focus = centre.0;
        }
        return;
    }

    // Pan on middle mouse: the bench moves with the pointer.
    if buttons.pressed(MouseButton::Middle) && motion.delta != Vec2::ZERO {
        let scale = rig.distance * 0.0016;
        let right = rig.ground_right();
        let forward = rig.ground_forward();
        rig.focus -= right * motion.delta.x * scale;
        rig.focus += forward * motion.delta.y * scale;
    }

    // Keyboard glide, WASD and arrows both.
    let mut pan = Vec3::ZERO;
    if keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        pan += rig.ground_forward();
    }
    if keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        pan -= rig.ground_forward();
    }
    if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        pan -= rig.ground_right();
    }
    if keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        pan += rig.ground_right();
    }
    if pan != Vec3::ZERO {
        let speed = (0.35 + rig.distance * 0.12) * 8.0 * time.delta_secs();
        rig.focus += pan.normalize() * speed;
    }

    // The wheel draws near and pulls away - unless the cursor is over a
    // pane, where the wheel belongs to the list.
    if scroll.delta.y != 0.0 && !over_pane.0 {
        // A world is kilometres across where a building is metres, so the eye
        // has to be able to stand far enough back to see a coastline.
        let (near, far) = if terrain {
            (20.0, 6_000.0)
        } else {
            (3.0, 60.0)
        };
        rig.distance = (rig.distance * (1.0 - scroll.delta.y * 0.08)).clamp(near, far);
    }
}

fn place(rig: Res<OrbitRig>, mut cameras: Query<&mut Transform, With<Camera3d>>) {
    for mut transform in &mut cameras {
        *transform = Transform::from_translation(rig.eye()).looking_at(rig.focus, Vec3::Y);
    }
}
