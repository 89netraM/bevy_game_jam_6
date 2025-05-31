use std::time::Duration;

use bevy::{platform::collections::HashMap, prelude::*, time::common_conditions::on_timer};

#[derive(Component)]
struct Pixel {
    pub material: EarthMaterial,
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            material: EarthMaterial::Dirt,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum EarthMaterial {
    Dirt,
    Grass,
    Water,
}

#[derive(Resource, Default)]
struct ContentMaterials {
    dirt: Handle<StandardMaterial>,
    grass: Handle<StandardMaterial>,
    water: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut content_materials: ResMut<ContentMaterials>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sphere_mesh = meshes.add(Sphere::default().mesh().ico(2).unwrap());
    content_materials.dirt = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.55, 0.44, 0.39),
        ..default()
    });
    content_materials.grass = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.0, 1.0, 0.0),
        ..default()
    });
    content_materials.water = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.25, 0.9, 1.0),
        ..default()
    });

    const SPHERE_COUNT: usize = 2500;
    for i in 0..SPHERE_COUNT {
        let y = 1.0 - (i as f32 / SPHERE_COUNT as f32) * 2.0;
        let radius = f32::sqrt(1.0 - y * y);
        let theta = 1.618_034 * (i as f32);
        let x = f32::cos(theta) * radius;
        let z = f32::sin(theta) * radius;
        commands
            .spawn((
                Pixel::default(),
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(content_materials.dirt.clone()),
                Transform::from_xyz(x, y, z).with_scale(Vec3::splat(0.1)),
            ))
            .observe(preview_next_material)
            .observe(restore_current_material)
            .observe(set_next_material);
    }
}

fn preview_next_material(
    trigger: Trigger<Pointer<Over>>,
    content_materials: Res<ContentMaterials>,
    mut query: Query<(&mut MeshMaterial3d<StandardMaterial>, &Pixel)>,
) {
    if let Ok((mut material, pixel)) = query.get_mut(trigger.target()) {
        material.0 = match pixel.material {
            EarthMaterial::Dirt => content_materials.grass.clone(),
            EarthMaterial::Grass => content_materials.water.clone(),
            EarthMaterial::Water => content_materials.dirt.clone(),
        };
    }
}
fn restore_current_material(
    trigger: Trigger<Pointer<Out>>,
    content_materials: Res<ContentMaterials>,
    mut query: Query<(&mut MeshMaterial3d<StandardMaterial>, &Pixel)>,
) {
    if let Ok((mut material, pixel)) = query.get_mut(trigger.target()) {
        material.0 = match pixel.material {
            EarthMaterial::Dirt => content_materials.dirt.clone(),
            EarthMaterial::Grass => content_materials.grass.clone(),
            EarthMaterial::Water => content_materials.water.clone(),
        };
    }
}
fn set_next_material(
    trigger: Trigger<Pointer<Click>>,
    content_materials: Res<ContentMaterials>,
    mut query: Query<(&mut MeshMaterial3d<StandardMaterial>, &mut Pixel)>,
) {
    if let Ok((mut material, mut pixel)) = query.get_mut(trigger.target()) {
        (material.0, pixel.material) = match pixel.material {
            EarthMaterial::Dirt => (content_materials.grass.clone(), EarthMaterial::Grass),
            EarthMaterial::Grass => (content_materials.water.clone(), EarthMaterial::Water),
            EarthMaterial::Water => (content_materials.dirt.clone(), EarthMaterial::Dirt),
        };
    }
}

fn tick(
    content_materials: Res<ContentMaterials>,
    mut param_set: ParamSet<(
        Query<(&mut MeshMaterial3d<StandardMaterial>, &mut Pixel, Entity)>,
        Query<(&Transform, &Pixel, Entity)>,
    )>,
) {
    let closest_map = calculate_five_closest_map(
        param_set
            .p1()
            .iter()
            .map(|(transform, pixel, id)| (transform.translation, pixel.material, id))
            .collect(),
    );
    for (mut material, mut pixel, id) in param_set.p0().iter_mut() {
        let five_closest = &closest_map[&id];
        let water_count = five_closest
            .iter()
            .filter(|(content, _)| *content == EarthMaterial::Water)
            .count();
        let grass_count = five_closest
            .iter()
            .filter(|(content, _)| *content == EarthMaterial::Grass)
            .count();
        let dirt_count = five_closest
            .iter()
            .filter(|(content, _)| *content == EarthMaterial::Dirt)
            .count();
        if pixel.material == EarthMaterial::Dirt {
            if grass_count >= 1 {
                material.0 = content_materials.grass.clone();
                pixel.material = EarthMaterial::Grass;
            } else if water_count >= 2 {
                material.0 = content_materials.water.clone();
                pixel.material = EarthMaterial::Water;
            }
        } else if pixel.material == EarthMaterial::Grass {
            if water_count >= 2 {
                material.0 = content_materials.water.clone();
                pixel.material = EarthMaterial::Water;
            } else if dirt_count >= 4 {
                material.0 = content_materials.dirt.clone();
                pixel.material = EarthMaterial::Dirt;
            }
        } else if pixel.material == EarthMaterial::Water && dirt_count >= 3 {
            material.0 = content_materials.dirt.clone();
            pixel.material = EarthMaterial::Dirt;
        }
    }
}

