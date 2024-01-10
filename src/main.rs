use ::rand::{rngs::ThreadRng, seq::IteratorRandom, Rng};
use macroquad::prelude::*;
use rayon::prelude::*;

#[derive(PartialEq, Clone, Copy)]
struct Particle {
    color: usize,
    position: Vec2,
    velocity: Vec2,
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
    let mut new_population: Vec<Particle> = population
        .par_iter()
        .map(|p1| {
            let mut total_force = Vec2::ZERO;
            for p2 in population {
                if p1 == p2 {
                    continue;
                };
                let distance = p1.position.distance(p2.position);
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
            new_p.velocity -= (new_p.position - vec2(0.5, 0.5)) / 16.;

            new_p
        })
        .collect();

    // update positions
    new_population
        .iter_mut()
        .map(|p| {
            p.position += p.velocity * TIME_STEP;
            *p
        })
        .collect()
}

const MAX_RADIUS: f32 = 0.1;
const TIME_STEP: f32 = 0.01;
const FRICTION_HALF_LIFE: f32 = 0.04;
const NUM_PARTICLES: usize = 3000;

fn conf() -> Conf {
    Conf {
        window_title: String::from("Particle Life"),
        window_width: 800,
        window_height: 800,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut rng: ThreadRng = ::rand::thread_rng();
    let colors: Vec<Color> = vec![RED, BLUE, GREEN, WHITE, GRAY, SKYBLUE, ORANGE, PINK, PURPLE];
    let attraction_matrix = flat_matrix(colors.len(), &mut rng);

    let mut population: Vec<Particle> = (0..NUM_PARTICLES)
        .map(|_| Particle {
            color: (0..colors.len()).choose(&mut rng).expect(""),
            position: vec2(rng.gen::<f32>(), rng.gen::<f32>()),
            velocity: Vec2::ZERO,
        })
        .collect();

    loop {
        clear_background(BLACK);
        draw_text(format!("FPS {}", get_fps()).as_str(), 10., 30., 30., WHITE);

        population = update_population(&population, &attraction_matrix);

        if is_mouse_button_down(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            let mouse_pos = vec2(mouse_x / screen_width(), mouse_y / screen_height());
            population = population
                .iter_mut()
                .map(|p| {
                    p.velocity -= (p.position - mouse_pos).normalize() * 0.5;
                    *p
                })
                .collect();
        }

        for p in &population {
            draw_circle(
                p.position.x * screen_width() as f32,
                p.position.y * screen_height() as f32,
                2.,
                colors[p.color],
            );
        }

        next_frame().await
    }
}
