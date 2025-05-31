use std::{f32::consts::FRAC_PI_2, ops::Range};

use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};

fn setup(mut commands: Commands, camera_settings: Res<CameraSettings>) {
    commands.spawn((
        PointLight {
            shadows_enabled: false,
            intensity: 10_000_000.,
            range: 100.0,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, camera_settings.orbit_distance)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn orbit(
    mut camera: Single<&mut Transform, With<Camera>>,
    camera_settings: Res<CameraSettings>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
) {
    let delta = mouse_motion.delta;
    if mouse_buttons.pressed(MouseButton::Left) {
        let delta_pitch = delta.y * -camera_settings.pitch_speed;
        let delta_yaw = delta.x * -camera_settings.yaw_speed;

        // Obtain the existing pitch, yaw, and roll values from the transform.
        let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);

        // Establish the new yaw and pitch, preventing the pitch value from exceeding our limits.
        let pitch = (pitch + delta_pitch).clamp(
            camera_settings.pitch_range.start,
            camera_settings.pitch_range.end,
        );
        let yaw = yaw + delta_yaw;
        camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
    let target = Vec3::ZERO;
    camera.translation = target - camera.forward() * camera_settings.orbit_distance;
}

fn zoom(
    mut camera: Single<&mut Transform, With<Camera>>,
    mut camera_settings: ResMut<CameraSettings>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
) {
    camera_settings.orbit_distance = (camera_settings.orbit_distance
        * (1.0 - mouse_scroll.delta.y * camera_settings.zoom_speed))
        .clamp(
            camera_settings.zoom_range.start,
            camera_settings.zoom_range.end,
        );
    let target = Vec3::ZERO;
    camera.translation = target - camera.forward() * camera_settings.orbit_distance;
}

#[derive(Debug, Resource)]
struct CameraSettings {
    orbit_distance: f32,
    pitch_speed: f32,
    pitch_range: Range<f32>,
    yaw_speed: f32,
    zoom_speed: f32,
    zoom_range: Range<f32>,
}

impl Default for CameraSettings {
    fn default() -> Self {
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            orbit_distance: 5.0,
            pitch_speed: 0.003,
            pitch_range: -pitch_limit..pitch_limit,
            yaw_speed: 0.004,
            zoom_speed: 0.05,
            zoom_range: 2.0..10.0,
        }
    }
}

pub trait CameraPlugin {
    fn add_camera_systems(&mut self) -> &mut Self;
}

impl CameraPlugin for App {
    fn add_camera_systems(&mut self) -> &mut Self {
        self.init_resource::<CameraSettings>()
            .add_systems(Startup, setup)
            .add_systems(Update, (orbit, zoom))
    }
}
