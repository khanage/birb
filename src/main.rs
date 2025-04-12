use bevy::prelude::*;

const BIRB_X: f32 = 80.0;

fn main() {
    let mut application = App::new();

    application
        .add_plugins(game::GamePlugin)
        .add_plugins(physics::PhysicsPlugin)
        .add_plugins(bird::BirdPlugin)
        .add_plugins(input::InputPlugin)
        .add_plugins(obstacles::ObstaclePlugin)
        .add_systems(Update, escape_to_quit);

    application.run();
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
        fn build(&self, application: &mut App) {
            application
                .add_event::<ButtonPressed>()
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
        fn build(&self, application: &mut App) {
            application
                .add_plugins(DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Blappy birb".into(),
                        name: Some("blappy_birb.app".into()),
                        window_theme: Some(WindowTheme::Dark),
                        // This breaks on WSL for some reason
                        // resolution: WindowResolution::new(640.0, 1136.0),
                        ..default()
                    }),
                    ..default()
                }))
                .init_resource::<Score>()
                .add_systems(Startup, setup_camera)
                .add_systems(Startup, spawn_ground_and_ceiling)
                .add_systems(Startup, spawn_ui)
                .add_systems(Update, detect_collisions)
                .add_systems(Update, update_score)
                .add_systems(Update, player_scored);
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

    #[derive(Default, Resource)]
    pub struct Score {
        score: usize,
    }

    impl Score {
        pub fn passed_obstactle(&mut self) {
            self.score += 100;
        }
    }

    #[derive(Default, Component)]
    struct ScoreMarker;

    fn spawn_ui(mut commands: Commands) {
        commands.spawn((
            ScoreMarker,
            Text::new("0"),
            TextLayout::new_with_justify(JustifyText::Right),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(15.0),
                right: Val::Px(15.0),
                ..default()
            },
        ));
    }

    fn update_score(score: Res<Score>, mut score_display: Query<&mut Text, With<ScoreMarker>>) {
        let mut score_display = score_display.single_mut();

        score_display.0 = format!("{}", score.score);
    }

    fn player_scored(
        mut passed_obstacle: EventReader<crate::obstacles::PlayerPassedObstacle>,
        mut score: ResMut<Score>,
    ) {
        for _ in passed_obstacle.read() {
            score.passed_obstactle();
        }
    }

    fn detect_collisions(mut collision_events: EventReader<CollisionEvent>) {
        for collision in collision_events.read() {
            println!("Collided: {collision:#?}");
        }
    }
}

mod physics {
    use bevy::prelude::*;
    use bevy_rapier2d::prelude::*;

    pub struct PhysicsPlugin;

    impl Plugin for PhysicsPlugin {
        fn build(&self, application: &mut App) {
            application
                .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
                .add_plugins(RapierDebugRenderPlugin::default());
        }
    }
}

mod bird {
    use bevy::prelude::*;
    use bevy_rapier2d::prelude::*;

    use crate::BIRB_X;

    pub struct BirdPlugin;

    impl Plugin for BirdPlugin {
        fn build(&self, application: &mut App) {
            application
                .add_systems(Startup, spawn_bird)
                .add_systems(Update, flap_bird);
        }
    }

    #[derive(Component)]
    struct BirdMarker;

    fn spawn_bird(mut commands: Commands, asset_server: Res<AssetServer>) {
        let spawn_y = 128.0;

        commands.spawn((
            BirdMarker,
            Sprite::from_image(asset_server.load("sprites/bevy_bird_dark.png")),
            RigidBody::Dynamic,
            Collider::ball(50.0),
            ActiveEvents::all(),
            Transform::from_xyz(BIRB_X, spawn_y, 0.0).with_scale(Vec3::new(0.25, 0.25, 0.0)),
            GravityScale(1.4),
            Velocity::default(),
            LockedAxes::ROTATION_LOCKED,
        ));
    }

