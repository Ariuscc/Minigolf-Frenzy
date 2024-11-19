use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::mesh::Indices;

const BG_COLOR: Color = Color::srgb(0.4, 0.8, 0.3);



fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(ClearColor(BG_COLOR))
    .add_systems(Startup, setup)
    .add_systems(FixedUpdate,
        (
        camera_movement,
        ball_movement_system,
        update_ball_position,
        aiming_system,
        update_message_system,
        check_ball_in_hole,
         )
        .chain(),
         )
    .run();
}


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

#[derive(Component)]
struct Line;

#[derive(Component)]
struct MessageText;

#[derive(Component)]
struct WoodenObstacle;



fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>, // Dodano ładowanie zasobu czcionki
) {
    // Ball entity
    commands.spawn(( 
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.6)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(0.0, 0.4, 0.0),
            ..default()
        },
        Ball {
            direction: Vec3::X,
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

    //Wooden Obstacle entity

    commands.spawn((
        PbrBundle{
            mesh: meshes.add(Cuboid:: new(2.0, 1.0, 3.0)),
            material: materials.add(Color::srgb(0.9,0.7, 0.2)),
            transform: Transform::from_xyz(0.0, 0.25, 5.0),
            ..default()
        },
        WoodenObstacle,
    ));




    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.0,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // Camera
    commands.spawn(( 
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 7.0, 14.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
        },
        CameraController { speed: 10.0 },
    ));

    //Audio
    commands.spawn(AudioBundle {
        source: asset_server.load("sounds/ambience_wind.ogg"),
        ..default()
    }); 
    

    // Text entity
    commands.spawn(TextBundle {
        text: Text::from_section(
            "Press SPACE to hit the ball",
            TextStyle {
                font: asset_server.load("fonts/TepoLalang.ttf"), // Upewnij się, że masz czcionkę
                font_size: 50.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    })
    .insert(MessageText)
    .insert(Transform::from_translation(Vec3::new(0.0, 4.0, 0.0))) // Pozycja tekstu na ekranie
    .insert(GlobalTransform::default());
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

fn update_ball_position(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Ball>>,
) {
    let friction: f32 = 0.96; // Współczynnik tarcia - im mniejszy, tym większe tarcie
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

fn aiming_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Ball, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    line_query: Query<Entity, With<Line>>, // Zapytanie do usunięcia poprzednich linii
) {
    let (mut ball, transform) = query.single_mut();
    
    if ball.aiming {
        // Usuwanie poprzednich linii
        for line_entity in line_query.iter() {
            commands.entity(line_entity).despawn();
        }

        if keys.pressed(KeyCode::ArrowLeft) {
            let rotation = Quat::from_rotation_y(0.05);
            ball.direction = rotation * ball.direction;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            let rotation = Quat::from_rotation_y(-0.05);
            ball.direction = rotation * ball.direction;
        }

        if keys.pressed(KeyCode::ArrowUp) {
            ball.power = (ball.power + 0.1).min(10.0);
        }
        if keys.pressed(KeyCode::ArrowDown) {
            ball.power = (ball.power - 0.1).max(0.0);
        }

        // Rysowanie nowej linii
        draw_line(&mut meshes, transform.translation, transform.translation + ball.direction * ball.power, &mut commands);

        if keys.just_pressed(KeyCode::Space) {
            ball.aiming = false;
        }
    }
}

fn draw_line(
    meshes: &mut ResMut<Assets<Mesh>>, 
    start: Vec3, 
    end: Vec3, 
    commands: &mut Commands,
) {
    let mut mesh = Mesh::new(PrimitiveTopology::LineList,  default() );
    let vertices = vec![start, end];
    let indices = Indices::U32(vec![0, 1]);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(indices);

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: Default::default(), // Tutaj możesz ustawić przezroczystość lub inny wygląd
            ..default()
        },
        Line, // Dodanie komponentu "Line" dla łatwego usuwania później
    ));
}

fn update_message_system(
    query: Query<&Ball>,
    mut text_query: Query<(&mut Text, &MessageText)>,
) {
    let ball = query.single();

    let mut text = text_query.single_mut();
    if ball.aiming {
        text.0.sections[0].value = "Press SPACE to hit the ball".to_string();
    } else {
        text.0.sections[0].value = "Ball is moving...".to_string();
    }
}

fn check_ball_in_hole(
    ball_query: Query<&Transform, With<Ball>>,
    hole_query: Query<&Transform, With<Hole>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut text_query: Query<(&mut Text, &MessageText)>,
    //keys: Res<ButtonInput<KeyCode>>,
    //mut exit: EventWriter<AppExit>,
    
) {
    use std::{thread, time};
    let ball_transform = ball_query.single();
    let hole_transform = hole_query.single();
    let wait = time::Duration::from_millis(500);
    let mut text = text_query.single_mut();
    
    let distance = ball_transform.translation.distance(hole_transform.translation);
    //do naprawienia, gdy pilka jest w dolku to dziwne rzeczy sie dzieja i nie mozna wylaczyc
    
    let mut win_condition = false;

    if distance < 0.55 {
        text.0.sections[0].value = "Ball in the hole!".to_string();
       
        thread::sleep(wait);
        println!("Ball is in the hole!");
        win_condition = true;
    }
    if win_condition == true{
        commands.spawn(AudioBundle {
            source: asset_server.load("sounds/fanfare.ogg"),
            ..default()
        });
        thread::sleep(wait);
        //exit.send(AppExit::Success);
    }

}

fn camera_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&CameraController, &mut Transform), With<Camera3d>>,
    mut exit: EventWriter<AppExit>,
) {
    let (controller, mut transform) = query.single_mut();
    let mut direction = Vec3::ZERO;
    
    if keys.pressed(KeyCode::KeyE)
    {
        exit.send(AppExit::Success); // zrobic wychodzenie pod przycisk E
    }
    
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
    
    if keys.pressed(KeyCode::KeyQ) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyE) {
        direction.y += 1.0;
    }
    
    if direction.length() > 0.0 {
        direction = direction.normalize();
    }
    
    transform.translation += direction * controller.speed * time.delta_seconds();
}
