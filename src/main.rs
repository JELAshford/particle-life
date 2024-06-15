use ::rand::distributions::{Distribution, Uniform};
use ::rand::prelude::*;
use kiddo::{KdTree, SquaredEuclidean};
use macroquad::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

struct Camera {
    position: Vec2,
    position_vel: Vec2,
    scroll_decay: f32,
    zoom: f32,
    zoom_mult: f32,
    zoom_interp: f32,
    abs_max_vel: f32,
}
impl Camera {
    fn default() -> Self {
        Camera {
            position: Vec2::ZERO,
            position_vel: Vec2::ZERO,
            zoom: 1.,
            zoom_mult: 1.,
            scroll_decay: 0.9,
            zoom_interp: 0.1,
            abs_max_vel: 10.,
        }
    }
    fn update(mut self) -> Self {
        self.position_vel = self.position_vel.clamp(
            vec2(-self.abs_max_vel, -self.abs_max_vel),
            vec2(self.abs_max_vel, self.abs_max_vel),
        );
        self.position += self.position_vel * self.zoom;
        self.position_vel *= self.scroll_decay;

        self.zoom *= self.zoom_mult;
        self.zoom_mult = (1. - self.zoom_interp) * self.zoom_mult + (self.zoom_interp * 1.);
        self
    }
}

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

fn randomise_attraction(
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
    let max_abs_width = screen_width() as f32 / 2.5;
    let max_abs_height = screen_width() as f32 / 2.5;
    (0..num_particles)
        .map(|_| Particle {
            color: rand::gen_range(0, color_array.len()),
            position: vec2(
                rand::gen_range(-max_abs_width, max_abs_width),
                rand::gen_range(-max_abs_height, max_abs_height),
            ),
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

fn attract_to_mouse(
    mut population_information: PopulationInfo,
    camera_obj: &Camera,
) -> PopulationInfo {
    let centering_vec = vec2(screen_width() as f32 / 2., screen_height() as f32 / 2.);
    if is_mouse_button_down(MouseButton::Left) {
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_pos =
            ((vec2(mouse_x, mouse_y) - centering_vec) * camera_obj.zoom) + camera_obj.position;
        for p in &mut population_information.particles {
            p.velocity -= (p.position - mouse_pos).normalize() * 5.;
        }
    }
    population_information
}

fn draw_fps() -> () {
    draw_text(format!("FPS {}", get_fps()).as_str(), 10., 30., 30., WHITE);
}

fn draw_particles(pop: &Vec<Particle>, color_array: &[Color], camera_obj: &Camera) -> () {
    let centering_vec = vec2(screen_width() as f32 / 2., screen_height() as f32 / 2.);
    let view_width = screen_width() as f32 * camera_obj.zoom;
    let view_height = screen_height() as f32 * camera_obj.zoom;
    let camera_obj_rect: Rect = Rect {
        x: camera_obj.position.x - (view_width / 2.),
        y: camera_obj.position.y - (view_height / 2.),
        w: view_width,
        h: view_height,
    };
    for p in pop {
        if camera_obj_rect.contains(p.position) {
            let draw_position =
                ((p.position - camera_obj.position) / camera_obj.zoom) + centering_vec;
            draw_circle(draw_position.x, draw_position.y, 2., color_array[p.color]);
        }
    }
}

fn update_camera(mut camera: Camera) -> Camera {
    let (_scroll_x, scroll_y) = mouse_wheel();
    if is_key_down(KeyCode::W) {
        camera.position_vel.y -= 0.3;
    }
    if is_key_down(KeyCode::S) {
        camera.position_vel.y += 0.3;
    }
    if is_key_down(KeyCode::A) {
        camera.position_vel.x -= 0.3;
    }
    if is_key_down(KeyCode::D) {
        camera.position_vel.x += 0.3;
    }
    if scroll_y < 0. {
        camera.zoom_mult -= 0.005;
    }
    if scroll_y > 0. {
        camera.zoom_mult += 0.005;
    }
    camera.update()
}

const SEED: u64 = 50;
const MAX_RADIUS: f32 = 30.;
const TIME_STEP: f32 = 0.02;
const FRICTION_HALF_LIFE: f32 = 0.04;
const NUM_PARTICLES: usize = 10000;
const FORCE_BETA: f32 = 0.3;
const COLORS: [Color; 5] = [RED, ORANGE, YELLOW, WHITE, VIOLET];

#[macroquad::main(conf)]
async fn main() {
    let mut camera = Camera::default();
    let num_colours = COLORS.len();
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut attraction_matrix = flat_matrix(num_colours, &mut rng);
    let mut pop_info: PopulationInfo =
        PopulationInfo::new(generate_population(NUM_PARTICLES, &COLORS));

    loop {
        // Run simulation update
        pop_info = update_population(pop_info, &attraction_matrix, num_colours);
        pop_info.kdtree = PopulationInfo::generate_poptree(&pop_info.particles);

        // User interaction
        camera = update_camera(camera);
        attraction_matrix = randomise_attraction(attraction_matrix, num_colours, &mut rng);
        pop_info = attract_to_mouse(pop_info, &camera);

        // Draw particles/UI
        clear_background(BLACK);
        draw_particles(&pop_info.particles, &COLORS, &camera);
        draw_fps();

        next_frame().await
    }
}
