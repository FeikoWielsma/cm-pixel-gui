use crate::canvas::Canvas;
use crate::anim::Animation;
use rand::Rng;

type Vec3 = [f32; 3];
type Mat3 = [[f32; 3]; 3];

const LIGHT: Vec3 = [0.29318268, 0.8376648, 0.46071563];

fn rot_matrix(axis: usize, deg: f32) -> Mat3 {
    let r = deg.to_radians();
    let c = r.cos();
    let s = r.sin();
    match axis {
        0 => [
            [1.0, 0.0, 0.0],
            [0.0, c, -s],
            [0.0, s, c],
        ],
        1 => [
            [c, 0.0, s],
            [0.0, 1.0, 0.0],
            [-s, 0.0, c],
        ],
        _ => [
            [c, -s, 0.0],
            [s, c, 0.0],
            [0.0, 0.0, 1.0],
        ],
    }
}

fn mat_mul_vec(m: &Mat3, v: &Vec3) -> Vec3 {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn mat_mul_mat(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut out = [[0.0; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            out[r][c] = a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c];
        }
    }
    out
}



fn rot_matrix_int(axis: usize, direction: i32) -> [[i32; 3]; 3] {
    let s = direction;
    match axis {
        0 => [
            [1, 0, 0],
            [0, 0, -s],
            [0, s, 0],
        ],
        1 => [
            [0, 0, s],
            [0, 1, 0],
            [-s, 0, 0],
        ],
        _ => [
            [0, -s, 0],
            [s, 0, 0],
            [0, 0, 1],
        ],
    }
}

fn mat_mul_vec_int(m: &[[i32; 3]; 3], v: &[i32; 3]) -> [i32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn get_sticker_color(palette: &str, normal: [i32; 3]) -> [u8; 3] {
    let colors = match palette {
        "catppuccin" => [
            [166, 218, 149], // +x green
            [138, 173, 244], // -x blue
            [245, 245, 245], // +y white (top)
            [238, 212, 159], // -y yellow
            [237, 135, 150], // +z red
            [245, 169, 127], // -z orange
        ],
        "everforest" => [
            [167, 192, 128], // +x green
            [127, 187, 179], // -x blue
            [218, 223, 172], // +y white (top)
            [219, 190, 106], // -y yellow
            [230, 126, 124], // +z red
            [245, 122, 85],  // -z orange
        ],
        "nord" => [
            [163, 190, 140], // +x green
            [129, 161, 193], // -x blue
            [216, 222, 233], // +y white (top)
            [235, 203, 139], // -y yellow
            [191, 97, 106],  // +z red
            [143, 188, 187], // -z orange
        ],
        "dracula" => [
            [80, 250, 123],  // +x green
            [139, 233, 253], // -x blue
            [248, 248, 242], // +y white (top)
            [241, 250, 140], // -y yellow
            [255, 85, 85],   // +z red
            [255, 184, 108], // -z orange
        ],
        "sunset" => [
            [4, 139, 168],   // +x green (using blue-green)
            [72, 12, 168],   // -x blue
            [255, 255, 255], // +y white (top)
            [255, 186, 8],   // -y yellow
            [247, 37, 133],  // +z red (magenta)
            [252, 76, 2],    // -z orange
        ],
        "cyberpunk" => [
            [0, 255, 255],   // +x green (using neon cyan)
            [0, 0, 255],     // -x blue
            [255, 255, 255], // +y white (top)
            [255, 255, 0],   // -y yellow
            [255, 0, 127],   // +z red (neon pink)
            [255, 95, 0],    // -z orange (hot orange)
        ],
        _ => [
            [0, 210, 40],     // +x green
            [0, 90, 255],    // -x blue
            [245, 245, 245],  // +y white (top)
            [255, 210, 0],   // -y yellow
            [235, 0, 0],      // +z red
            [255, 95, 0],    // -z orange
        ],
    };

    match normal {
        [1, 0, 0] => colors[0],
        [-1, 0, 0] => colors[1],
        [0, 1, 0] => colors[2],
        [0, -1, 0] => colors[3],
        [0, 0, 1] => colors[4],
        [0, 0, -1] => colors[5],
        _ => [0, 0, 0],
    }
}

#[derive(Clone)]
struct Sticker {
    normal: [i32; 3],
    color: [u8; 3],
}

#[derive(Clone)]
struct Cubie {
    pos: [i32; 3],
    stickers: Vec<Sticker>,
}

struct Cube {
    cubies: Vec<Cubie>,
}

impl Cube {
    fn new(palette: &str) -> Self {
        let mut cubies = Vec::new();
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if x == 0 && y == 0 && z == 0 {
                        continue;
                    }
                    let pos = [x, y, z];
                    let mut stickers = Vec::new();
                    for &(ax, n) in &[(0, x), (1, y), (2, z)] {
                        if n != 0 {
                            let mut normal = [0, 0, 0];
                            normal[ax] = n;
                            let color = get_sticker_color(palette, normal);
                            stickers.push(Sticker { normal, color });
                        }
                    }
                    cubies.push(Cubie { pos, stickers });
                }
            }
        }
        Self { cubies }
    }

    fn turn(&mut self, axis: usize, layer: i32, direction: i32) {
        let r = rot_matrix_int(axis, direction);
        for cub in &mut self.cubies {
            if cub.pos[axis] == layer {
                cub.pos = mat_mul_vec_int(&r, &cub.pos);
                for s in &mut cub.stickers {
                    s.normal = mat_mul_vec_int(&r, &s.normal);
                }
            }
        }
    }
}

