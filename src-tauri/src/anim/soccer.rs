use crate::anim::text::{draw_text, text_width};
use crate::anim::Animation;
use crate::canvas::Canvas;
use rand::Rng;

const FIELD_T: f32 = 1.0;
const FIELD_B: f32 = 30.0;
const FIELD_L: f32 = 1.0;
const FIELD_R: f32 = 30.0;
const GOAL_TOP: f32 = 12.5;
const GOAL_BOT: f32 = 19.5;
const HALFWAY: f32 = 15.0;

const GRASS:  [u8; 3] = [18, 100, 22];
const LINE:   [u8; 3] = [200, 200, 200];
const NET:    [u8; 3] = [25,  25,  25];
const BALL:   [u8; 3] = [255, 220, 30];
const BLUE:   [u8; 3] = [40,  90, 230];
const GK_A:   [u8; 3] = [0,  210, 200];
const RED:    [u8; 3] = [220, 38,  28];
const GK_B:   [u8; 3] = [255, 50,  200];

#[derive(Copy, Clone)]
enum State {
    Playing,
    Goal { timer: f32, scorer: u8 },
}

struct Player {
    x: f32,
    y: f32,
    home_x: f32,
    home_y: f32,
    speed: f32,
    is_gk: bool,
    team: u8,
}

impl Player {
    fn new(x: f32, y: f32, speed: f32, is_gk: bool, team: u8) -> Self {
        Player { x, y, home_x: x, home_y: y, speed, is_gk, team }
    }

    fn reset(&mut self) { self.x = self.home_x; self.y = self.home_y; }

    fn step_toward(&mut self, tx: f32, ty: f32, dt: f32) {
        let dx = tx - self.x;
        let dy = ty - self.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > 0.3 {
            let mv = (self.speed * dt).min(dist);
            self.x += dx / dist * mv;
            self.y += dy / dist * mv;
        }
        self.x = self.x.clamp(FIELD_L + 0.5, FIELD_R - 0.5);
        self.y = self.y.clamp(FIELD_T + 0.5, FIELD_B - 0.5);
    }

    fn dist_to(&self, bx: f32, by: f32) -> f32 {
        ((bx - self.x).powi(2) + (by - self.y).powi(2)).sqrt()
    }
}

pub struct Soccer {
    ball_x: f32,
    ball_y: f32,
    ball_vx: f32,
    ball_vy: f32,
    // [0] A keeper, [1] A mid-def, [2] A mid-att, [3] B keeper, [4] B mid-def, [5] B mid-att
    players: Vec<Player>,
    score: [u32; 2],
    state: State,
    kick_cd: f32,
    speed: f32,
}

impl Soccer {
    pub fn new(speed: f32) -> Self {
        let players = vec![
            // Team A (attacks right): GK + defensive mid + attacking mid
            Player::new( 4.0, 15.5, 6.0, true,  0),
            Player::new(10.0, 10.0, 8.5, false, 0),
            Player::new(15.0, 21.0, 8.5, false, 0),
            // Team B (attacks left): GK + defensive mid + attacking mid
            Player::new(27.0, 15.5, 6.0, true,  1),
            Player::new(21.0, 10.0, 8.5, false, 1),
            Player::new(16.0, 21.0, 8.5, false, 1),
        ];
        let mut s = Soccer {
            ball_x: 15.5, ball_y: 15.5,
            ball_vx: 0.0, ball_vy: 0.0,
            players,
            score: [0, 0],
            state: State::Playing,
            kick_cd: 0.0,
            speed,
        };
        s.kickoff(0);
        s
    }

    fn kickoff(&mut self, team: u8) {
        self.ball_x = 15.5;
        self.ball_y = 15.5;
        let mut rng = rand::thread_rng();
        let angle: f32 = rng.gen_range(-0.4f32..0.4f32);
        let dir = if team == 0 { 1.0f32 } else { -1.0 };
        let spd = 7.0 + rng.gen_range(0.0f32..2.0f32);
        self.ball_vx = dir * spd * angle.cos();
        self.ball_vy = spd * angle.sin();
        self.kick_cd = 0.5;
        for p in &mut self.players { p.reset(); }
    }

