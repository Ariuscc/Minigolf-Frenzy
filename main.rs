use bevy::prelude::*;
use bevy::app::AppExit;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Hole;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct CameraController {
    speed: f32,
}

const BALL_SPEED: f32 = 15.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.3, 0.7, 0.3)))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, ball_movement)
        .add_systems(FixedUpdate, check_ball_in_hole)
        .add_systems(FixedUpdate, camera_movement)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ball entity
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::default()),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Ball,
        Velocity(Vec3::ZERO),
    ));

    // Hole entity
    commands.spawn((
        PbrBundle {
        mesh: meshes.add(Circle::new(0.75)),
        material: materials.add(Color::BLACK),
        transform: Transform::from_xyz(5.0, 0.0, 5.0),
            ..default()
        },
        Hole,
    ));

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // Camera
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    }, 
    CameraController { speed:10.0},
));
}


fn ball_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Ball>>,
) {
    let (mut velocity, mut transform) = query.single_mut();

    let mut directionx = 0.0;
    let mut directionz = 0.0;

    // Modify direction based on input
    if keys.pressed(KeyCode::ArrowUp) {
        directionz -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        directionz += 1.0;
    }
    if keys.pressed(KeyCode::ArrowLeft) {
        directionx -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        directionx += 1.0;
    }

    // Calculate velocity
    velocity.0 = Vec3::new(
        directionx * BALL_SPEED * time.delta_seconds(),
        0.0,
        directionz * BALL_SPEED * time.delta_seconds(),
    );

    // Update the ball's position based on velocity
    transform.translation += velocity.0;
}

fn check_ball_in_hole(
    ball_query: Query<&Transform, With<Ball>>,
    hole_query: Query<&Transform, With<Hole>>,
    mut exit: EventWriter<AppExit>,
) {
    let ball_transform = ball_query.single();
    let hole_transform = hole_query.single();

    let distance = ball_transform.translation.distance(hole_transform.translation);

    // Check if the ball is close enough to the hole
    if distance < 0.60 {
        println!("Ball is in the hole!");
        exit.send(AppExit::Success); // Close the game
    }
}

fn camera_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&CameraController, &mut Transform), With<Camera3d>>,
) {
    let (controller, mut transform) = query.single_mut();
    let mut direction = Vec3::ZERO;

    // Horizontal movement (WSAD)
    if keys.pressed(KeyCode::KeyW) {
        direction += *transform.forward();
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= *transform.forward();
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= *transform.right();
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += *transform.right();
    }

    // Vertical movement (Q/E)
    if keys.pressed(KeyCode::KeyQ) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyE) {
        direction.y += 1.0;
    }

    // Normalize direction to avoid diagonal speed boost
    if direction.length() > 0.0 {
        direction = direction.normalize();
    }

    // Apply the movement
    transform.translation += direction * controller.speed * time.delta_seconds();
}

