
use std::{collections::VecDeque, f64::consts::PI, time::Duration,};

use bevy::prelude::*;

use rand::{distributions::Standard, prelude::*};

const GRAV: f64 = 9.8;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (draw_pendulum, handle_keypress, draw_trails))
        .add_systems(FixedUpdate, physics_sim)
        .init_resource::<DrawTrails>()
        .init_resource::<DrawPends>()
        .init_resource::<Damping>()
        .insert_resource(TrailTime{timer: Timer::new(Duration::from_secs_f32(0.06), TimerMode::Repeating)})
        .run();
}

#[derive(Resource, Default)]
struct DrawTrails(bool);

#[derive(Resource, Default)]
struct DrawPends(bool);

#[derive(Resource, Default)]
struct Damping(bool);

#[derive(Component)]
struct DoublePend;

#[derive(Component)]
struct InnerPend {
    length: f64,
    angle: f64,
    velocity: f64,
    acceleration: f64,
    startpos: Vec2,
    endpos: Vec2,
    lastpos: VecDeque<Vec2>,
}

#[derive(Component)]
struct OuterPend {
    length: f64,
    angle: f64,
    velocity: f64,
    acceleration: f64,
    startpos: Vec2,
    endpos: Vec2,
    lastpos: VecDeque<Vec2>,
}

#[derive(Resource)]
struct TrailTime {
    timer: Timer,
}

#[derive(Component)]
struct PendTraits {
    mass: f64,
    colour: Srgba,
}
impl Default for PendTraits {
    fn default() -> Self {
        let mut rng = thread_rng();
        let rand_color: Vec<f32> = (&mut rng).sample_iter(Standard).take(16).collect();
        Self {
            mass: 1.0,
            colour: Srgba::rgb(rand_color[0], rand_color[1], rand_color[2]),
        }
    }
}

#[derive(Bundle)]
struct PendBundle {
    outerpend: OuterPend,
    innerpend: InnerPend,
    pendtraits: PendTraits,
    pend: DoublePend,
}


fn new_pend() -> PendBundle {
    PendBundle {
        outerpend: OuterPend { 
            length: 200.0, 
            angle: 2.0*PI as  f64 * random::<f64>(), 
            velocity: 0.0, 
            acceleration: 0.0, 
            startpos: Vec2::ZERO, 
            endpos: Vec2::ZERO, 
            lastpos: VecDeque::new(),
        },
        innerpend: InnerPend { 
            length: 200.0, 
            angle: 2.0*PI as  f64 * random::<f64>(), 
            velocity: 0.0, 
            acceleration: 0.0, 
            startpos: Vec2::ZERO, 
            endpos: Vec2::ZERO, 
            lastpos: VecDeque::new(),
        },
        pendtraits: PendTraits { mass: 1.0, colour: Srgba { red: random(), green: random(), blue: random(), alpha: 1.0 } },
        pend: DoublePend,
    }
}

fn setup(mut commands: Commands) {
    // spawn camera
    commands.spawn(Camera2dBundle::default());

    // setup settings
    commands.insert_resource(DrawTrails(true));
}