fn get_basis(normal: [i32; 3]) -> (Vec3, Vec3) {
    let mut ax = 0;
    let mut max_val = normal[0].abs();
    if normal[1].abs() > max_val {
        ax = 1;
        max_val = normal[1].abs();
    }
    if normal[2].abs() > max_val {
        ax = 2;
    }

    let mut u = [0.0; 3];
    let mut v = [0.0; 3];
    u[(ax + 1) % 3] = 1.0;
    v[(ax + 2) % 3] = 1.0;

    (u, v)
}

fn is_inside(poly: &[(f32, f32)], px: f32, py: f32) -> bool {
    let mut sign = 0;
    let n = poly.len();
    for i in 0..n {
        let (ax, ay) = poly[i];
        let (bx, by) = poly[(i + 1) % n];
        let cross = (bx - ax) * (py - ay) - (by - ay) * (px - ax);
        if cross != 0.0 {
            let s = if cross > 0.0 { 1 } else { -1 };
            if sign == 0 {
                sign = s;
            } else if s != sign {
                return false;
            }
        }
    }
    true
}

fn fill_poly(canvas: &mut Canvas, pts: &[(f32, f32)], color: [u8; 3]) {
    if pts.is_empty() {
        return;
    }
    let mut min_x = pts[0].0;
    let mut max_x = pts[0].0;
    let mut min_y = pts[0].1;
    let mut max_y = pts[0].1;
    for &p in pts.iter().skip(1) {
        if p.0 < min_x { min_x = p.0; }
        if p.0 > max_x { max_x = p.0; }
        if p.1 < min_y { min_y = p.1; }
        if p.1 > max_y { max_y = p.1; }
    }

    let x0 = (min_x.floor() as i32).max(0) as usize;
    let x1 = (max_x.ceil() as i32).min(31) as usize;
    let y0 = (min_y.floor() as i32).max(0) as usize;
    let y1 = (max_y.ceil() as i32).min(31) as usize;

    for yy in y0..=y1 {
        for xx in x0..=x1 {
            if is_inside(pts, xx as f32 + 0.5, yy as f32 + 0.5) {
                canvas.set(xx, yy, color);
            }
        }
    }
}

struct Quad {
    depth: f32,
    black_pts: Vec<(f32, f32)>,
    col_pts: Vec<(f32, f32)>,
    col: [u8; 3],
}

