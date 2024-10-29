use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::mesh::Indices;

#[derive(Component)]
struct Ball {
    direction: Vec3,
    power: f32,
    aiming: bool,
}


#[derive(Component)]
struct Hole;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct CameraController {
    speed: f32,
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.3, 0.7, 0.3)))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, ball_movement_system)
        .add_systems(FixedUpdate, check_ball_in_hole)
        .add_systems(FixedUpdate, camera_movement)
        .add_systems(FixedUpdate, aiming_system)
        .add_systems(FixedUpdate, update_ball_position)
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
        Ball {
            direction: Vec3::X,  // Początkowy kierunek na osi X
            power: 0.0,
            aiming: true,
        },
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


fn ball_movement_system(
    mut query: Query<(&mut Ball, &mut Velocity)>,

) {
    let (mut ball, mut velocity) = query.single_mut();



    // Logowanie, aby zobaczyć, czy piłka przestaje celować i jest "uderzana"
    if !ball.aiming {
        println!("Piłka jest uderzana! Kierunek: {:?}, Moc: {}", ball.direction, ball.power);
        
        velocity.0 = ball.direction * ball.power * 10.0; // Skaluje moc na prędkość
        ball.aiming = true; // Przygotuj do kolejnego celowania
        ball.power = 0.0; // Zresetuj moc

    }
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

fn aiming_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Ball, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,

    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,

) {
    let (mut ball, transform) = query.single_mut();
    
    if ball.aiming {
        // Zmiana kierunku przy użyciu strzałek
        if keys.pressed(KeyCode::ArrowLeft) {
            let rotation = Quat::from_rotation_y(0.05); // Obracamy o mały kąt
            ball.direction = rotation * ball.direction;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            let rotation = Quat::from_rotation_y(-0.05);
            ball.direction = rotation * ball.direction;
        }

        // Zmiana mocy uderzenia strzałkami w górę i dół
        if keys.pressed(KeyCode::ArrowUp) {
            ball.power = (ball.power + 0.1).min(10.0); // Maksymalna moc
        }
        if keys.pressed(KeyCode::ArrowDown) {
            ball.power = (ball.power - 0.1).max(0.0); // Minimalna moc
        }

        // Rysowanie linii kierunku
        draw_line(
            &mut commands,
            &mut meshes,
            &mut materials,
            transform.translation,
            transform.translation + ball.direction * ball.power,
        );

        // Uderzenie piłki (zatwierdzenie kierunku i mocy)
        if keys.just_pressed(KeyCode::Space) {
            ball.aiming = false; // Przestajemy celować
        }
    }
}

fn draw_line(    
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    start: Vec3,
    end: Vec3,) {
    let mut mesh = Mesh::new(PrimitiveTopology::LineList, default());
    let vertices = vec![start, end];
    let indices = Indices::U32(vec![0, 1]);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(indices);
    
    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE, // Dla lepszej widoczności ustawiamy linię na biały kolor
            emissive: Color::WHITE.into(),   // Biała emisja, aby linia była jaśniejsza
            ..default()
        }),
        ..default()
    });
}

fn update_ball_position(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Ball>>,
) {
    let friction: f32 = 0.95; // Współczynnik tarcia - im mniejszy, tym większe tarcie
    let stop_threshold: f32 = 0.1; // Próg, poniżej którego piłka uznawana jest za zatrzymaną

    let (mut velocity, mut transform) = query.single_mut();
    
    // Zaktualizuj pozycję na podstawie prędkości
    transform.translation += velocity.0 * time.delta_seconds();

    // Redukcja prędkości w zależności od współczynnika tarcia
    velocity.0 *= friction;

    // Jeśli prędkość jest bardzo mała, zatrzymaj piłkę
    if velocity.0.length() < stop_threshold {
        velocity.0 = Vec3::ZERO;
        println!("Piłka się zatrzymała.");
    }
}
