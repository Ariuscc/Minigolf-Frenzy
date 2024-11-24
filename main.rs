use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::mesh::Indices;

const G: f32 = -9.81;
const BG_COLOR: Color = Color::srgb(0.39, 0.82, 0.34);

fn main() {
    App::new()
    .add_systems(Startup, setup)
    .add_plugins(DefaultPlugins)
    .insert_resource(ClearColor(BG_COLOR))
    .add_systems(Update,
(
            aiming_system,
        ) 
    )
    .add_systems(FixedUpdate,
        (
            obstacle_collision,
            ball_animation_and_movement,
            despawn_trajectory_points,
            winning_condition,
            camera_movement,
        )
        
        .chain(),)
    .run();
}

//komponenty
#[derive(Component)]
struct Hole;

#[derive(Component)]
struct Ball 
{
    force_applied: f32,
    direction: Vec3,
    aiming: bool,
    strokes_counter: u32,
}

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct WoodenObstacle;

#[derive(Component)]
struct Line;


#[derive(Component)]
struct TrajectoryPoint;

#[derive(Component)]
struct MessageText;

#[derive(Component)]
struct CameraController 
{
    velocity: f32,
}

// funkcja inicjalizujaca
fn setup
(
    mut commands: Commands, // spawnowanie obiektow
    mut meshes: ResMut<Assets<Mesh>>, //obiekty z biblioteki silnika
    mut materials: ResMut<Assets<StandardMaterial>>, //kolory, tekstury etc.
    asset_server: Res<AssetServer>, // ladowanie assetow z folderu
) 
{
    //spawny

    // pilka
    let ball_handle = asset_server.load("models/golfball2.glb#Scene0");
    commands.spawn
    (
        ( 
        SceneBundle 
        {
            scene: ball_handle, //wczytanie modelu
            transform: Transform::from_xyz(-12.0, 0.4, 0.0).with_scale(Vec3::splat(0.5)),
            ..default()
        },
        Ball 
        {
            force_applied: 0.0,
            direction: Vec3::X,
            aiming: true,
            strokes_counter: 0,
        },
        Velocity(Vec3::ZERO),
    ));

    // dolek
    commands.spawn
    (
        ( 
        PbrBundle 
        {
            mesh: meshes.add(Circle::new(0.75)),
            material: materials.add(Color::BLACK),
            transform: Transform
            {
                translation: Vec3::new(5.0, 0.0, 5.0),
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2), //obrocenie dolka poziomo, bez tego kolko stoi pionowo
                ..default()
            },
            ..default() 
        },
        Hole,
    ));

    // donutowa przeszkoda (oryginalnie miala wygladac jak pien drzewa dlatego woodenobstacle)

    let obstacle_cords = vec!
    [
        Vec3::new(-15.0, 0.25, -10.0),
        Vec3::new(-12.0, 0.25, 4.0),
        Vec3::new(-10.0, 0.25, 8.0),
        Vec3::new(-3.0, 0.25, -14.0),
        Vec3::new(-7.0, 0.25, -8.0),
        Vec3::new(2.0, 0.25, 4.0),
        Vec3::new(8.0, 0.25, 13.0),
        Vec3::new(12.0, 0.25, 2.0),
        Vec3::new(7.0, 0.25, -8.0),
        Vec3::new(2.0, 0.25, -4.0),
        Vec3::new(-8.0, 0.25, 0.0),

    ];

    for coordinates in obstacle_cords
    {
        commands.spawn
        (
            (
            PbrBundle
            {
                mesh: meshes.add(Circle::new(2.0)),
                material: materials.add(Color::srgb(1.0, 0.7, 0.3)),
                transform: Transform
                {
                    translation: (coordinates),
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                    ..default()
                },
                ..default()
            },
            WoodenObstacle,
        ));
        commands.spawn
        ( 
            PbrBundle
            {
                mesh: meshes.add(Torus::new(1.6,2.0)),
                material: materials.add(Color::srgb(0.7,0.4, 0.1)),
                transform: Transform::from_translation(coordinates),
                ..default()
            }
        );
    }

    // oswietlenie sceny
    commands.spawn
    (PointLightBundle 
        {
        point_light: PointLight 
        {
            shadows_enabled: false,
            intensity: 10_000_0000.0,
            range: 100.0,
            ..default()
        },
        transform: Transform::from_xyz(-10.0, 16.0, 0.0),
        ..default()
    });

    // spawn kamery
    commands.spawn
    (
        ( 
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 10.0, 13.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
        },
        CameraController { velocity: 10.0 },
    ));

    // audio
    commands.spawn(AudioBundle 
    {
        source: asset_server.load("sounds/ambience_wind.ogg"),
        ..default()
    }); 
    

    // pole tekstowe
    commands.spawn
    (TextBundle 
    {
        text: Text::from_section(
            "Minigolf3D",
            TextStyle {
                font: asset_server.load("fonts/TepoLalang.ttf"),
                font_size: 55.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    })
    .insert(MessageText)
    .insert(Transform::from_translation(Vec3::new(9.0, 10.0, 1.0))) 
    .insert(GlobalTransform::default());
}



fn ball_animation_and_movement
(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform, &mut Ball)>,
) 
{
    let ground_y_position: f32 = 0.4;
    let stopping_point: f32 = 0.1;
    // tarcie
    let u: f32 = 0.96;

    let (mut velocity, mut transform, mut ball) = query.single_mut();

    // animacja obrotu pilki

    if velocity.0.length() > stopping_point 
    {
     let rotation = Quat::from_axis_angle(-velocity.0.cross(Vec3::Y).normalize(), velocity.0.length() * time.delta_seconds());

     transform.rotation = rotation * transform.rotation;
    }

    // ruch ukosny, predkosc pionowa: Vy= g*t
    if transform.translation.y > ground_y_position 
    {
        velocity.0.y += G * time.delta_seconds();
    }

    if transform.translation.y < ground_y_position  
    {
        transform.translation.y = ground_y_position;
        velocity.0.y = 0.0;
    }

    
    //zmiana polozenia ze wzoru deltaX = V*t
    transform.translation += velocity.0 * time.delta_seconds();

    velocity.0 *= u;

    //uderzanie zmienia predkosc
   if !ball.aiming 
   {
       velocity.0 = 8.0  * ball.force_applied * ball.direction;
       ball.aiming = true;
       ball.force_applied = 0.0;
   }
    
    // zatrzymywanie pilki zeby nie leciala w nieskonczonosc
    if velocity.0.length() < stopping_point
    {
         velocity.0 = Vec3::ZERO;
    }

}