    fn flap_bird(
        mut bird: Query<&mut Velocity, With<BirdMarker>>,
        mut input_pressed: EventReader<crate::input::ButtonPressed>,
    ) {
        const JUMP_VELOCITY: f32 = 600.0;

        for _ in input_pressed.read() {
            let mut bird_velocity = bird.single_mut();
            if bird_velocity.linvel.y <= JUMP_VELOCITY / 2.0 {
                bird_velocity.linvel.y = JUMP_VELOCITY;
            }
        }
    }
}

mod obstacles {
    use bevy::{prelude::*, window::PrimaryWindow};
    use bevy_rapier2d::prelude::*;

    use crate::BIRB_X;

    pub const TIME_BETWEEN_SPAWN: f32 = 2.0;

    pub const OBSTACLE_WIDTH: f32 = 20.0;

    pub struct ObstaclePlugin;

    impl Plugin for ObstaclePlugin {
        fn build(&self, application: &mut App) {
            application
                .add_event::<PlayerPassedObstacle>()
                .insert_resource(ObstacleSpawnTimer {
                    timer: Timer::from_seconds(TIME_BETWEEN_SPAWN, TimerMode::Repeating),
                })
                .add_systems(Startup, spawn_obstacle)
                .add_systems(Update, (track_obstacle_movement, score_obstacle))
                .add_systems(Update, spawn_obstacle_timed);
        }
    }

    #[derive(Resource)]
    struct ObstacleSpawnTimer {
        timer: Timer,
    }

    #[derive(Default, Component)]
    struct ObstacleMarker;

    #[derive(Default, Component)]
    struct AlreadyScoredMarker;

    #[derive(Default, Event)]
    pub struct PlayerPassedObstacle;

    fn track_obstacle_movement(
        mut commands: Commands,
        obstacles: Query<(Entity, &Transform), With<ObstacleMarker>>,
        window: Query<&Window, With<PrimaryWindow>>,
    ) {
        let window = window.single();
        let left_boundary = -(window.size().x / 2.0) - OBSTACLE_WIDTH;

        for (obstacle, transform) in obstacles.iter() {
            if transform.translation.x < left_boundary {
                println!("Should despawn {obstacle}");
                commands.entity(obstacle).despawn_recursive();
            }
        }
    }

    type ObstacleNotScored = (With<ObstacleMarker>, Without<AlreadyScoredMarker>);

    fn score_obstacle(
        mut commands: Commands,
        obstacles: Query<(Entity, &Transform), ObstacleNotScored>,
        mut passed_obstacle: EventWriter<PlayerPassedObstacle>,
    ) {
        for (obstacle, transform) in obstacles.iter() {
            if transform.translation.x < BIRB_X {
                commands.entity(obstacle).insert(AlreadyScoredMarker);
                passed_obstacle.send_default();
            }
        }
    }

    fn spawn_obstacle_timed(
        commands: Commands,
        time: Res<Time>,
        mut obstacle_spawner: ResMut<ObstacleSpawnTimer>,
        window: Query<&Window, With<PrimaryWindow>>,
    ) {
        if obstacle_spawner.timer.tick(time.delta()).just_finished() {
            spawn_obstacle(commands, window);
        }
    }

    fn spawn_obstacle(mut commands: Commands, window: Query<&Window, With<PrimaryWindow>>) {
        println!("Time to spawn");
        let window = window.single();
        let left_boundary = (window.size().x / 2.0) - OBSTACLE_WIDTH;

        commands
            .spawn((
                ObstacleMarker,
                Transform::from_xyz(left_boundary, 100.0, 0.0),
                RigidBody::KinematicVelocityBased,
                Velocity {
                    linvel: Vec2::new(-200.0, 0.0),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Collider::cuboid(OBSTACLE_WIDTH, 400.0),
                    Transform::from_xyz(0.0, 300.0, 0.0),
                    Sensor,
                ));
                parent.spawn((
                    Collider::cuboid(OBSTACLE_WIDTH, 400.0),
                    Transform::from_xyz(0.0, -700.0, 0.0),
                    Sensor,
                ));
            });
    }
}
