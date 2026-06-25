use crate::canvas::Canvas;
use crate::anim::Animation;
use crate::anim::procedural::evaluate_palette;
use rand::Rng;

type Vec3 = [f32; 3];
type Mat3 = [[f32; 3]; 3];

pub struct Icecube {
    speed: f32,
    palette: String,
    angle_x: f32,
    angle_y: f32,
    scale: f32,
    pan_x: f32,
    pan_y: f32,
    fixed_color: [u8; 3],
    t_accum: f32,

    // Rotational matrix state (local cube space to world space)
    base_r: Mat3,
    curr_r: Mat3,

    // Step-rotation parameters
    rot_axis: usize,
    rot_direction: f32,
    transition_timer: f32,
    transition_duration: f32,
    next_rotation_timer: f32,
}

impl Icecube {
    pub fn new(
        speed: f32,
        palette: String,
        angle_x: f32,
        angle_y: f32,
        scale: f32,
        pan_x: f32,
        pan_y: f32,
        fixed_color: [u8; 3],
    ) -> Self {
        let identity = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        Self {
            speed,
            palette,
            angle_x,
            angle_y,
            scale,
            pan_x,
            pan_y,
            fixed_color,
            t_accum: 0.0,
            base_r: identity,
            curr_r: identity,
            rot_axis: 0,
            rot_direction: 1.0,
            transition_timer: 3.0, // Start finished
            transition_duration: 2.0, // Sleek 2s rotation duration
            next_rotation_timer: 10.0,
        }
    }
}

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

fn transpose(m: &Mat3) -> Mat3 {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
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

fn hash3(x: f32, y: f32, z: f32) -> f32 {
    let s = x * 127.1 + y * 311.7 + z * 74.7;
    (s.sin() * 43758.5453123).fract().abs()
}

fn noise3(p: [f32; 3]) -> f32 {
    let ix = p[0].floor();
    let iy = p[1].floor();
    let iz = p[2].floor();
    let fx = p[0].fract();
    let fy = p[1].fract();
    let fz = p[2].fract();

    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);
    let uz = fz * fz * (3.0 - 2.0 * fz);

    let n000 = hash3(ix, iy, iz);
    let n100 = hash3(ix + 1.0, iy, iz);
    let n010 = hash3(ix, iy + 1.0, iz);
    let n110 = hash3(ix + 1.0, iy + 1.0, iz);
    let n001 = hash3(ix, iy, iz + 1.0);
    let n101 = hash3(ix + 1.0, iy, iz + 1.0);
    let n011 = hash3(ix, iy + 1.0, iz + 1.0);
    let n111 = hash3(ix + 1.0, iy + 1.0, iz + 1.0);

    let a = n000 + ux * (n100 - n000);
    let b = n010 + ux * (n110 - n010);
    let c = n001 + ux * (n101 - n001);
    let d = n011 + ux * (n111 - n011);

    let mac = a + uy * (b - a);
    let mbd = c + uy * (d - c);

    mac + uz * (mbd - mac)
}

fn ray_box_intersection(origin: Vec3, dir: Vec3, box_min: Vec3, box_max: Vec3) -> Option<(f32, f32)> {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;
    for i in 0..3 {
        if dir[i].abs() < 1e-6 {
            if origin[i] < box_min[i] || origin[i] > box_max[i] {
                return None;
            }
        } else {
            let inv_d = 1.0 / dir[i];
            let mut t1 = (box_min[i] - origin[i]) * inv_d;
            let mut t2 = (box_max[i] - origin[i]) * inv_d;
            if t1 > t2 {
                let temp = t1;
                t1 = t2;
                t2 = temp;
            }
            t_min = t_min.max(t1);
            t_max = t_max.min(t2);
            if t_min > t_max {
                return None;
            }
        }
    }
    if t_max < 0.0 {
        None
    } else {
        Some((t_min, t_max))
    }
}