fn aiming_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Ball, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    line_query: Query<Entity, With<Line>>,
    mut text_query: Query<(&mut Text, &MessageText)>,
    asset_server: Res<AssetServer>,

) {
    let (mut ball, transform) = query.single_mut();
    
    if ball.aiming 
    {

        //usuwanie starych linii celowania
        for line_entity in line_query.iter() 
        {
            commands.entity(line_entity).despawn();
        }

        //celowanie w roznych kierunkach
        if keys.pressed(KeyCode::ArrowLeft) 
        {
            let rotation = Quat::from_rotation_y(0.03);
            ball.direction = rotation * ball.direction;
        }
        
        if keys.pressed(KeyCode::ArrowRight) 
        {
            let rotation = Quat::from_rotation_y(-0.03);
            ball.direction = rotation * ball.direction;
        }

        if keys.pressed(KeyCode::ShiftLeft) 
        {
            let rotation = Quat::from_rotation_x(0.03);
            ball.direction = rotation * ball.direction;
        }
        
        if keys.pressed(KeyCode::ShiftRight) 
        {
            let rotation = Quat::from_rotation_x(-0.03);
            ball.direction = rotation * ball.direction;
        }

        //moc uderzenia
        if keys.pressed(KeyCode::ArrowUp) 
        {
            ball.force_applied = (ball.force_applied + 0.1).min(8.0);
        }
        
        if keys.pressed(KeyCode::ArrowDown) 
        {
            ball.force_applied = (ball.force_applied - 0.1).max(0.0);
        }

        //spawn linii celowania
        aiming_line(&mut meshes, transform.translation, transform.translation + ball.direction * ball.force_applied, &mut commands);

        //uderzanie pilki i licznik uderzen
        if keys.just_pressed(KeyCode::Space) 
        {
            ball.aiming = false;
            ball.strokes_counter +=1;

            commands.spawn(AudioBundle 
            {
                source: asset_server.load("sounds/club_hit.ogg"),
                ..default()
            });
        }

        let mut text = text_query.single_mut();
        text.0.sections[0].value= format! ("Number of strokes: {}", ball.strokes_counter);

    }
}

fn aiming_line
(
    meshes: &mut ResMut<Assets<Mesh>>, 
    start: Vec3, 
    end: Vec3, 
    commands: &mut Commands,
) 
{
    let mut mesh = Mesh::new(PrimitiveTopology::LineList,  default() );
    let vertices = vec![start, end];
    let indices = Indices::U32(vec![0, 1]);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(indices);

    commands.spawn
    ((
        PbrBundle 
        {
            mesh: meshes.add(mesh),
            material: Default::default(),
            ..default()
        }, //komponent do despawnowania starych linii
        Line, 
    ));
}

