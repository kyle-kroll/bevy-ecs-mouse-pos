use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

struct MousePosition {
    x: f32,
    y: f32,
}

pub struct MainCamera;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut map_query: MapQuery,
) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands.spawn_bundle(UiCameraBundle::default());

    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let map_settings = LayerSettings::new(
        UVec2::new(1, 1),
        UVec2::new(5, 5),
        Vec2::new(32.0, 32.0),
        Vec2::new(96.0, 32.0),
    );

    // Layer 0
    let (mut layer_0, layer_0_entity) =
        LayerBuilder::new(&mut commands, map_settings.clone(), 0u16, 0u16);

    // Required to keep track of layers for a map internally.
    map.add_layer(&mut commands, 0u16, layer_0_entity);

    layer_0.set_all(TileBundle::default());

    map_query.build_layer(&mut commands, layer_0, material_handle.clone());

    // Make 2 layers on "top" of the base map.

    let mut new_settings = map_settings.clone();
    new_settings.set_layer_id(1u16);
    let (mut layer_builder, layer_entity) =
        LayerBuilder::new(&mut commands, new_settings, 0u16, 1u16);

    let position = UVec2::new(2, 2);
    // Ignore errors for demo sake.
    let _ = layer_builder.set_tile(
        position,
        TileBundle {
            tile: Tile {
                texture_index: 1,
                ..Default::default()
            },
            ..Default::default()
        },
    );

    map_query.build_layer(&mut commands, layer_builder, material_handle.clone());

    // Required to keep track of layers for a map internally.
    map.add_layer(&mut commands, 0u16, layer_entity);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(GlobalTransform::default());

    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Mouse pos: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.5, 0.5, 1.0),
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(1.0, 0.5, 0.5),
                    },
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    App::build()
        .insert_resource(WindowDescriptor {
            width: 512.0,
            height: 512.0,
            title: String::from("Accessing tiles"),
            ..Default::default()
        })
        .insert_resource(MousePosition { x: 0., y: 0. })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(movement.system())
        .add_system(my_cursor_system.system())
        .run();
}

pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let scale = transform.scale.x;

        if keyboard_input.pressed(KeyCode::A) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::S) {
            direction -= Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::Z) {
            let scale = scale + 0.1;
            transform.scale = Vec3::splat(scale);
        }

        if keyboard_input.pressed(KeyCode::X) {
            let scale = scale - 0.1;
            transform.scale = Vec3::splat(scale);
        }

        if transform.scale.x < 1.0 {
            transform.scale = Vec3::splat(1.0)
        }

        transform.translation += time.delta_seconds() * direction * 500.;
    }
}

fn my_cursor_system(
    wnds: Res<Windows>,
    q_camera: Query<&Transform, With<MainCamera>>,
    buttons: Res<Input<MouseButton>>,
    mut MousePosition: ResMut<MousePosition>,
    mut text_query: Query<&mut Text>,
) {
    // get the primary window
    let wnd = wnds.get_primary().unwrap();

    // check if the cursor is in the primary window
    if let Some(pos) = wnd.cursor_position() {
        // get the size of the window
        let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // the default orthographic projection is in pixels from the center;
        // just undo the translation
        let p = pos - size / 2.0;

        // assuming there is exactly one main camera entity, so this is OK
        let camera_transform = q_camera.single().unwrap();

        // apply the camera transform
        let pos_wld = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);

        // If the user clicks in the window, display the mouse tile position
        if buttons.just_pressed(MouseButton::Left) {
            MousePosition.x = pos_wld.x / 32.;
            MousePosition.y = pos_wld.y / 32.;
            let mut text = text_query.single_mut().unwrap();
            text.sections[0].value = format!(
                "Mouse pos: {}, {}",
                MousePosition.x as i32, MousePosition.y as i32
            );
        }
    }
}
