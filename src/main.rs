use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_rand::prelude::*;
use consts::*;

fn main() {
    let mut application = App::new();
    let seed = 42i64.to_be_bytes();

    application
        .add_plugins(EntropyPlugin::<WyRand>::with_seed(seed))
        .add_plugins(game::GamePlugin)
        .add_plugins(physics::PhysicsPlugin)
        .add_plugins(bird::BirdPlugin)
        .add_plugins(input::InputPlugin)
        .add_plugins(obstacles::ObstaclePlugin)
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
                .load_collection::<AudioAssets>()
                .load_collection::<SpriteAssets>(),
        )
        .enable_state_scoped_entities::<GameState>()
        .add_systems(Update, escape_to_quit);

    #[cfg(feature = "debug")]
    application.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

    application.run();
}

#[derive(AssetCollection, Resource)]
struct SpriteAssets {
    #[asset(path = "sprites/background-day.png")]
    background_day: Handle<Image>,
    #[asset(path = "sprites/birb.png")]
    birb: Handle<Image>,
    #[asset(path = "sprites/pipe-green.png")]
    green_pipe: Handle<Image>,
    #[asset(path = "sprites/base.png")]
    ground: Handle<Image>,
    #[asset(path = "sprites/message.png")]
    start_screen_instructions: Handle<Image>,
    #[asset(path = "sprites/background-night.png")]
    start_screen_background: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
struct AudioAssets {
    #[asset(path = "audio/hit.ogg")]
    hit: Handle<AudioSource>,
}

fn escape_to_quit(keys: Res<ButtonInput<KeyCode>>, mut app_exit: EventWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        app_exit.send_default();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    Menu,
    InGame,
}

mod consts {
    pub const BIRB_X: f32 = 40.0;
    pub const JUMP_VELOCITY: f32 = 600.0;
    pub const TIME_BETWEEN_SPAWN: f32 = 2.0;
    pub const OBSTACLE_WIDTH: f32 = 20.0;
}

mod input {
    use crate::*;

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
        mouse: Res<ButtonInput<MouseButton>>,
        touches: Res<Touches>,
    ) {
        if keyboard.just_pressed(KeyCode::Space)
            || mouse.just_pressed(MouseButton::Left)
            || touches.iter_just_pressed().next().is_some()
        {
            event_pressed.send_default();
        }
    }
}

mod game {
    use crate::*;
    use bevy::{
        audio::Volume,
        window::{PrimaryWindow, WindowTheme},
    };
    use bevy_rapier2d::prelude::*;

    pub struct GamePlugin;

    impl Plugin for GamePlugin {
        fn build(&self, application: &mut App) {
            let default_plugins = DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Blappy birb".into(),
                        name: Some("blappy_birb.app".into()),
                        window_theme: Some(WindowTheme::Dark),
                        // This breaks on WSL for some reason
                        // #[cfg(not(target_os = "linux"))]
                        resolution: bevy::window::WindowResolution::new(640.0, 1136.0),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                });