fn mark_trajectory_points
(
    meshes: &mut ResMut<Assets<Mesh>>, 
    position: Vec3, 
    commands: &mut Commands,
) 
{
    
    commands.spawn
    ((
        PbrBundle 
        {
            mesh: meshes.add(Sphere::new(0.05)),
            transform: Transform::from_translation(position),
            ..default()
        },
        TrajectoryPoint,
    ));
}

fn despawn_trajectory_points
(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    query: Query<Entity, With<TrajectoryPoint>>,
) 
{
    if keys.pressed(KeyCode::Backspace)
    {
        for entity in query.iter() 
        {
            commands.entity(entity).despawn();
        }
    }
}



fn winning_condition
(
    ball_query: Query<&Transform, With<Ball>>,
    hole_query: Query<&Transform, With<Hole>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut exit: EventWriter<AppExit>,
    mut previous_cords: Local<Vec3>,
    
    
) 
{
    use std::{thread, time};
    let ball_transform = ball_query.single();
    let hole_transform = hole_query.single();
    
    
    let wait = time::Duration::from_millis(400);    
    let trajectory_distance = ball_transform.translation.distance(*previous_cords); 
    let distance = ball_transform.translation.distance(hole_transform.translation);
    
    if trajectory_distance > 0.5 
    {
        mark_trajectory_points(&mut meshes, ball_transform.translation, &mut commands);

        *previous_cords = ball_transform.translation;
    }


    if distance < 0.55 
    {
        println!("You won!");

        commands.spawn
        (
            AudioBundle 
        {
            source: asset_server.load("sounds/fanfare.ogg"),
            ..default()
        });

        thread::sleep(wait);

        exit.send(AppExit::Success);
    }

}

fn obstacle_collision 
(
    mut ball_query: Query<(&Transform, &mut Velocity), With<Ball>>,
    woodenobstacle_query: Query<&Transform, With<WoodenObstacle>>,
)
{
    let (ball_transform, mut velocity) = ball_query.single_mut();

    for woodenobstacle_transform in woodenobstacle_query.iter()
    {
        let distance = ball_transform.translation.distance(woodenobstacle_transform.translation);

        //wzor z forum : https://www.physicsforums.com/threads/general-formula-for-reflection-direction.426594/
        // The general formula for reflection direction is: R = I - 2(N · I)N, where R is the reflected ray, I is the incident ray, and N is the normal vector of the surface.
        // can be simplified to: R = I - 2(NI)N, where NI is the dot product of the normal vector and the incident ray.

        if distance < 2.7 
        {
            // wektor normalny powierzchni (wektor od przeszkody do piłki)
            let normal = (ball_transform.translation - woodenobstacle_transform.translation).normalize();

            // predkosc z wzoru powyzej
            velocity.0 = velocity.0 - 2.0 * velocity.0.dot(normal) * normal;

        }
    }
}

fn camera_movement
(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&CameraController, &mut Transform), With<Camera3d>>,
    mut exit: EventWriter<AppExit>,
) 
{
    let (controller, mut transform) = query.single_mut();
    let mut direction = Vec3::ZERO;
    
    if keys.pressed(KeyCode::Escape)
    {
        exit.send(AppExit::Success);
    }
    
    if keys.pressed(KeyCode::KeyA) 
    {
        direction.x -= 1.0;
    }
    
    if keys.pressed(KeyCode::KeyD) 
    {
        direction.x += 1.0;
    }
    
    if keys.pressed(KeyCode::ControlLeft) 
    {
        direction.y -= 1.0;
    }
    
    if keys.pressed(KeyCode::ControlRight) 
    {
        direction.y += 1.0;
    }
    
    if keys.pressed(KeyCode::KeyQ)
    {
        direction.z -= 1.0;
    }
    
    if keys.pressed(KeyCode::KeyE) 
    {
        direction.z += 1.0;
    }
    
    if direction.length() > 0.0
    {
        direction = direction.normalize();
    }
    
    //przyblizanie i oddalanie
    if keys.pressed(KeyCode::KeyW) 
    {
        direction += *transform.forward();
    }
    
    if keys.pressed(KeyCode::KeyS) 
    {
        direction -= *transform.forward();
    }

    transform.translation += direction * controller.velocity * time.delta_seconds();
}




