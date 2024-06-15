use ::rand::distributions::{Distribution, Uniform};
use ::rand::prelude::*;
use kiddo::{KdTree, SquaredEuclidean};
use macroquad::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

#[derive(PartialEq, Clone)]
struct Particle {
    color: usize,
    position: Vec2,
    velocity: Vec2,
}

struct PopulationInfo {
    particles: Vec<Particle>,
    kdtree: KdTree<f32, 2>,
}
impl PopulationInfo {
    fn generate_poptree(particle_vec: &Vec<Particle>) -> KdTree<f32, 2> {
        let mut pop_tree: KdTree<f32, 2> = KdTree::new();
        for (ind, p) in particle_vec.iter().enumerate() {
            pop_tree.add(&p.position.to_array(), ind as u64);
        }
        pop_tree
    }
    fn new(particle_vec: Vec<Particle>) -> Self {
        PopulationInfo {
            kdtree: Self::generate_poptree(&particle_vec),
            particles: particle_vec,
        }
    }
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

fn flat_matrix(side_length: usize, rand_obj: &mut ChaCha8Rng) -> Vec<f32> {
    let dist = Uniform::from(-1f32..1f32);
    (0..(side_length * side_length))
        .map(|_| dist.sample(rand_obj))
        .collect()
}

fn reset_attraction(
    attractions: Vec<f32>,
    num_colours: usize,
    rand_obj: &mut ChaCha8Rng,
) -> Vec<f32> {
    if is_key_pressed(KeyCode::Space) {
        flat_matrix(num_colours, rand_obj)
    } else {
        attractions
    }
}

fn generate_population(num_particles: usize, color_array: &[Color]) -> Vec<Particle> {
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

fn update_population(
    mut population_information: PopulationInfo,
    attractions: &Vec<f32>,
    num_colours: usize,
) -> PopulationInfo {
    let friction_factor: f32 = 0.5_f32.powf(TIME_STEP / FRICTION_HALF_LIFE);
    let population = &population_information.particles;
    population_information.particles = population_information
        .particles
        .par_iter()
        .map(|p1| {
            let mut total_force = Vec2::ZERO;
            for neighbour in population_information
                .kdtree
                .within_unsorted::<SquaredEuclidean>(&p1.position.to_array(), MAX_RADIUS.powf(2.))
            {
                let distance = neighbour.distance.sqrt();
                let p2 = &population[neighbour.item as usize];
                if p1 == p2 {
                    continue;
                };
                let f = force(
                    distance / MAX_RADIUS,
                    attractions[(p1.color * num_colours) + p2.color],
                    FORCE_BETA,
                );
                total_force += ((p2.position - p1.position) / distance) * f;
            }
            total_force *= MAX_RADIUS;

            // Create new particle with velocity driven by this force
            let mut new_p = p1.clone();
            new_p.velocity *= friction_factor;
            new_p.velocity += total_force * TIME_STEP;

            // Update position based on this velocity
            new_p.position += new_p.velocity * TIME_STEP;
            new_p
        })
        .collect();
    population_information
}

fn attract_to_mouse(mut population_information: PopulationInfo) -> PopulationInfo {
    if is_mouse_button_down(MouseButton::Left) {
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_pos = vec2(mouse_x / screen_width(), mouse_y / screen_height());
        for p in &mut population_information.particles {
            p.velocity -= (p.position - mouse_pos).normalize() * 0.1;
        }
    }
    population_information
}

fn draw_fps() -> () {
    draw_text(format!("FPS {}", get_fps()).as_str(), 10., 30., 30., WHITE);
}

fn draw_particles(pop: &Vec<Particle>, color_array: &[Color]) -> () {
    for p in pop {
        draw_circle(
            p.position.x * screen_width() as f32,
            p.position.y * screen_height() as f32,
            2.,
            color_array[p.color],
        );
    }
}

const SEED: u64 = 50;
const MAX_RADIUS: f32 = 0.05;
const TIME_STEP: f32 = 0.02;
const FRICTION_HALF_LIFE: f32 = 0.04;
const NUM_PARTICLES: usize = 10000;
const FORCE_BETA: f32 = 0.3;
const COLORS: [Color; 5] = [RED, ORANGE, YELLOW, WHITE, VIOLET];

#[macroquad::main(conf)]
async fn main() {
    let num_colours = COLORS.len();
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut attraction_matrix = flat_matrix(num_colours, &mut rng);
    let mut pop_info: PopulationInfo =
        PopulationInfo::new(generate_population(NUM_PARTICLES, &COLORS));

    loop {
        clear_background(BLACK);
        attraction_matrix = reset_attraction(attraction_matrix, num_colours, &mut rng);
        pop_info = update_population(pop_info, &attraction_matrix, num_colours);
        pop_info = attract_to_mouse(pop_info);
        pop_info.kdtree = PopulationInfo::generate_poptree(&pop_info.particles);
        draw_particles(&pop_info.particles, &COLORS);
        draw_fps();
        next_frame().await
    }
}
