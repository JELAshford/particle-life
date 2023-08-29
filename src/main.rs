use rand::{rngs::ThreadRng, seq::IteratorRandom, Rng};
use raylib::prelude::*;
use rayon::prelude::*;

#[derive(PartialEq, Clone, Copy)]
struct Particle {
    color: usize,
    position: Vector2,
    velocity: Vector2,
}

fn flat_matrix(side_length: usize, rng_obj: &mut ThreadRng) -> Vec<f32> {
    (0..(side_length * side_length))
        .map(|_| (rng_obj.gen::<f32>() - 0.5) * 2.)
        .collect()
}

fn force(r: f32, a: f32, beta: f32) -> f32 {
    if r < beta {
        r / beta - 1.
    } else if (beta < r) && (r < 1.) {
        a * (1. - (2. * r - 1. - beta).abs() / (1. - beta))
    } else {
        0.
    }
}

fn update_population(population: &Vec<Particle>, attractions: &Vec<f32>) -> Vec<Particle> {
    let friction_factor: f32 = 0.5_f32.powf(TIME_STEP / FRICTION_HALF_LIFE);

    // Update velocity
    let new_population: Vec<Particle> = population
        .par_iter()
        .map(|p1| {
            let mut total_force: Vector2 = Vector2::zero();
            for p2 in population {
                if p1 == p2 {
                    continue;
                };
                let distance = p1.position.distance_to(p2.position);
                if (distance > 0.) & (distance < MAX_RADIUS) {
                    let f = force(
                        distance / MAX_RADIUS,
                        attractions[(p1.color * 3) + p2.color],
                        0.3,
                    );
                    total_force += ((p2.position - p1.position) / distance) * f;
                }
            }
            total_force *= MAX_RADIUS;
            let mut new_p = *p1;
            new_p.velocity *= friction_factor;
            new_p.velocity += total_force * TIME_STEP;

            // Push toward centre
            new_p.velocity -= (new_p.position - Vector2 { x: 0.5, y: 0.5 }) / 16.;

            new_p
        })
        .collect();

    // update positions
    new_population
        .iter()
        .map(|p| {
            let mut new_p = *p;
            new_p.position += new_p.velocity * TIME_STEP;
            new_p
        })
        .collect()
}

const WIDTH: i32 = 1000;
const HEIGHT: i32 = 1000;
const MAX_RADIUS: f32 = 0.1;
const TIME_STEP: f32 = 0.02;
const FRICTION_HALF_LIFE: f32 = 0.04;
const NUM_PARTICLES: usize = 2000;

fn main() {
    let mut rng: ThreadRng = rand::thread_rng();
    let colors: Vec<Color> = vec![
        Color::RED,
        Color::BLUE,
        Color::GREEN,
        Color::WHITE,
        Color::GRAY,
        Color::SKYBLUE,
        Color::ORANGE,
        Color::PINK,
        Color::PURPLE,
    ];
    let attraction_matrix = flat_matrix(colors.len(), &mut rng);

    let mut population: Vec<Particle> = (0..NUM_PARTICLES)
        .map(|_| Particle {
            color: (0..colors.len()).choose(&mut rng).expect(""),
            position: rvec2(rng.gen::<f32>(), rng.gen::<f32>()),
            velocity: Vector2::zero(),
        })
        .collect();

    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Particle Life")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_fps(0, 0);

        population = update_population(&population, &attraction_matrix);

        if d.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
            let mut mouse_pos = d.get_mouse_position();
            mouse_pos.x /= WIDTH as f32;
            mouse_pos.y /= HEIGHT as f32;
            population = population
                .iter_mut()
                .map(|p| {
                    p.velocity -= (p.position - mouse_pos).normalized() * 0.5;
                    *p
                })
                .collect();
        }

        for p in &population {
            d.draw_circle(
                (p.position.x * WIDTH as f32) as i32,
                (p.position.y * HEIGHT as f32) as i32,
                2.,
                colors[p.color],
            );
        }
    }
}