    // Predict where ball will cross player's x column, clamped to field height.
    // Returns None if ball is moving away or crossing is too far in the future.
    fn intercept_y(bx: f32, by: f32, bvx: f32, bvy: f32, px: f32) -> Option<f32> {
        if bvx.abs() < 0.5 { return None; }
        let t = (px - bx) / bvx;
        if t > 0.05 && t < 1.8 {
            Some((by + bvy * t).clamp(FIELD_T, FIELD_B))
        } else {
            None
        }
    }

    fn draw_field(canvas: &mut Canvas) {
        canvas.fill(GRASS);
        // Goal nets
        for y in 12..=19usize { canvas.set(0, y, NET); canvas.set(31, y, NET); }
        // Touchlines
        for x in 0..32usize { canvas.set(x, 0, LINE); canvas.set(x, 31, LINE); }
        // Side walls with goal gap
        for y in 0..32usize {
            if y < 12 || y > 19 { canvas.set(0, y, LINE); canvas.set(31, y, LINE); }
        }
        // Goal posts
        canvas.set(0, 12, LINE); canvas.set(0, 19, LINE);
        canvas.set(31, 12, LINE); canvas.set(31, 19, LINE);
        // Halfway line
        for y in 1..=30usize { canvas.set(15, y, LINE); }
        // Centre circle (r = 4.5, sampled at 72 points)
        for i in 0..72u32 {
            let a = i as f32 * std::f32::consts::TAU / 72.0;
            let x = (15.5 + 4.5 * a.cos()).round() as i32;
            let y = (15.5 + 4.5 * a.sin()).round() as i32;
            if x > 0 && x < 31 && y > 0 && y < 31 {
                canvas.set(x as usize, y as usize, LINE);
            }
        }
        canvas.set(15, 15, LINE); canvas.set(16, 15, LINE);
        // Goal area boxes
        for y in 10..=21usize { canvas.set(5, y, LINE); canvas.set(26, y, LINE); }
        for x in 1..=5usize  { canvas.set(x, 10, LINE); canvas.set(x, 21, LINE); }
        for x in 26..=30usize { canvas.set(x, 10, LINE); canvas.set(x, 21, LINE); }
    }

    fn blit(canvas: &mut Canvas, x: f32, y: f32, col: [u8; 3], size: i32) {
        let xi = x.round() as i32;
        let yi = y.round() as i32;
        for dy in 0..size {
            for dx in 0..size {
                let (px, py) = (xi + dx, yi + dy);
                if px >= 0 && px < 32 && py >= 0 && py < 32 {
                    canvas.set(px as usize, py as usize, col);
                }
            }
        }
    }
}

impl Animation for Soccer {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        let dt = dt.min(0.1) * self.speed;

        // ── Goal celebration ──────────────────────────────────────────────────
        if let State::Goal { timer, scorer } = self.state {
            let remaining = timer - dt;
            if remaining <= 0.0 {
                self.kickoff(1 - scorer);
                self.state = State::Playing;
            } else {
                self.state = State::Goal { timer: remaining, scorer };
                let mut canvas = Canvas::black();
                let col = if scorer == 0 { BLUE } else { RED };
                let gw = text_width("GOAL!", 1) as i32;
                draw_text(&mut canvas, "GOAL!", (32 - gw) / 2, 8, col, 1);
                let s = format!("{} - {}", self.score[0], self.score[1]);
                let sw = text_width(&s, 1) as i32;
                draw_text(&mut canvas, &s, (32 - sw) / 2, 18, LINE, 1);
                return canvas;
            }
        }

        // ── Physics ───────────────────────────────────────────────────────────
        self.kick_cd = (self.kick_cd - dt).max(0.0);
        self.ball_x += self.ball_vx * dt;
        self.ball_y += self.ball_vy * dt;
        let friction = 0.97f32.powf(dt * 30.0);
        self.ball_vx *= friction;
        self.ball_vy *= friction;