fn physics_sim(time: Res<Time>, damping: Res<Damping>, mut q: Query<(&mut InnerPend, &mut OuterPend, &PendTraits), With<DoublePend>>) {
    for (mut innerpend, mut outerpend, pendtraits) in &mut q {
        // calc acceleration for  innerpend
        let m1 = pendtraits.mass;
        let m2 = pendtraits.mass;
        let a1 = innerpend.angle;
        let v1 = innerpend.velocity;
        let l1 = 10.;
        let a2 = outerpend.angle;
        let v2 = outerpend.velocity;
        let l2 = 10.;


        innerpend.acceleration = 
        (-GRAV * (2. * m1 + m2) * a1.sin() - m2 * GRAV * (a1 - 2.*a2).sin() - (2.0 * (a1 - a2).sin() * m2) * (v2.powi(2) * l2 + v1.powi(2) * l1 * (a1 - a2).cos())) /
        (l1 *(2.*m1 + m2 - m2 * (2. * a1 - 2. * a2).cos()));

        if damping.0 { innerpend.acceleration -= 0.1 * v1; }

        outerpend.acceleration = 
        (2. * (a1 - a2).sin() * (v1.powi(2) * l1 * (m1 + m2) + GRAV * (m1 + m2) * a1.cos() + v2.powi(2) * l2 * m2 * (a1 - a2).cos())) /
        (l2 * (2.*m1 + m2 - m2 * (2. * a1 - 2. * a2).cos()));

        if damping.0 { outerpend.acceleration -= 0.1 * v1; }
    
        // calc velocity 
        innerpend.velocity += innerpend.acceleration * time.delta_seconds() as f64;
        // calc angle pos
        innerpend.angle += innerpend.velocity * time.delta_seconds() as f64;
        // calc position as vec
        innerpend.endpos = Vec2::from_angle(innerpend.angle as f32 - (PI / 2.0) as f32) * innerpend.length as f32;

        
        // calc velocity 
        outerpend.velocity += outerpend.acceleration * time.delta_seconds() as f64;
        // calc angle pos
        outerpend.angle += outerpend.velocity * time.delta_seconds() as f64;
        // calc position as vec
        outerpend.startpos = innerpend.endpos;
        outerpend.endpos = innerpend.endpos + Vec2::from_angle(outerpend.angle as f32 - (PI / 2.0) as f32) * outerpend.length as f32;
        // dbg!(innerpend.endpos, outerpend.endpos);
    }
}

fn draw_pendulum(
    mut gizmos: Gizmos, q: Query<(&InnerPend, &OuterPend, &PendTraits), With<DoublePend>>, draw_pends: Res<DrawPends>
) {
    if draw_pends.0 {
        for (outerpend, innerpend, pendtraits) in &q {     
            gizmos.line_2d(outerpend.startpos, outerpend.endpos, pendtraits.colour);
            gizmos.line_2d(innerpend.startpos, innerpend.endpos, pendtraits.colour);

            // gizmos.circle_2d(pend.pos, 20., pend.colour).resolution(32);
            // dbg!(outerpend.endpos);
        }
    }
}

fn draw_trails(
    mut gizmos: Gizmos, mut q: Query<(&mut OuterPend, &PendTraits), With<DoublePend>>, draw_trails: Res<DrawTrails>, mut trailtimer: ResMut<TrailTime>, time: Res<Time>,  
) {
    if draw_trails.0 {
        trailtimer.timer.tick(time.delta());
        for (mut outerpend, pendtraits) in &mut q {
            if outerpend.lastpos.len() == 100 { outerpend.lastpos.pop_front(); }
            if trailtimer.timer.finished() {
                let current_pendulum_pos = outerpend.endpos.clone();
                outerpend.lastpos.push_back(current_pendulum_pos);
            }
            gizmos.linestrip_2d(outerpend.lastpos.clone(), pendtraits.colour);
    
        }
    }
}



// Keyboard handling
fn handle_keypress(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    samples: Query<Entity, With<DoublePend>>,
    mut draw_trails: ResMut<DrawTrails>,
    mut draw_pends: ResMut<DrawPends>,
    mut damping: ResMut<Damping>,
) {
    // R => restart
    if keyboard.just_pressed(KeyCode::KeyR) {
        for entity in &samples {
            commands.entity(entity).despawn();
        }
    }

    // S => sample once
    if keyboard.just_pressed(KeyCode::KeyS) {
        commands.spawn(new_pend());
        dbg!()
    }

    // w => generate many samples
    if keyboard.just_pressed(KeyCode::KeyW) {
        for _ in 0..10 {commands.spawn(new_pend());}  

    }

    // A => draw trails
    if keyboard.just_pressed(KeyCode::KeyA) {
          draw_trails.0 = !draw_trails.0;
    }

    // p => draw trails
    if keyboard.just_pressed(KeyCode::KeyP) {
          draw_pends.0 = !draw_pends.0;
    }

    // D => damping
    if keyboard.just_pressed(KeyCode::KeyD) {
        damping.0 = !damping.0;
        dbg!(damping.0);
  }
}