fn get_box_normal(p: Vec3, box_min: Vec3, box_max: Vec3) -> Vec3 {
    let eps = 1e-4;
    if (p[0] - box_min[0]).abs() < eps { return [-1.0, 0.0, 0.0]; }
    if (p[0] - box_max[0]).abs() < eps { return [1.0, 0.0, 0.0]; }
    if (p[1] - box_min[1]).abs() < eps { return [0.0, -1.0, 0.0]; }
    if (p[1] - box_max[1]).abs() < eps { return [0.0, 1.0, 0.0]; }
    if (p[2] - box_min[2]).abs() < eps { return [0.0, 0.0, -1.0]; }
    if (p[2] - box_max[2]).abs() < eps { return [0.0, 0.0, 1.0]; }

    let mut normal = [0.0, 0.0, 0.0];
    let mut min_d = f32::MAX;
    for i in 0..3 {
        let d1 = (p[i] - box_min[i]).abs();
        if d1 < min_d { min_d = d1; normal = [0.0, 0.0, 0.0]; normal[i] = -1.0; }
        let d2 = (p[i] - box_max[i]).abs();
        if d2 < min_d { min_d = d2; normal = [0.0, 0.0, 0.0]; normal[i] = 1.0; }
    }
    normal
}

fn reflect(i: Vec3, n: Vec3) -> Vec3 {
    let dot = i[0]*n[0] + i[1]*n[1] + i[2]*n[2];
    [
        i[0] - 2.0 * dot * n[0],
        i[1] - 2.0 * dot * n[1],
        i[2] - 2.0 * dot * n[2],
    ]
}

fn refract(i: Vec3, n: Vec3, eta: f32) -> Option<Vec3> {
    let dot_i = i[0]*n[0] + i[1]*n[1] + i[2]*n[2];
    let k = 1.0 - eta * eta * (1.0 - dot_i * dot_i);
    if k < 0.0 {
        None
    } else {
        let f = eta * dot_i + k.sqrt();
        Some([
            eta * i[0] - f * n[0],
            eta * i[1] - f * n[1],
            eta * i[2] - f * n[2],
        ])
    }
}

fn intersect_sphere(origin: Vec3, dir: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let oc = [origin[0] - center[0], origin[1] - center[1], origin[2] - center[2]];
    let b = 2.0 * (dir[0] * oc[0] + dir[1] * oc[1] + dir[2] * oc[2]);
    let c = oc[0]*oc[0] + oc[1]*oc[1] + oc[2]*oc[2] - radius * radius;
    let disc = b*b - 4.0 * c;
    if disc < 0.0 {
        None
    } else {
        let t = (-b - disc.sqrt()) / 2.0;
        if t > 0.0 {
            Some(t)
        } else {
            None
        }
    }
}

fn get_sky_color(dir: Vec3) -> [f32; 3] {
    let y = dir[1];
    let factor = (y * 0.5 + 0.5).clamp(0.0, 1.0);
    [
        0.08 * (1.0 - factor) + 0.6 * factor,
        0.25 * (1.0 - factor) + 0.85 * factor,
        0.50 * (1.0 - factor) + 1.0 * factor,
    ]
}