        if self.ball_y < FIELD_T { self.ball_y = FIELD_T; self.ball_vy =  self.ball_vy.abs(); }
        if self.ball_y > FIELD_B { self.ball_y = FIELD_B; self.ball_vy = -self.ball_vy.abs(); }

        if self.ball_x < FIELD_L {
            if self.ball_y >= GOAL_TOP && self.ball_y <= GOAL_BOT {
                self.score[1] += 1;
                self.state = State::Goal { timer: 2.0, scorer: 1 };
            } else {
                self.ball_x = FIELD_L; self.ball_vx = self.ball_vx.abs() * 0.8;
            }
        }
        if self.ball_x > FIELD_R {
            if self.ball_y >= GOAL_TOP && self.ball_y <= GOAL_BOT {
                self.score[0] += 1;
                self.state = State::Goal { timer: 2.0, scorer: 0 };
            } else {
                self.ball_x = FIELD_R; self.ball_vx = -self.ball_vx.abs() * 0.8;
            }
        }

        let bx = self.ball_x;
        let by = self.ball_y;
        let bvx = self.ball_vx;
        let bvy = self.ball_vy;

        // ── Find chasers ──────────────────────────────────────────────────────
        let mut chaser_a = 1usize;
        let mut chaser_b = 4usize;
        {
            let (mut ma, mut mb) = (f32::MAX, f32::MAX);
            for (i, p) in self.players.iter().enumerate() {
                if p.is_gk { continue; }
                let d = p.dist_to(bx, by);
                if p.team == 0 && d < ma { ma = d; chaser_a = i; }
                if p.team == 1 && d < mb { mb = d; chaser_b = i; }
            }
        }

        // ── Movement targets ──────────────────────────────────────────────────
        let targets: Vec<(f32, f32)> = self.players.iter().enumerate().map(|(i, p)| {
            if p.is_gk {
                // Keeper: stay home x, track ball y across full goal
                (p.home_x, by.clamp(GOAL_TOP, GOAL_BOT))
            } else if i == chaser_a || i == chaser_b {
                // Chaser: intercept prediction — slide to where ball will cross our column
                match Soccer::intercept_y(bx, by, bvx, bvy, p.x) {
                    Some(iy) => (p.x, iy),   // ball coming to us: slide into path
                    None     => (bx, by),    // ball going away: chase directly
                }
            } else {
                // Non-chaser: support run — push forward when ball is deep in attack
                let sp_x = if p.team == 0 {
                    if bx < HALFWAY { p.home_x }
                    else { (bx - 7.0).max(p.home_x).min(FIELD_R - 2.0) }
                } else {
                    if bx > HALFWAY { p.home_x }
                    else { (bx + 7.0).min(p.home_x).max(FIELD_L + 2.0) }
                };
                (sp_x, p.home_y)
            }
        }).collect();

        for (i, p) in self.players.iter_mut().enumerate() {
            p.step_toward(targets[i].0, targets[i].1, dt);
        }

