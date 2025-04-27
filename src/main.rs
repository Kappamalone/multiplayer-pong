use std::time::Duration;

use bevy::{
    color::palettes::css::{BLACK, WHITE},
    prelude::*,
};

const PADDLE_MOVE_SPEED: f32 = 15.;
const PADDLE_WIDTH: f32 = 50.;
const PADDLE_HEIGHT: f32 = 150.;
const BALL_RADIUS: f32 = 15.;
const BALL_SPEED: f32 = 15.;
const FONT_SIZE: f32 = 50.;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Menu,
    InGame,
    PointScored,
    GameOver,
}

#[derive(Resource)]
pub struct Score((usize, usize));

// from the unofficial bevy cheat book
mod cleanup {
    use bevy::prelude::*;
    #[derive(Component)]
    pub struct MenuCleanup;
    #[derive(Component)]
    pub struct InGameCleanup;
    #[derive(Component)]
    pub struct ToMenuCleanup;
}

fn cleanup_system<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for entity in q {
        commands.entity(entity).despawn();
    }
}

mod menu {
    use crate::*;
    pub fn spawn(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        window_query: Query<&Window>,
    ) {
        let window = window_query.iter().nth(0).unwrap();
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        let text_font = TextFont {
            font: font.clone(),
            font_size: FONT_SIZE,
            ..default()
        };

        commands.spawn((
            Text2d::new("Pong"),
            text_font.clone(),
            TextLayout::new_with_justify(JustifyText::Center),
            Transform::from_translation(Vec3::new(0., 0., 0.)),
            cleanup::MenuCleanup,
        ));

        commands.spawn((
            Text2d::new("Press anywhere to start"),
            text_font.clone(),
            TextLayout::new_with_justify(JustifyText::Center),
            Transform::from_translation(Vec3::new(0., -window.height() / 4., 0.)),
            cleanup::MenuCleanup,
        ));
    }
    pub fn handle_input(
        buttons: Res<ButtonInput<MouseButton>>,
        mut next_state: ResMut<NextState<GameState>>,
    ) {
        if buttons.just_pressed(MouseButton::Left) || buttons.just_pressed(MouseButton::Right) {
            next_state.set(GameState::InGame);
        }
    }
}

mod in_game {
    use crate::*;

    enum PaddleSide {
        Left,
        Right,
    }

    #[derive(Component)]
    pub struct Paddle(PaddleSide);

    #[derive(Component)]
    pub struct Ball(Vec3);

    pub fn spawn(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        commands.spawn((
            Paddle(PaddleSide::Left),
            Mesh2d(meshes.add(Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT))),
            MeshMaterial2d(materials.add(Color::from(WHITE))),
            Transform::from_translation(Vec3::new(-600., 0., 0.)),
            cleanup::InGameCleanup,
        ));