            application
                .add_plugins(default_plugins)
                .init_resource::<Score>()
                .add_systems(Startup, setup_camera)
                .add_systems(OnEnter(GameState::Loading), spawn_loading_screen)
                .add_systems(OnExit(GameState::Loading), despawn_loading_screen)
                .add_systems(OnEnter(GameState::Menu), spawn_start_menu)
                .add_systems(
                    Update,
                    start_game_on_input.run_if(in_state(GameState::Menu)),
                )
                .add_systems(
                    OnEnter(GameState::InGame),
                    (spawn_ground_and_ceiling, spawn_ui),
                )
                .add_systems(
                    Update,
                    (detect_collisions, update_score, player_scored)
                        .run_if(in_state(GameState::InGame)),
                );
        }
    }

    fn setup_camera(mut commands: Commands) {
        commands.spawn(Camera2d);
    }

    #[derive(Debug, Component)]
    struct LoadingMarker;

    fn spawn_loading_screen(mut commands: Commands) {
        commands.spawn((
            Name::new("Loading UI"),
            LoadingMarker,
            Text::new("Loading..."),
        ));
    }

    fn despawn_loading_screen(mut commands: Commands, query: Query<Entity, With<LoadingMarker>>) {
        let Ok(loading_screen) = query.get_single() else {
            return;
        };

        commands
            .get_entity(loading_screen)
            .expect("Tried to despawn an invalid entity")
            .despawn_recursive();
    }

    fn spawn_start_menu(mut commands: Commands, sprites: Res<SpriteAssets>) {
        let background = Sprite {
            image: sprites.start_screen_background.clone(),
            custom_size: Some(Vec2::new(
            640.0,
            1146.0
            )),
            ..default()
        };

        commands.spawn((
            Name::new("Startup background"),
            background,
            Transform::from_translation(Vec3::new(1.0, 1.0, 0.0)),
            StateScoped(GameState::Menu),
        ));

        let start_screen_instructions = Sprite {
            image: sprites.start_screen_instructions.clone(),
            custom_size: Some(Vec2::new(
                640.0,
                1146.0
            )),
            ..default()
        };

        commands.spawn((
            Name::new("Start game UI"),
            start_screen_instructions,
            StateScoped(GameState::Menu),
        ));
    }

    fn start_game_on_input(
        mut input: EventReader<input::ButtonPressed>,
        mut next_state: ResMut<NextState<GameState>>,
    ) {
        for _ in input.read() {
            next_state.set(GameState::InGame);
        }

        input.clear();
    }

    fn spawn_ground_and_ceiling(
        mut commands: Commands,
        window: Query<&Window, With<PrimaryWindow>>,
        sprites: Res<SpriteAssets>,
    ) {
        let window = window.single();
        let (width, height) = (window.resolution.width(), window.resolution.height());
        let background = Sprite::from_image(sprites.background_day.clone());
        let bottom = Sprite::from_image(sprites.ground.clone());

        commands.spawn((
            Name::new("Background image"),
            background,
            Transform::from_xyz(0.0, 0.0, -2.0).with_scale(Vec3::new(2.25, 2.25, 1.0)),
            StateScoped(GameState::InGame),
        ));

        commands.spawn((
            Name::new("Ground texture"),
            bottom,
            Transform::from_xyz(0.0, -500.0, -1.0).with_scale(Vec3::new(2.25, 2.25, 1.0)),
            StateScoped(GameState::InGame),
        ));

        commands.spawn((
            Name::new("Roof collider"),
            Collider::cuboid(width, 10.0),
            Transform::from_xyz(-width / 2.0, height / 2.0, 0.0),
            RigidBody::Fixed,
            StateScoped(GameState::InGame),
        ));

        commands.spawn((
            Name::new("Roof collider"),
            Collider::cuboid(width, 10.0),
            Transform::from_xyz(-width / 2.0, -height / 2.0, 0.0),
            RigidBody::Fixed,
            StateScoped(GameState::InGame),
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
            Name::new("Score UI"),
            ScoreMarker,
            Text::new("0"),
            TextLayout::new_with_justify(JustifyText::Right),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(15.0),
                right: Val::Px(15.0),
                ..default()
            },
            StateScoped(GameState::InGame),
        ));
    }

    fn update_score(score: Res<Score>, mut score_display: Query<&mut Text, With<ScoreMarker>>) {
        let Ok(mut score_display) = score_display.get_single_mut() else {
            return;
        };

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

    fn detect_collisions(
        mut commands: Commands,
        mut collision_events: EventReader<CollisionEvent>,
        mut next_state: ResMut<NextState<GameState>>,
        audio_asssets: Res<AudioAssets>,
    ) {
        for collision in collision_events.read() {
            let CollisionEvent::Started(_, _, _) = collision else {
                continue;
            };

            info!("Spawning new sound");
            commands.spawn((
                Name::new("Hit effect"),
                AudioPlayer(audio_asssets.hit.clone()),
                PlaybackSettings::DESPAWN.with_volume(Volume::new(0.25)),
            ));

            next_state.set(GameState::Menu);
        }
    }
}

mod physics {
    use crate::*;
    use bevy_rapier2d::prelude::*;

    pub struct PhysicsPlugin;

    impl Plugin for PhysicsPlugin {
        fn build(&self, application: &mut App) {
            application
                .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));

            #[cfg(feature = "debug")]
            application
                .add_plugins(RapierDebugRenderPlugin::default());
        }
    }
}

mod bird {
    use crate::*;
    use bevy_rapier2d::prelude::*;

    pub struct BirdPlugin;

    impl Plugin for BirdPlugin {
        fn build(&self, application: &mut App) {
            application
                .add_systems(OnEnter(GameState::InGame), spawn_bird)
                .add_systems(Update, (flap_bird, animate_bird).run_if(in_state(GameState::InGame)));
        }
    }

    #[derive(Component)]
    struct BirdMarker;

    #[derive(Component)]
    struct AnimationIndices {
        first: usize,
        last: usize,
    }

    #[derive(Component, Deref, DerefMut)]
    struct AnimationTimer(Timer);

