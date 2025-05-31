mod camera;
mod pixels;

use camera::*;
use pixels::*;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_pixels_systems()
        .add_camera_systems()
        .run();
}