        commands.spawn((
            Paddle(PaddleSide::Right),
            Mesh2d(meshes.add(Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT))),
            MeshMaterial2d(materials.add(Color::from(WHITE))),
            Transform::from_translation(Vec3::new(600., 0., 0.)),
            cleanup::InGameCleanup,
        ));

        commands.spawn((
            Ball(Vec3::new(-1., 1., 0.).normalize()),
            Mesh2d(meshes.add(Circle::new(BALL_RADIUS))),
            MeshMaterial2d(materials.add(Color::from(WHITE))),
            Transform::from_translation(Vec3::new(0., 0., 0.)),
            cleanup::InGameCleanup,
        ));
    }

    pub fn handle_input(
        keys: Res<ButtonInput<KeyCode>>,
        mut query: Query<(&Paddle, &mut Transform)>,
        window_query: Query<&Window>,
    ) {
        let window = window_query.iter().nth(0).unwrap();
        for (side, mut transform) in &mut query {
            let mut direction = Vec3::ZERO;

            match side.0 {
                PaddleSide::Left => {
                    if keys.pressed(KeyCode::KeyW) {
                        direction.y += 1.0;
                    }
                    if keys.pressed(KeyCode::KeyS) {
                        direction.y -= 1.0;
                    }
                }
                PaddleSide::Right => {
                    if keys.pressed(KeyCode::ArrowUp) {
                        direction.y += 1.0;
                    }
                    if keys.pressed(KeyCode::ArrowDown) {
                        direction.y -= 1.0;
                    }
                }
            }

            if 0.0 < direction.length() {
                transform.translation += PADDLE_MOVE_SPEED * direction.normalize();
            }

            if transform.translation.y >= window.height() / 2. - PADDLE_HEIGHT / 2. {
                transform.translation.y = window.height() / 2. - PADDLE_HEIGHT / 2.;
            }

            if transform.translation.y <= -window.height() / 2. + PADDLE_HEIGHT / 2. {
                transform.translation.y = -window.height() / 2. + PADDLE_HEIGHT / 2.;
            }
        }
    }

    // need to check for intersections first, then move the ball
    pub fn move_ball(
        mut score: ResMut<Score>,
        mut next_state: ResMut<NextState<GameState>>,
        mut query: Query<(&mut Ball, &mut Transform)>,
        window_query: Query<&Window>,
    ) {
        let window = window_query.iter().nth(0).unwrap();
        let (mut direction, mut transform) = query.iter_mut().nth(0).unwrap();

        if transform.translation.y + BALL_RADIUS >= (window.height() / 2.) {
            direction.0.y = -1.;
        }
        if transform.translation.y - BALL_RADIUS <= -(window.height() / 2.) {
            direction.0.y = 1.;
        }
        if transform.translation.x + BALL_RADIUS >= (window.width() / 2.) {
            next_state.set(GameState::PointScored);
            score.0.0 += 1;
            println!("{} vs {}", score.0.0, score.0.1);
            if score.0.0 == 3 {
                next_state.set(GameState::GameOver);
            }
        }
        if transform.translation.x - BALL_RADIUS <= -(window.width() / 2.) {
            next_state.set(GameState::PointScored);
            score.0.1 += 1;
            println!("{} vs {}", score.0.0, score.0.1);
            if score.0.1 == 3 {
                next_state.set(GameState::GameOver);
            }
        }
        transform.translation += BALL_SPEED * direction.0.normalize();
    }

    pub fn handle_collision(
        paddles_query: Query<(&Paddle, &Transform)>,
        mut ball_query: Query<(&mut Ball, &Transform)>,
    ) {
        let (mut ball_direction, ball_transform) = ball_query.iter_mut().nth(0).unwrap();
        for (paddle_side, paddle_transform) in paddles_query {
            if ball_transform.translation.y + BALL_RADIUS
                <= paddle_transform.translation.y + PADDLE_HEIGHT
                && ball_transform.translation.y - BALL_RADIUS
                    >= paddle_transform.translation.y - PADDLE_HEIGHT
            {
                match paddle_side.0 {
                    PaddleSide::Left => {
                        if ball_transform.translation.x + BALL_RADIUS
                            <= paddle_transform.translation.x + PADDLE_WIDTH
                        {
                            ball_direction.0.x = 1.0;
                        }
                    }
                    PaddleSide::Right => {
                        if ball_transform.translation.x - BALL_RADIUS
                            >= paddle_transform.translation.x - PADDLE_WIDTH
                        {
                            ball_direction.0.x = -1.0;
                        }
                    }
                }
            }
        }
    }
}

mod point_scored {
    use crate::*;

    #[derive(Resource)]
    pub struct PointScoredTimer(Timer);

    pub fn spawn(mut commands: Commands) {
        commands.insert_resource(PointScoredTimer(Timer::from_seconds(1., TimerMode::Once)));
    }

    pub fn wait(
        time: Res<Time>,
        mut timer: ResMut<PointScoredTimer>,
        mut next_state: ResMut<NextState<GameState>>,
    ) {
        if timer.0.tick(time.delta()).just_finished() {
            next_state.set(GameState::InGame);
        }
    }
}

#[derive(Resource)]
struct DebugTimer(Timer);
fn debug(
    time: Res<Time>,
    mut timer: ResMut<DebugTimer>,
    query: Query<(&in_game::Paddle, &Transform)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (_, transform) in query {
            println!("[DEBUG] {:?}", transform.translation);
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(DebugTimer(Timer::new(
            Duration::from_secs(1),
            TimerMode::Repeating,
        )))
        .init_state::<GameState>()
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
            commands.insert_resource(Score((0, 0)));
        })
        .add_systems(OnEnter(GameState::Menu), menu::spawn)
        .add_systems(
            Update,
            (menu::handle_input).run_if(in_state(GameState::Menu)),
        )
        .add_systems(
            OnExit(GameState::Menu),
            cleanup_system::<cleanup::MenuCleanup>,
        )
        .add_systems(OnEnter(GameState::InGame), in_game::spawn)
        .add_systems(
            Update,
            (
                in_game::handle_input,
                in_game::move_ball,
                in_game::handle_collision,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            OnExit(GameState::InGame),
            cleanup_system::<cleanup::InGameCleanup>,
        )
        .add_systems(OnEnter(GameState::PointScored), point_scored::spawn)
        .add_systems(
            Update,
            (point_scored::wait).run_if(in_state(GameState::PointScored)),
        )
        .add_systems(Update, debug)
        .run();
}