    fn spawn_bird(mut commands: Commands, assets: Res<SpriteAssets>, mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>) {
        let spawn_y = 128.0;

        let birb_texture = assets.birb.clone();
        let layout = TextureAtlasLayout::from_grid(UVec2::new(34, 24), 4, 1, None, None);
        let layout = texture_atlas_layouts.add(layout);
        let animation_indices = AnimationIndices { first: 0, last: 3 };
        let sprite = Sprite::from_atlas_image(birb_texture, TextureAtlas { layout, index: animation_indices.first });

        commands.spawn((
            Name::new("Birb"),
            BirdMarker,
            sprite,
            animation_indices,
            AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
            RigidBody::Dynamic,
            Collider::ball(15.0),
            ActiveEvents::all(),
            Transform::from_xyz(BIRB_X, spawn_y, 0.0).with_scale(Vec3::new(1.25, 1.25, 0.0)),
            GravityScale(1.4),
            Velocity::default(),
            LockedAxes::ROTATION_LOCKED,
            StateScoped(GameState::InGame),
        ));
    }

    fn animate_bird(time: Res<Time>, mut query: Query<(&AnimationIndices, &mut AnimationTimer,  &mut Sprite)>){
        for (animation_indices, mut animation_timer, mut sprite) in query.iter_mut(){
            animation_timer.tick(time.delta());

            if animation_timer.just_finished() {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = if atlas.index == animation_indices.last {
                        animation_indices.first
                    } else {
                        atlas.index + 1
                    };
                }
            }
        }
    }

    fn flap_bird(
        mut bird: Query<&mut Velocity, With<BirdMarker>>,
        mut input_pressed: EventReader<crate::input::ButtonPressed>,
    ) {
        for _ in input_pressed.read() {
            let mut bird_velocity = bird.single_mut();
            if bird_velocity.linvel.y <= JUMP_VELOCITY / 2.0 {
                bird_velocity.linvel.y = JUMP_VELOCITY;
            }
        }
    }
}

mod obstacles {
    use crate::*;
    use bevy::window::PrimaryWindow;
    use bevy_rapier2d::prelude::*;
    use rand::Rng;

    pub struct ObstaclePlugin;

    impl Plugin for ObstaclePlugin {
        fn build(&self, application: &mut App) {
            application
                .add_event::<PlayerPassedObstacle>()
                .insert_resource(ObstacleSpawnTimer {
                    timer: Timer::from_seconds(TIME_BETWEEN_SPAWN, TimerMode::Repeating),
                })
                .add_systems(OnEnter(GameState::InGame), (spawn_obstacle, reset_timer))
                .add_systems(
                    Update,
                    (
                        track_obstacle_movement,
                        score_obstacle,
                        spawn_obstacle_timed,
                    )
                        .run_if(in_state(GameState::InGame)),
                );
        }
    }

    #[derive(Resource)]
    struct ObstacleSpawnTimer {
        timer: Timer,
    }

    fn reset_timer(mut timer: ResMut<ObstacleSpawnTimer>) {
        timer.timer.reset();
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
        let Ok(window) = window.get_single() else {
            return;
        };

        let left_boundary = -(window.resolution.width() / 2.0) - OBSTACLE_WIDTH;

        for (obstacle, transform) in obstacles.iter() {
            if transform.translation.x < left_boundary {
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
        rng: GlobalEntropy<WyRand>,
        assets: Res<SpriteAssets>,
    ) {
        if obstacle_spawner.timer.tick(time.delta()).just_finished() {
            spawn_obstacle(commands, window, rng, assets);
        }
    }

    fn spawn_obstacle(
        mut commands: Commands,
        window: Query<&Window, With<PrimaryWindow>>,
        mut rng: GlobalEntropy<WyRand>,
        assets: Res<SpriteAssets>,
    ) {
        let window = window.single();
        let left_boundary = (window.size().x / 2.0) + OBSTACLE_WIDTH;
        let height = rng.gen_range(100.0..400.0);

        commands
            .spawn((
                Name::new("Obstacle"),
                ObstacleMarker,
                Transform::from_xyz(left_boundary, height, 0.0),
                RigidBody::KinematicVelocityBased,
                Velocity {
                    linvel: Vec2::new(-200.0, 0.0),
                    ..default()
                },
                Visibility::Visible,
                StateScoped(GameState::InGame),
            ))
            .with_children(|parent| {
                let mut flipped_sprite = Sprite::from_image(assets.green_pipe.clone());
                flipped_sprite.flip_y = true;

                parent.spawn((
                    Name::new("Top pipe"),
                    Collider::cuboid(OBSTACLE_WIDTH, 400.0),
                    flipped_sprite,
                    Transform::from_xyz(0.0, 300.0, 0.0),
                    Sensor,
                ));
                parent.spawn((
                    Name::new("Bottom pipe"),
                    Collider::cuboid(OBSTACLE_WIDTH, 400.0),
                    Sprite::from_image(assets.green_pipe.clone()),
                    Transform::from_xyz(0.0, -700.0, 0.0),
                    Sensor,
                ));
            });
    }
}