fn trace_ray(origin: Vec3, dir: Vec3, r_cube: &Mat3, t_time: f32, palette: &str, fixed_color: [u8; 3]) -> [u8; 3] {
    let box_min = [-1.0, -1.0, -1.0];
    let box_max = [1.0, 1.0, 1.0];

    let r_cube_t = transpose(r_cube);
    let local_origin = mat_mul_vec(&r_cube_t, &origin);
    let local_dir = mat_mul_vec(&r_cube_t, &dir);

    if let Some((t_min, t_max)) = ray_box_intersection(local_origin, local_dir, box_min, box_max) {
        let local_hit = [
            local_origin[0] + t_min * local_dir[0],
            local_origin[1] + t_min * local_dir[1],
            local_origin[2] + t_min * local_dir[2],
        ];

        let local_normal = get_box_normal(local_hit, box_min, box_max);

        // Add bump mapping / surface noise in local space
        let freq = 8.0;
        let n_eps = 0.01;
        let p_noise = [local_hit[0] * freq, local_hit[1] * freq, local_hit[2] * freq];
        let nx = noise3([p_noise[0] + n_eps, p_noise[1], p_noise[2]]) - noise3([p_noise[0] - n_eps, p_noise[1], p_noise[2]]);
        let ny = noise3([p_noise[0], p_noise[1] + n_eps, p_noise[2]]) - noise3([p_noise[0], p_noise[1] - n_eps, p_noise[2]]);
        let nz = noise3([p_noise[0], p_noise[1], p_noise[2] + n_eps]) - noise3([p_noise[0], p_noise[1], p_noise[2] - n_eps]);
        let bump = 0.08;
        let local_perturbed_normal = [
            local_normal[0] + bump * nx,
            local_normal[1] + bump * ny,
            local_normal[2] + bump * nz,
        ];

        let n_len = (local_perturbed_normal[0]*local_perturbed_normal[0] + local_perturbed_normal[1]*local_perturbed_normal[1] + local_perturbed_normal[2]*local_perturbed_normal[2]).sqrt();
        let local_perturbed_normal = if n_len > 0.0 {
            [local_perturbed_normal[0] / n_len, local_perturbed_normal[1] / n_len, local_perturbed_normal[2] / n_len]
        } else {
            local_normal
        };

        // Transform normals to world space for lighting
        let world_normal = mat_mul_vec(r_cube, &local_normal);
        let world_perturbed_normal = mat_mul_vec(r_cube, &local_perturbed_normal);

        // Fresnel reflection factor
        let cos_theta = -(dir[0]*world_perturbed_normal[0] + dir[1]*world_perturbed_normal[1] + dir[2]*world_perturbed_normal[2]);
        let cos_theta = cos_theta.clamp(0.0, 1.0);
        let r0 = (-0.31f32 / 2.31f32).powi(2);
        let fresnel = r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5);

        // Light 1 rotates around the vertical axis
        let l1_angle = t_time * 1.5;
        let l1 = [l1_angle.cos(), 0.6, l1_angle.sin()];
        let l1_len = (l1[0]*l1[0] + l1[1]*l1[1] + l1[2]*l1[2]).sqrt();
        let l1 = [l1[0]/l1_len, l1[1]/l1_len, l1[2]/l1_len];

        // Light 2 is stationary fill light
        let l2 = [-0.5f32, 0.8f32, -0.3f32];
        let l2_len = (l2[0]*l2[0] + l2[1]*l2[1] + l2[2]*l2[2]).sqrt();
        let l2 = [l2[0]/l2_len, l2[1]/l2_len, l2[2]/l2_len];

        // Calculate face-dependent shading
        // Distinguishes top, left, and right faces stably (maintains 3D volume readability)
        // 85% stable hemispherical lighting + 15% dynamic diffuse breathing
        let stable_shade = 0.48f32 + 0.38f32 * world_normal[1].max(0.0) + 0.14f32 * world_normal[0].max(0.0);
        let dot_l1 = (world_normal[0] * l1[0] + world_normal[1] * l1[1] + world_normal[2] * l1[2]).max(0.0);
        let dynamic_shade = 0.3f32 + 0.7f32 * dot_l1;
        let face_shade = 0.85f32 * stable_shade + 0.15f32 * dynamic_shade;

        let reflect_dir = reflect(dir, world_perturbed_normal);

        let spec1 = (reflect_dir[0]*l1[0] + reflect_dir[1]*l1[1] + reflect_dir[2]*l1[2]).max(0.0).powf(24.0);
        let spec2 = (reflect_dir[0]*l2[0] + reflect_dir[1]*l2[1] + reflect_dir[2]*l2[2]).max(0.0).powf(16.0);

        let reflection_color = [
            (spec1 * 255.0 + spec2 * 128.0).min(255.0),
            (spec1 * 255.0 + spec2 * 128.0).min(255.0),
            (spec1 * 255.0 + spec2 * 128.0).min(255.0),
        ];

        // Refraction ray inside ice
        let mut refraction_color = [0.0, 0.0, 0.0];
        let eta = 1.0 / 1.31;
        if let Some(refract_dir) = refract(dir, world_perturbed_normal, eta) {
            let local_refract_dir = mat_mul_vec(&r_cube_t, &refract_dir);
            let bubble_t = t_time * 0.5;
            let bubbles = [
                ([0.3 * (bubble_t + 1.0).sin(), 0.4 * bubble_t.cos(), 0.3 * (bubble_t * 1.5).cos()], 0.12),
                ([-0.4 * bubble_t.cos(), -0.3 * (bubble_t + 2.0).sin(), 0.2 * bubble_t.sin()], 0.16),
                ([0.1 * (bubble_t * 2.0).cos(), -0.5 * bubble_t.sin(), -0.4 * bubble_t.cos()], 0.10),
            ];

            let mut bubble_hit = None;
            let mut min_bubble_t = f32::MAX;

            for (idx, &(center, radius)) in bubbles.iter().enumerate() {
                if let Some(t_bub) = intersect_sphere(local_hit, local_refract_dir, center, radius) {
                    if t_bub < min_bubble_t && t_bub < (t_max - t_min) {
                        min_bubble_t = t_bub;
                        bubble_hit = Some((idx, center, radius));
                    }
                }
            }

            let mut tint = if palette == "fixed" {
                [fixed_color[0] as f32 / 255.0, fixed_color[1] as f32 / 255.0, fixed_color[2] as f32 / 255.0]
            } else {
                let p_col = evaluate_palette(palette, (t_time * 0.1).fract());
                [p_col[0] as f32 / 255.0, p_col[1] as f32 / 255.0, p_col[2] as f32 / 255.0]
            };

            // Apply face shading and subtle normal tinting
            let shade_y = (world_normal[1] * 0.5 + 0.5).clamp(0.0, 1.0);
            tint[0] *= face_shade * (0.8 + 0.2 * shade_y);
            tint[1] *= face_shade * (0.9 + 0.1 * shade_y);
            tint[2] *= face_shade;

            if let Some((_idx, center, radius)) = bubble_hit {
                let local_b_hit_point = [
                    local_hit[0] + min_bubble_t * local_refract_dir[0],
                    local_hit[1] + min_bubble_t * local_refract_dir[1],
                    local_hit[2] + min_bubble_t * local_refract_dir[2],
                ];
                let local_b_normal = [
                    (local_b_hit_point[0] - center[0]) / radius,
                    (local_b_hit_point[1] - center[1]) / radius,
                    (local_b_hit_point[2] - center[2]) / radius,
                ];
                let world_b_normal = mat_mul_vec(r_cube, &local_b_normal);

                let b_refdir = reflect(refract_dir, world_b_normal);
                let b_spec = (b_refdir[0]*l1[0] + b_refdir[1]*l1[1] + b_refdir[2]*l1[2]).max(0.0).powf(16.0);

                let sky_val = get_sky_color(b_refdir);
                refraction_color = [
                    (sky_val[0] * tint[0] * 0.6 + b_spec * 0.8) * 255.0,
                    (sky_val[1] * tint[1] * 0.85 + b_spec * 0.8) * 255.0,
                    (sky_val[2] * tint[2] * 1.0 + b_spec * 0.8) * 255.0,
                ];
            } else {
                let local_exit_point = [
                    local_hit[0] + (t_max - t_min) * local_refract_dir[0],
                    local_hit[1] + (t_max - t_min) * local_refract_dir[1],
                    local_hit[2] + (t_max - t_min) * local_refract_dir[2],
                ];
                let local_exit_normal = get_box_normal(local_exit_point, box_min, box_max);
                let world_exit_normal = mat_mul_vec(r_cube, &local_exit_normal);

                let mut final_dir = refract_dir;
                let eta_exit = 1.31;
                if let Some(exit_refract_dir) = refract(refract_dir, [-world_exit_normal[0], -world_exit_normal[1], -world_exit_normal[2]], eta_exit) {
                    final_dir = exit_refract_dir;
                }

                let sky_val = get_sky_color(final_dir);
                let local_mid_point = [
                    (local_hit[0] + local_exit_point[0]) * 0.5,
                    (local_hit[1] + local_exit_point[1]) * 0.5,
                    (local_hit[2] + local_exit_point[2]) * 0.5,
                ];
                let int_noise = noise3([local_mid_point[0]*4.0 + t_time, local_mid_point[1]*4.0, local_mid_point[2]*4.0]);
                let glisten_spark = if int_noise > 0.82 { (int_noise - 0.82) * 5.0 } else { 0.0 };

                refraction_color = [
                    (sky_val[0] * tint[0] * 0.65 + glisten_spark) * 255.0,
                    (sky_val[1] * tint[1] * 0.90 + glisten_spark) * 255.0,
                    (sky_val[2] * tint[2] * 1.0 + glisten_spark) * 255.0,
                ];
            }
        }

        // Combine reflection and refraction
        let mut r_final = (fresnel * reflection_color[0] as f32 + (1.0 - fresnel) * refraction_color[0]).clamp(0.0, 255.0);
        let mut g_final = (fresnel * reflection_color[1] as f32 + (1.0 - fresnel) * refraction_color[1]).clamp(0.0, 255.0);
        let mut b_final = (fresnel * reflection_color[2] as f32 + (1.0 - fresnel) * refraction_color[2]).clamp(0.0, 255.0);

        // Apply edge highlight or shadow to define the faces cleanly (using local coordinates so it rotates)
        let mut max_other = 0.0f32;
        let eps = 1e-3;
        for i in 0..3 {
            if (local_hit[i].abs() - 1.0).abs() > eps {
                max_other = max_other.max(local_hit[i].abs());
            }
        }
        if max_other > 0.93 {
            let t = (max_other - 0.93) / 0.07;
            let edge_highlight = 0.25f32 * t * (world_normal[1] * 0.5 + 0.5);
            r_final = (r_final + edge_highlight * 255.0).min(255.0);
            g_final = (g_final + edge_highlight * 255.0).min(255.0);
            b_final = (b_final + edge_highlight * 255.0).min(255.0);

            if max_other > 0.97 {
                let seam_t = (max_other - 0.97) / 0.03;
                let edge_shadow = 1.0 - 0.4 * seam_t;
                r_final *= edge_shadow;
                g_final *= edge_shadow;
                b_final *= edge_shadow;
            }
        }

        [r_final as u8, g_final as u8, b_final as u8]
    } else {
        [0, 0, 0]
    }
}

