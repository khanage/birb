use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(game::GamePlugin)
        .add_plugins(physics::PhysicsPlugin)
        .add_plugins(bird::BirdPlugin)
        .add_plugins(input::InputPlugin)
        .add_systems(Update, escape_to_quit);

    app.run();
}

fn escape_to_quit(keys: Res<ButtonInput<KeyCode>>, mut app_exit: EventWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        app_exit.send_default();
    }
}

mod input {
    use bevy::prelude::*;

    pub struct InputPlugin;

    impl Plugin for InputPlugin {
        fn build(&self, app: &mut App) {
            app.add_event::<ButtonPressed>()
                .add_systems(Update, listen_for_input);
        }
    }

    #[derive(Default, Event)]
    pub struct ButtonPressed;

    fn listen_for_input(
        mut event_pressed: EventWriter<ButtonPressed>,
        keyboard: Res<ButtonInput<KeyCode>>,
    ) {
        if keyboard.just_pressed(KeyCode::Space) {
            event_pressed.send_default();
        }
    }
}

mod game {
    use bevy::{
        prelude::*,
        window::{PrimaryWindow, WindowTheme},
    };
    use bevy_rapier2d::prelude::*;

    pub struct GamePlugin;

    impl Plugin for GamePlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Blappy birb".into(),
                    name: Some("blappy_birb.app".into()),
                    window_theme: Some(WindowTheme::Dark),
                    ..default()
                }),
                ..default()
            }))
            .add_systems(Startup, setup_camera)
            .add_systems(Startup, spawn_ground_and_ceiling);
        }
    }

    fn setup_camera(mut commands: Commands) {
        commands.spawn(Camera2d);
    }

    fn spawn_ground_and_ceiling(
        mut commands: Commands,
        window: Query<&Window, With<PrimaryWindow>>,
    ) {
        let window = window.single();
        let (width, height) = (window.resolution.width(), window.resolution.height());

        commands.spawn((
            Collider::cuboid(width, 10.0),
            Transform::from_xyz(-width / 2.0, height / 2.0, 0.0),
            RigidBody::Fixed,
        ));

        commands.spawn((
            Collider::cuboid(width, 10.0),
            Transform::from_xyz(-width / 2.0, -height / 2.0, 0.0),
            RigidBody::Fixed,
        ));
    }
}

mod physics {
    use bevy::prelude::*;
    use bevy_rapier2d::prelude::*;

    pub struct PhysicsPlugin;

    impl Plugin for PhysicsPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
                .add_plugins(RapierDebugRenderPlugin::default());
        }
    }
}

mod bird {
    use bevy::{prelude::*, window::PrimaryWindow};
    use bevy_rapier2d::prelude::*;

    pub struct BirdPlugin;

    impl Plugin for BirdPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Startup, spawn_bird)
                .add_systems(Update, flap_bird)
                .add_systems(Update, bird_falls);
        }
    }

    #[derive(Component)]
    struct BirdMarker;

    fn spawn_bird(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        window: Query<&Window, With<PrimaryWindow>>,
    ) {
        let window = window.single();
        let width = window.resolution.width();

        let spawn_x = 20.0 - (width / 2.0) + 128.0;
        let spawn_y = 128.0;

        commands.spawn((
            BirdMarker,
            Sprite::from_image(asset_server.load("sprites/bevy_bird_dark.png")),
            RigidBody::Dynamic,
            Collider::ball(50.0),
            Transform::from_xyz(spawn_x, spawn_y, 0.0).with_scale(Vec3::new(0.5, 0.5, 0.0)),
            GravityScale(1.4),
            Velocity::default(),
            LockedAxes::ROTATION_LOCKED,
        ));
    }

    fn flap_bird(
        mut bird: Query<&mut Velocity, With<BirdMarker>>,
        mut input_pressed: EventReader<crate::input::ButtonPressed>,
    ) {
        for _ in input_pressed.read() {
            println!("Input pressed");

            let mut bird_velocity = bird.single_mut();
            bird_velocity.linvel.y = 800.0;
        }
    }

    fn bird_falls(time: Res<Time>, mut bird_query: Query<&mut Transform, With<BirdMarker>>) {
        let mut bird_transform = bird_query.single_mut();

        bird_transform.translation.y -= 150. * time.delta_secs();
    }
}