fn render_cube(
    cube: &Cube,
    moving_axis: Option<usize>,
    moving_layer: Option<i32>,
    anim_r: Option<&Mat3>,
    angle_x: f32,
    angle_y: f32,
    scale: f32,
    pan_x: f32,
    pan_y: f32,
) -> Canvas {
    let mut canvas = Canvas::black();
    let r0 = rot_matrix(0, angle_x);
    let r1 = rot_matrix(1, angle_y);
    let view = mat_mul_mat(&r0, &r1);
    let cx = 15.5f32 + pan_x;
    let cy = 15.5f32 + pan_y;

    let mut quads = Vec::new();

    for cub in &cube.cubies {
        let in_layer = match (moving_axis, moving_layer) {
            (Some(ax), Some(layer)) => cub.pos[ax] == layer,
            _ => false,
        };

        for sticker in &cub.stickers {
            let mut n = [
                sticker.normal[0] as f32,
                sticker.normal[1] as f32,
                sticker.normal[2] as f32,
            ];
            let pos_f = [
                cub.pos[0] as f32,
                cub.pos[1] as f32,
                cub.pos[2] as f32,
            ];

            let center = [
                pos_f[0] + 0.5 * n[0],
                pos_f[1] + 0.5 * n[1],
                pos_f[2] + 0.5 * n[2],
            ];

            let (u, v) = get_basis(sticker.normal);

            let mut corners = [
                [center[0] + u[0]*0.5 + v[0]*0.5, center[1] + u[1]*0.5 + v[1]*0.5, center[2] + u[2]*0.5 + v[2]*0.5],
                [center[0] + u[0]*0.5 - v[0]*0.5, center[1] + u[1]*0.5 - v[1]*0.5, center[2] + u[2]*0.5 - v[2]*0.5],
                [center[0] - u[0]*0.5 - v[0]*0.5, center[1] - u[1]*0.5 - v[1]*0.5, center[2] - u[2]*0.5 - v[2]*0.5],
                [center[0] - u[0]*0.5 + v[0]*0.5, center[1] - u[1]*0.5 + v[1]*0.5, center[2] - u[2]*0.5 + v[2]*0.5],
            ];

            let mut face_corners = [
                [center[0] + u[0]*0.46 + v[0]*0.46, center[1] + u[1]*0.46 + v[1]*0.46, center[2] + u[2]*0.46 + v[2]*0.46],
                [center[0] + u[0]*0.46 - v[0]*0.46, center[1] + u[1]*0.46 - v[1]*0.46, center[2] + u[2]*0.46 - v[2]*0.46],
                [center[0] - u[0]*0.46 - v[0]*0.46, center[1] - u[1]*0.46 - v[1]*0.46, center[2] - u[2]*0.46 - v[2]*0.46],
                [center[0] - u[0]*0.46 + v[0]*0.46, center[1] - u[1]*0.46 + v[1]*0.46, center[2] - u[2]*0.46 + v[2]*0.46],
            ];

            if in_layer {
                if let Some(r_mat) = anim_r {
                    n = mat_mul_vec(r_mat, &n);
                    for c in &mut corners {
                        *c = mat_mul_vec(r_mat, c);
                    }
                    for c in &mut face_corners {
                        *c = mat_mul_vec(r_mat, c);
                    }
                }
            }

            let nv = mat_mul_vec(&view, &n);
            if nv[2] <= 0.02 {
                continue;
            }

            let dot = n[0] * LIGHT[0] + n[1] * LIGHT[1] + n[2] * LIGHT[2];
            let shade = 0.45 + 0.55 * dot.max(0.0);

            let mut depth = 0.0f32;
            let mut black_pts = Vec::with_capacity(4);
            for c in &corners {
                let cv = mat_mul_vec(&view, c);
                depth += cv[2];
                black_pts.push((cx + cv[0] * scale, cy - cv[1] * scale));
            }

            let mut col_pts = Vec::with_capacity(4);
            for c in &face_corners {
                let cv = mat_mul_vec(&view, c);
                col_pts.push((cx + cv[0] * scale, cy - cv[1] * scale));
            }

            let col = [
                (sticker.color[0] as f32 * shade).clamp(0.0, 255.0) as u8,
                (sticker.color[1] as f32 * shade).clamp(0.0, 255.0) as u8,
                (sticker.color[2] as f32 * shade).clamp(0.0, 255.0) as u8,
            ];

            quads.push(Quad {
                depth,
                black_pts,
                col_pts,
                col,
            });
        }
    }

    quads.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal));

    for quad in quads {
        fill_poly(&mut canvas, &quad.black_pts, [8, 8, 8]);
        fill_poly(&mut canvas, &quad.col_pts, quad.col);
    }

    canvas
}