impl Animation for Icecube {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.t_accum += dt * self.speed;

        // Auto-rotation state machine
        self.next_rotation_timer -= dt;
        if self.next_rotation_timer <= 0.0 {
            if self.transition_timer < self.transition_duration {
                let end_r = rot_matrix(self.rot_axis, self.rot_direction * 90.0);
                self.base_r = mat_mul_mat(&self.base_r, &end_r);
            }

            let mut rng = rand::thread_rng();
            self.rot_axis = rng.gen_range(0..3); // 0 = X, 1 = Y, 2 = Z
            self.rot_direction = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };

            self.transition_timer = 0.0;
            self.next_rotation_timer = rng.gen_range(8.0..12.0); // every ~10s
        }

        if self.transition_timer < self.transition_duration {
            self.transition_timer += dt;
            if self.transition_timer >= self.transition_duration {
                let end_r = rot_matrix(self.rot_axis, self.rot_direction * 90.0);
                self.base_r = mat_mul_mat(&self.base_r, &end_r);
                self.curr_r = self.base_r;
            } else {
                let t = self.transition_timer / self.transition_duration;
                let t_smooth = t * t * (3.0 - 2.0 * t);
                let step_r = rot_matrix(self.rot_axis, self.rot_direction * t_smooth * 90.0);
                self.curr_r = mat_mul_mat(&self.base_r, &step_r);
            }
        } else {
            self.curr_r = self.base_r;
        }

        let mut canvas = Canvas::black();

        let r0 = rot_matrix(0, self.angle_x);
        let r1 = rot_matrix(1, self.angle_y);
        let view = mat_mul_mat(&r0, &r1);

        let cx = 15.5 + self.pan_x;
        let cy = 15.5 + self.pan_y;

        let u_dir = [view[0][0], view[0][1], view[0][2]];
        let v_dir = [view[1][0], view[1][1], view[1][2]];
        let w_dir = [view[2][0], view[2][1], view[2][2]];

        for y in 0..32 {
            for x in 0..32 {
                let px = (x as f32 - cx) / self.scale;
                let py = (cy - y as f32) / self.scale;

                let origin = [
                    px * u_dir[0] + py * v_dir[0] + 8.0 * w_dir[0],
                    px * u_dir[1] + py * v_dir[1] + 8.0 * w_dir[1],
                    px * u_dir[2] + py * v_dir[2] + 8.0 * w_dir[2],
                ];
                let dir = [-w_dir[0], -w_dir[1], -w_dir[2]];

                let color = trace_ray(origin, dir, &self.curr_r, self.t_accum, &self.palette, self.fixed_color);
                canvas.set(x, y, color);
            }
        }

        canvas
    }
}