fn calculate_five_closest_map(
    pixels: Vec<(Vec3, EarthMaterial, Entity)>,
) -> HashMap<Entity, Vec<(EarthMaterial, f32)>> {
    let mut closest_map = HashMap::new();
    for i in 0..pixels.len() {
        let (position, content, id) = &pixels[i];
        for (other_position, other_content, other_id) in pixels.iter().skip(i + 1) {
            let distance = position.distance_squared(*other_position);
            insert_closest(
                closest_map.entry(*id).or_insert_with(Vec::new),
                *other_content,
                distance,
            );
            insert_closest(
                closest_map.entry(*other_id).or_insert_with(Vec::new),
                *content,
                distance,
            );
        }
    }
    return closest_map;

    fn insert_closest(
        closest: &mut Vec<(EarthMaterial, f32)>,
        content: EarthMaterial,
        distance: f32,
    ) {
        if closest.len() < 5 {
            closest.push((content, distance));
        } else if let Some(max_distance) = closest
            .iter()
            .map(|(_, d)| d)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
        {
            if distance < *max_distance {
                if let Some(index) = closest.iter().position(|(_, d)| d == max_distance) {
                    closest[index] = (content, distance);
                }
            }
        }
    }
}

#[derive(Resource)]
struct AutomataState {
    playing: bool,
}

impl Default for AutomataState {
    fn default() -> Self {
        Self { playing: true }
    }
}

impl AutomataState {
    fn is_playing(automata_state: Res<Self>) -> bool {
        automata_state.playing
    }
}

#[derive(Component)]
struct AutomataIsPlayingText;

fn setup_automata_state(mut commands: Commands) {
    commands
        .spawn((
            Text::new("Press Space to toggle automata state: "),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                bottom: Val::Px(10.0),
                ..default()
            },
        ))
        .with_child((
            TextSpan::new("Playing"),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            AutomataIsPlayingText,
        ));
}

fn update_automata_state(
    mut automata_state: ResMut<AutomataState>,
    input: Res<ButtonInput<KeyCode>>,
    mut text: Single<&mut TextSpan, With<AutomataIsPlayingText>>,
) {
    if input.just_pressed(KeyCode::Space) {
        automata_state.playing = !automata_state.playing;
        text.0 = if automata_state.playing {
            "Playing".to_string()
        } else {
            "Paused".to_string()
        };
    }
}

fn setup_reset(mut commands: Commands) {
    commands.spawn((
        Text::new("Press R to reset"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            bottom: Val::Px(10.0),
            ..default()
        },
    ));
}
fn reset(
    content_materials: Res<ContentMaterials>,
    mut query: Query<(&mut Pixel, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    for (mut pixel, mut material) in query.iter_mut() {
        pixel.material = EarthMaterial::Dirt;
        material.0 = content_materials.dirt.clone();
    }
}
fn reset_button_is_pressed(input: Res<ButtonInput<KeyCode>>) -> bool {
    input.just_pressed(KeyCode::KeyR)
}

pub trait PixelsSystems {
    fn add_pixels_systems(&mut self) -> &mut Self;
}

impl PixelsSystems for App {
    fn add_pixels_systems(&mut self) -> &mut Self {
        self.init_resource::<ContentMaterials>()
            .init_resource::<AutomataState>()
            .add_systems(Startup, (setup, setup_automata_state, setup_reset))
            .add_systems(
                Update,
                (update_automata_state, reset.run_if(reset_button_is_pressed)),
            )
            .add_systems(
                FixedUpdate,
                tick.run_if(AutomataState::is_playing)
                    .run_if(on_timer(Duration::from_secs(1))),
            )
    }
}
