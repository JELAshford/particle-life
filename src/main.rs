use macroquad::prelude::*;
use rayon::prelude::*;

#[derive(PartialEq, Clone, Copy)]
struct Particle {
    color: usize,
    position: Vec2,
    velocity: Vec2,
}

fn conf() -> Conf {
    Conf {
        window_title: String::from("Particle Life"),
        window_width: 800,
        window_height: 800,
        fullscreen: false,
        ..Default::default()
    }
}

fn flat_matrix(side_length: usize) -> Vec<f32> {
    (0..(side_length * side_length))
        .map(|_| rand::gen_range(-1., 1.))
        .collect()
}

fn generate_population(num_particles: usize, color_array: &Vec<Color>) -> Vec<Particle> {
    (0..num_particles)
        .map(|_| Particle {
            color: rand::gen_range(0, color_array.len()),
            position: vec2(rand::gen_range(0., 1.), rand::gen_range(0., 1.)),
            velocity: Vec2::ZERO,
        })
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
            new_p.velocity -= (new_p.position - vec2(0.5, 0.5)) / 128.;

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

fn attract_to_mouse(mut pop: Vec<Particle>) -> Vec<Particle> {
    if is_mouse_button_down(MouseButton::Left) {
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_pos = vec2(mouse_x / screen_width(), mouse_y / screen_height());
        pop = pop
            .iter_mut()
            .map(|p| {
                p.velocity -= (p.position - mouse_pos).normalize() * 0.1;
                *p
            })
            .collect();
    }
    pop
}

fn draw_fps() -> () {
    draw_text(format!("FPS {}", get_fps()).as_str(), 10., 30., 30., WHITE);
}

fn draw_particles(pop: &Vec<Particle>, color_array: &Vec<Color>) -> () {
    for p in pop {
        draw_circle(
            p.position.x * screen_width() as f32,
            p.position.y * screen_height() as f32,
            2.,
            color_array[p.color],
        );
    }
}

const MAX_RADIUS: f32 = 0.05;
const TIME_STEP: f32 = 0.02;
const FRICTION_HALF_LIFE: f32 = 0.04;
const NUM_PARTICLES: usize = 5000;

#[macroquad::main(conf)]
async fn main() {
    let colors: Vec<Color> = vec![RED, BLUE, GREEN, WHITE, GRAY, SKYBLUE, ORANGE, PINK, PURPLE];
    let attraction_matrix = flat_matrix(colors.len());
    let mut population: Vec<Particle> = generate_population(NUM_PARTICLES, &colors);
    loop {
        clear_background(BLACK);
        population = update_population(&population, &attraction_matrix);
        population = attract_to_mouse(population);
        draw_particles(&population, &colors);
        draw_fps();
        next_frame().await
    }
}
