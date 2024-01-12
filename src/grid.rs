use macroquad::prelude::*;

#[derive(PartialEq, Clone, Copy)]
pub struct Particle {
    pub color: usize,
    pub position: Vec2,
    pub velocity: Vec2,
}

struct GridEntry {
    key_val: usize,
    point_index: usize,
}
pub struct Grid {
    edge_size: f32,
    num_points: u32,
    spatial_lookup: Vec<GridEntry>,
    start_indices: Vec<usize>,
}
impl Grid {
    pub fn pos_to_coordinate(position: Vec2, edge_size: f32) -> Vec2 {
        (position / edge_size).floor()
    }
    pub fn hash_coordinate(coordinate: Vec2, num_points: u32) -> usize {
        ((coordinate.x as u32 * 15823 + coordinate.y as u32 * 9737333) % num_points) as usize
    }
    pub fn create(edge_size: f32, points: &Vec<Particle>) -> Self {
        let num_points = points.len() as u32;

        // Load in the points by assigning GridEntries for each point
        let mut spatial_lookup = Vec::<GridEntry>::with_capacity(points.len());
        for (ind, point) in points.iter().enumerate() {
            spatial_lookup.push(GridEntry {
                key_val: Grid::hash_coordinate(
                    Grid::pos_to_coordinate(point.position, edge_size),
                    num_points,
                ),
                point_index: ind,
            });
        }
        // Sort the lookup by key value
        spatial_lookup.sort_by_key(|grid_entry| grid_entry.key_val);
        // Regenerate start indices
        let mut start_indices = vec![usize::MAX; num_points as usize];
        for index in 0..(num_points as usize) {
            let key = spatial_lookup[index].key_val;
            let prev_key = if index == 0 {
                usize::MAX
            } else {
                spatial_lookup[index - 1].key_val
            };
            if key != prev_key {
                start_indices[key] = index;
            }
        }
        Grid {
            edge_size,
            num_points,
            spatial_lookup,
            start_indices,
        }
    }

    pub fn get_candidates(&self, point: Vec2) -> Vec<usize> {
        let mut possible_indexes = Vec::<usize>::with_capacity(self.num_points as usize);
        let pos_coord = Grid::pos_to_coordinate(point, self.edge_size);
        for x_off in [-1., 0., 1.] {
            for y_off in [-1., 0., 1.] {
                let offset_pos = pos_coord + vec2(x_off, y_off);
                let offset_key = Grid::hash_coordinate(offset_pos, self.num_points);
                let cell_start_index = self.start_indices[offset_key];
                for index in cell_start_index..(self.num_points as usize) {
                    if self.spatial_lookup[index].key_val != offset_key {
                        break;
                    }
                    let possible_point = self.spatial_lookup[index].point_index;
                    possible_indexes.push(possible_point)
                }
            }
        }
        possible_indexes
    }
}