#[derive(Clone, Copy)]
struct Move {
    axis: usize,
    layer: i32,
    direction: i32,
}

fn scramble(n: usize) -> Vec<Move> {
    let mut rng = rand::thread_rng();
    let mut moves = Vec::new();
    let mut last = None;
    for _ in 0..n {
        let mut ax;
        let mut layer;
        loop {
            ax = rng.gen_range(0..3);
            layer = if rng.gen_bool(0.5) { -1 } else { 1 };
            if last != Some((ax, layer)) {
                break;
            }
        }
        let direction = if rng.gen_bool(0.5) { -1 } else { 1 };
        moves.push(Move { axis: ax, layer, direction });
        last = Some((ax, layer));
    }
    moves
}

enum State {
    Idle,
    Pause(usize),
    Moving {
        move_data: Move,
        current_step: usize,
        total_steps: usize,
    },
}

pub struct Rubik {
    speed: f32,
    cube: Cube,
    queue: Vec<(Move, usize)>,
    state: State,
    accumulator: f32,
    angle_x: f32,
    angle_y: f32,
    scale: f32,
    pan_x: f32,
    pan_y: f32,
}

impl Rubik {
    pub fn new(
        speed: f32,
        palette: String,
        angle_x: f32,
        angle_y: f32,
        scale: f32,
        pan_x: f32,
        pan_y: f32,
    ) -> Self {
        Self {
            speed,
            cube: Cube::new(&palette),
            queue: Vec::new(),
            state: State::Idle,
            accumulator: 0.0,
            angle_x,
            angle_y,
            scale,
            pan_x,
            pan_y,
        }
    }
}

impl Animation for Rubik {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.accumulator += dt * 15.0;
        let limit = 10;
        let mut steps_run = 0;

        while self.accumulator >= 1.0 && steps_run < limit {
            match &mut self.state {
                State::Pause(frames) => {
                    if *frames > 1 {
                        *frames -= 1;
                    } else {
                        self.state = State::Idle;
                    }
                }
                State::Moving { move_data, current_step, total_steps } => {
                    if *current_step < *total_steps {
                        *current_step += 1;
                    } else {
                        self.cube.turn(move_data.axis, move_data.layer, move_data.direction);
                        if self.queue.is_empty() {
                            let pause_frames = ((18.0 / self.speed.max(0.25)) as usize).max(1);
                            self.state = State::Pause(pause_frames);
                        } else {
                            self.state = State::Idle;
                        }
                    }
                }
                State::Idle => {
                    if self.queue.is_empty() {
                        let scr = scramble(16);
                        let mut sol = scr.clone();
                        sol.reverse();
                        for m in &mut sol {
                            m.direction = -m.direction;
                        }

                        let mut queue = Vec::new();
                        for m in scr {
                            queue.push((m, 3));
                        }
                        for m in sol {
                            queue.push((m, 9));
                        }
                        self.queue = queue;
                    }

                    if !self.queue.is_empty() {
                        let (m, fpm) = self.queue.remove(0);
                        let total = (((fpm as f32) / self.speed.max(0.25)).round() as usize).max(1);
                        self.state = State::Moving {
                            move_data: m,
                            current_step: 1,
                            total_steps: total,
                        };
                    }
                }
            }
            self.accumulator -= 1.0;
            steps_run += 1;
        }

        match &self.state {
            State::Moving { move_data, current_step, total_steps } => {
                let theta = 90.0 * (move_data.direction as f32) * (*current_step as f32) / (*total_steps as f32);
                let rot = rot_matrix(move_data.axis, theta);
                render_cube(
                    &self.cube,
                    Some(move_data.axis),
                    Some(move_data.layer),
                    Some(&rot),
                    self.angle_x,
                    self.angle_y,
                    self.scale,
                    self.pan_x,
                    self.pan_y,
                )
            }
            _ => render_cube(
                &self.cube,
                None,
                None,
                None,
                self.angle_x,
                self.angle_y,
                self.scale,
                self.pan_x,
                self.pan_y,
            ),
        }
    }
}