        // ── Kick / pass logic ─────────────────────────────────────────────────
        if self.kick_cd <= 0.0 {
            let closest_idx = self.players.iter().enumerate()
                .min_by(|(_, a), (_, b)|
                    a.dist_to(bx, by).partial_cmp(&b.dist_to(bx, by)).unwrap())
                .map(|(i, _)| i);

            if let Some(idx) = closest_idx {
                let kicker = &self.players[idx];
                let kicker_team = kicker.team;
                let kicker_is_gk = kicker.is_gk;
                let kx = kicker.x;
                let ky = kicker.y;
                let kick_r = if kicker_is_gk { 2.5 } else { 2.0 };

                if kicker.dist_to(bx, by) < kick_r {
                    let mut rng = rand::thread_rng();
                    let goal_x = if kicker_team == 0 { FIELD_R + 1.0 } else { FIELD_L - 1.0 };
                    let dist_to_goal = (goal_x - bx).abs();

                    let (tx, ty, spd) = if kicker_is_gk {
                        // GK distributes short to nearest outfield teammate
                        let teammate = self.players.iter().enumerate()
                            .filter(|(j, c)| *j != idx && c.team == kicker_team && !c.is_gk)
                            .min_by(|(_, a), (_, b)|
                                a.dist_to(kx, ky).partial_cmp(&b.dist_to(kx, ky)).unwrap())
                            .map(|(_, c)| (c.x, c.y));
                        if let Some((tx, ty)) = teammate {
                            (tx + rng.gen_range(-1.0f32..1.0f32),
                             ty + rng.gen_range(-1.0f32..1.0f32),
                             8.0 + rng.gen_range(0.0f32..2.0f32))
                        } else {
                            // No outfield player — clear forward
                            let cx = kx + if kicker_team == 0 { 8.0 } else { -8.0 };
                            (cx, 15.5 + rng.gen_range(-3.0f32..3.0f32), 9.0)
                        }
                    } else if dist_to_goal <= 11.0 {
                        // Shooting range — aim at goal with tight spread
                        (goal_x, 15.5 + rng.gen_range(-4.0f32..4.0f32),
                         12.0 + rng.gen_range(0.0f32..3.0f32))
                    } else {
                        // Look for a more advanced teammate to pass to
                        let is_fwd = |cx: f32| {
                            if kicker_team == 0 { cx > kx + 2.0 } else { cx < kx - 2.0 }
                        };
                        let best = self.players.iter().enumerate()
                            .filter(|(j, c)|
                                *j != idx && c.team == kicker_team && !c.is_gk && is_fwd(c.x))
                            .min_by(|(_, a), (_, b)|
                                (goal_x - a.x).abs().partial_cmp(&(goal_x - b.x).abs()).unwrap())
                            .map(|(_, c)| (c.x, c.y));

                        if let Some((tx, ty)) = best {
                            // Pass to most advanced teammate
                            (tx + rng.gen_range(-1.0f32..1.0f32),
                             ty + rng.gen_range(-1.5f32..1.5f32),
                             8.0 + rng.gen_range(0.0f32..2.0f32))
                        } else {
                            // No pass option — drive toward goal
                            (goal_x, 15.5 + rng.gen_range(-5.0f32..5.0f32),
                             11.0 + rng.gen_range(0.0f32..3.0f32))
                        }
                    };

                    let dx = tx - bx;
                    let dy = ty - by;
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    self.ball_vx = dx / d * spd;
                    self.ball_vy = dy / d * spd;
                    self.kick_cd = 0.3;
                }
            }
        }

        // ── Nudge stalled ball ────────────────────────────────────────────────
        if self.ball_vx.abs() + self.ball_vy.abs() < 0.8 {
            let mut rng = rand::thread_rng();
            let team = self.players.iter()
                .min_by(|a, b| a.dist_to(bx, by).partial_cmp(&b.dist_to(bx, by)).unwrap())
                .map(|p| p.team).unwrap_or(0);
            let dir = if team == 0 { 1.0f32 } else { -1.0 };
            self.ball_vx = dir * 5.0 + rng.gen_range(-2.0f32..2.0f32);
            self.ball_vy = rng.gen_range(-2.5f32..2.5f32);
        }

        // ── Render ────────────────────────────────────────────────────────────
        let mut canvas = Canvas::black();
        Soccer::draw_field(&mut canvas);

        for p in &self.players {
            let col = match (p.team, p.is_gk) {
                (0, false) => BLUE, (0, true) => GK_A,
                (1, false) => RED,  _          => GK_B,
            };
            Soccer::blit(&mut canvas, p.x - 0.5, p.y - 0.5, col, 2);
        }
        Soccer::blit(&mut canvas, self.ball_x, self.ball_y, BALL, 1);

        // Score corners
        draw_text(&mut canvas, &self.score[0].to_string(), 2, 2, GK_A, 1);
        let sb = self.score[1].to_string();
        let tw = text_width(&sb, 1) as i32;
        draw_text(&mut canvas, &sb, 29 - tw, 2, GK_B, 1);

        canvas
    }
}
