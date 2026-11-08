// game.rs - entities, physics, collision and scoring for pyCTF (Rust/Macroquad port)
use macroquad::prelude::*;

pub const WIDTH: f32 = 1600.0;
pub const HEIGHT: f32 = 900.0;

/// Physics constants (tuned for ~60 FPS, matching the original clock.tick(60)).
pub const ACC: f32 = 0.9;
pub const FRICT: f32 = -0.12;
pub const GRAVITY: f32 = 0.5;

/// Sizes
pub const PLAYER_SIZE: f32 = 42.0;
pub const HITBOX_SIZE: f32 = 45.0;
pub const FLAG_SIZE: f32 = 45.0;
pub const COIN_SIZE: f32 = 30.0;

/// Sprite sheet: 24x24 frames, 6 columns, 6 rows (player_red.png / player_blue.png are 144x144).
pub const FRAME_W: f32 = 24.0;
pub const FRAME_H: f32 = 24.0;
pub const ANIM_SPEED: f32 = 0.1; // seconds per frame

/// Flag sprite sheet: 45x45 frames, single row of 8 frames (360x45 total).
pub const FLAG_FRAME_W: f32 = 45.0;
pub const FLAG_FRAME_H: f32 = 45.0;
pub const FLAG_FRAME_COUNT: usize = 8;
pub const FLAG_ANIM_SPEED: f32 = 0.1; // seconds per frame

/// Coin sprite sheet: five 16x16 frames in a single row (80x16 total).
pub const COIN_FRAME_SIZE: f32 = 16.0;
pub const COIN_FRAME_COUNT: usize = 5;

/// Animation rows in the sprite sheet.
pub const ROW_IDLE: usize = 0;
pub const ROW_WALK: usize = 1;
pub const ROW_RUN: usize = 2;
pub const ROW_JUMP: usize = 3;
pub const ROW_FALL: usize = 4;
pub const ROW_DASH: usize = 5;

/// Number of frames used in each row.
pub const FRAME_COUNT: [usize; 6] = [4, 6, 6, 3, 2, 3];

/// Round / scoring settings - change these to tune the game.
pub const ROUND_SECONDS: f32 = 120.0;
pub const SCORE_FLAG: u32 = 10; // attacker captures the flag
pub const SCORE_TAG: u32 = 10; // defender catches the attacker
pub const SCORE_VOID: u32 = 5; // opponent falls into the void

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    CaptureTheFlag,
    CoinRace,
}

impl GameMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::CaptureTheFlag => "CAPTURE THE FLAG",
            Self::CoinRace => "COIN RACE",
        }
    }

    pub fn toggle(&mut self) {
        *self = match self {
            Self::CaptureTheFlag => Self::CoinRace,
            Self::CoinRace => Self::CaptureTheFlag,
        };
    }
}

pub const COIN_MIN_SPAWN_DISTANCE: f32 = 400.0;

fn advance_match_clock(timer: &mut f32, dt: f32) -> bool {
    *timer = (*timer - dt).max(0.0);
    *timer == 0.0
}

/// Intersect a circle around the coin with safe standing positions on platforms.
fn race_spawn_pairs(platforms: &[Platform], coin: Vec2) -> Vec<[Vec2; 2]> {
    let mut pairs = Vec::new();
    for radius in (COIN_MIN_SPAWN_DISTANCE as u32..=1400).step_by(25) {
        let radius = radius as f32;
        let mut candidates = Vec::new();
        for platform in platforms {
            let y = platform.rect.y + 1.0;
            let dx_squared = radius * radius - (y - coin.y).powi(2);
            if dx_squared < 0.0 {
                continue;
            }
            for x in [coin.x - dx_squared.sqrt(), coin.x + dx_squared.sqrt()] {
                if x >= platform.rect.x + HITBOX_SIZE / 2.0
                    && x <= platform.rect.right() - HITBOX_SIZE / 2.0
                {
                    candidates.push(vec2(x, y));
                }
            }
        }
        for (i, &a) in candidates.iter().enumerate() {
            for &b in &candidates[i + 1..] {
                if a.distance(b) >= COIN_MIN_SPAWN_DISTANCE {
                    pairs.push([a, b]);
                }
            }
        }
    }
    pairs
}

/// Colours.
pub const COLOR_RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
pub const COLOR_BLUE: Color = Color { r: 77.0 / 255.0, g: 109.0 / 255.0, b: 242.0 / 255.0, a: 1.0 };
pub const COLOR_DARK_GREEN: Color = Color { r: 0.0, g: 100.0 / 255.0, b: 0.0, a: 1.0 };

/// Key order used everywhere: [left, right, jump, fall_through_platform, dash_left, dash_right]
///
/// Player 1 (red): A D W S Q E
/// Player 2 (blue): J L I K U O
pub struct Player {
    /// Bottom-centre of the sprite.
    pub pos: Vec2,
    pub vel: Vec2,
    pub acc: Vec2,
    /// Shown above the player.
    pub name: String,
    pub jumping: bool,
    pub wooshing: bool,
    /// 45x45 hitbox, bottom-centre on `pos`.
    pub hitbox: Rect,
    /// Sprite sheet texture.
    pub texture: Texture2D,
    pub anim_row: usize,
    pub anim_frame: usize,
    pub anim_timer: f32,
    /// true = facing right.
    pub facing_right: bool,
    /// Seconds of spawn flash remaining (> 0 means currently spawning).
    pub spawn_timer: f32,
    pub spawn_color: Color,
}

impl Player {
    fn new(name: &str, texture: Texture2D, spawn_color: Color) -> Self {
        Player {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            acc: Vec2::ZERO,
            name: name.to_string(),
            jumping: false,
            wooshing: false,
            hitbox: Rect::new(0.0, 0.0, HITBOX_SIZE, HITBOX_SIZE),
            texture,
            anim_row: ROW_IDLE,
            anim_frame: 0,
            anim_timer: 0.0,
            facing_right: true,
            spawn_timer: 0.0,
            spawn_color,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Platform {
    pub rect: Rect,
}

pub struct Flag {
    /// Bottom-centre of the flag image.
    pub pos: Vec2,
    pub texture: Texture2D,
    /// Current animation frame index (0..FLAG_FRAME_COUNT).
    pub anim_frame: usize,
    /// Seconds accumulated toward the next animation frame.
    pub anim_timer: f32,
}

pub struct Game {
    pub mode: GameMode,
    pub coin: Flag,
    coin_spawn_timer: f32,
    race_spawns: [Vec2; 2],
    pub background: Texture2D,
    pub platforms: Vec<Platform>,
    /// Death zone at the bottom of the screen.
    pub void: Rect,
    pub players: [Player; 2],
    /// flags[0] belongs to player 1 (red), flags[1] to player 2 (blue).
    pub flags: [Flag; 2],
    /// Only the defender's flag is in play / drawn.
    pub active_flags: Vec<usize>,
    pub p1_score: u32,
    pub p2_score: u32,
    pub font: Option<Font>,
    /// Roles: one player attacks, the other defends their flag.
    pub attacker_idx: usize,
    pub defender_idx: usize,
    /// Hitboxes, recomputed every frame.
    pub p1_hitbox: Rect,
    pub p2_hitbox: Rect,
    pub f1_hitbox: Rect,
    pub f2_hitbox: Rect,
    /// Seconds left before roles swap.
    pub round_timer: f32,
    /// "ROLES REVERSED!" banner state.
    pub role_reversal_active: bool,
    pub role_reversal_timer: f32,
}

impl Game {
    pub fn new(
        background: Texture2D,
        red_flag: Texture2D,
        blue_flag: Texture2D,
        red_player: Texture2D,
        blue_player: Texture2D,
        coin: Texture2D,
        font: Option<Font>,
    ) -> Self {
        let platforms = arena_platforms();

        let void = Rect::new(0.0, HEIGHT - 30.0, WIDTH, 30.0);
        let empty = Rect::new(0.0, 0.0, HITBOX_SIZE, HITBOX_SIZE);

        let mut game = Game {
            mode: GameMode::CaptureTheFlag,
            coin: Flag { pos: Vec2::ZERO, texture: coin, anim_frame: 0, anim_timer: 0.0 },
            coin_spawn_timer: 0.0,
            race_spawns: [Vec2::ZERO; 2],
            background,
            platforms,
            void,
            players: [
                Player::new("Player1", red_player, COLOR_RED),
                Player::new("Player2", blue_player, COLOR_BLUE),
            ],
            flags: [
                Flag { pos: Vec2::ZERO, texture: red_flag, anim_frame: 0, anim_timer: 0.0 },
                Flag { pos: Vec2::ZERO, texture: blue_flag, anim_frame: 0, anim_timer: 0.0 },
            ],
            active_flags: vec![],
            p1_score: 0,
            p2_score: 0,
            font,
            attacker_idx: 0,
            defender_idx: 1,
            p1_hitbox: empty,
            p2_hitbox: empty,
            f1_hitbox: empty,
            f2_hitbox: empty,
            round_timer: ROUND_SECONDS,
            role_reversal_active: false,
            role_reversal_timer: 0.0,
        };

        game.reset();
        game
    }

    /// Start a fresh match: scores, roles, flag, spawns and timer.
    pub fn reset(&mut self) {
        self.p1_score = 0;
        self.p2_score = 0;
        self.randomize_roles();
        self.reset_round();
        self.round_timer = ROUND_SECONDS;
        self.role_reversal_active = false;
        self.role_reversal_timer = 0.0;
    }

    /// Put the flag and both players back in their starting spots.
    /// The flag goes first so the defender can spawn near it.
    fn reset_round(&mut self) {
        if self.mode == GameMode::CoinRace {
            self.respawn_coin();
            self.respawn(0);
            self.respawn(1);
            return;
        }
        self.respawn_flags();
        self.respawn(0);
        self.respawn(1);
    }

    /// Advance one player by one frame using its six keys.
    pub fn move_player(&mut self, idx: usize, keys: &[bool; 6]) {
        let player = &mut self.players[idx];

        let mut motion = crate::motion::Motion::from(&*player);
        motion.step(keys, &self.platforms);
        player.pos = motion.pos;
        player.vel = motion.vel;
        player.acc = motion.acc;
        player.jumping = motion.jumping;
        player.wooshing = motion.wooshing;
        player.hitbox = hitbox_at(player.pos);
    }

    /// Pick the animation row / frame / facing from the player's motion.
    fn update_player_animation(&mut self, idx: usize, dt: f32) {
        let player = &mut self.players[idx];

        let row = if player.wooshing {
            ROW_DASH
        } else if player.vel.y < -1.0 {
            ROW_JUMP
        } else if player.vel.y > 1.0 {
            ROW_FALL
        } else if player.vel.x.abs() > 5.0 {
            ROW_RUN
        } else if player.vel.x.abs() > 0.5 {
            ROW_WALK
        } else {
            ROW_IDLE
        };

        if player.vel.x > 0.5 {
            player.facing_right = true;
        } else if player.vel.x < -0.5 {
            player.facing_right = false;
        }

        if row != player.anim_row {
            player.anim_row = row;
            player.anim_frame = 0;
            player.anim_timer = 0.0;
        } else {
            player.anim_timer += dt;
            if player.anim_timer >= ANIM_SPEED {
                player.anim_timer -= ANIM_SPEED;
                player.anim_frame = (player.anim_frame + 1) % FRAME_COUNT[player.anim_row];
            }
        }
    }

    /// Everything that happens after both players have moved this frame.
    pub fn update(&mut self, dt: f32) {
        // Coin matches end at zero; leave the final arena and scores intact.
        if self.mode == GameMode::CoinRace
            && advance_match_clock(&mut self.round_timer, dt)
        {
            return;
        }
        self.p1_hitbox = hitbox_at(self.players[0].pos);
        self.p2_hitbox = hitbox_at(self.players[1].pos);
        self.f1_hitbox = hitbox_at(self.flags[0].pos);
        self.f2_hitbox = hitbox_at(self.flags[1].pos);

        // Void death.
        if rects_overlap(&self.p1_hitbox, &self.void) {
            self.p2_score += SCORE_VOID;
            self.respawn(0);
        }
        if rects_overlap(&self.p2_hitbox, &self.void) {
            self.p1_score += SCORE_VOID;
            self.respawn(1);
        }

        self.p1_hitbox = self.players[0].hitbox;
        self.p2_hitbox = self.players[1].hitbox;

        // Flag capture / tag.
        self.check_scoring();

        // Animation.
        self.update_player_animation(0, dt);
        self.update_player_animation(1, dt);

        // Flag animation.
        for flag in self.flags.iter_mut() {
            flag.anim_timer += dt;
            if flag.anim_timer >= FLAG_ANIM_SPEED {
                flag.anim_timer -= FLAG_ANIM_SPEED;
                flag.anim_frame = (flag.anim_frame + 1) % FLAG_FRAME_COUNT;
            }
        }
        self.coin.anim_timer += dt;
        if self.coin.anim_timer >= FLAG_ANIM_SPEED {
            self.coin.anim_timer %= FLAG_ANIM_SPEED;
            self.coin.anim_frame = (self.coin.anim_frame + 1) % COIN_FRAME_COUNT;
        }

        // Spawn flash timers.
        self.coin_spawn_timer = (self.coin_spawn_timer - dt).max(0.0);
        for p in self.players.iter_mut() {
            if p.spawn_timer > 0.0 {
                p.spawn_timer -= dt;
            }
        }

        // Round timer and role swap.
        if self.mode == GameMode::CoinRace {
            return;
        }
        if self.role_reversal_active {
            self.role_reversal_timer -= dt;
            if self.role_reversal_timer <= 0.0 {
                self.role_reversal_active = false;
                self.role_reversal_timer = 0.0;
            }
        } else {
            self.round_timer -= dt;
            if self.round_timer <= 0.0 {
                self.round_timer = ROUND_SECONDS;
                self.swap_roles();
                self.role_reversal_active = true;
                self.role_reversal_timer = 2.0; // banner shows for 2 seconds
            }
        }
    }

    pub fn match_finished(&self) -> bool {
        self.mode == GameMode::CoinRace && self.round_timer <= 0.0
    }

    /// Attacker touches the defender's flag -> flag score.
    /// Attacker touches the defender -> tag score for the defender.
    fn check_scoring(&mut self) {
        if self.mode == GameMode::CoinRace {
            let coin_hitbox = self.coin_hitbox();
            let touches = [
                rects_overlap(&self.p1_hitbox, &coin_hitbox),
                rects_overlap(&self.p2_hitbox, &coin_hitbox),
            ];
            // A simultaneous pickup is a tie, so neither player gets priority.
            if touches[0] || touches[1] {
                if touches[0] && !touches[1] { self.p1_score += SCORE_FLAG; }
                if touches[1] && !touches[0] { self.p2_score += SCORE_FLAG; }
                self.reset_round();
            }
            return;
        }
        let attacker_hitbox = if self.attacker_idx == 0 { self.p1_hitbox } else { self.p2_hitbox };
        let defender_hitbox = if self.defender_idx == 0 { self.p1_hitbox } else { self.p2_hitbox };
        let defender_flag_hitbox = if self.defender_idx == 0 { self.f1_hitbox } else { self.f2_hitbox };

        if rects_overlap(&attacker_hitbox, &defender_flag_hitbox) {
            if self.attacker_idx == 0 {
                self.p1_score += SCORE_FLAG;
            } else {
                self.p2_score += SCORE_FLAG;
            }
            self.reset_round();
            return;
        }

        if rects_overlap(&attacker_hitbox, &defender_hitbox) {
            if self.defender_idx == 0 {
                self.p1_score += SCORE_TAG;
            } else {
                self.p2_score += SCORE_TAG;
            }
            self.reset_round();
        }
    }

    pub fn draw(&self, draw_hitboxes: bool) {
        // Background.
        draw_texture(&self.background, 0.0, 0.0, WHITE);

        // Platforms + void are only shown in debug mode (key 9).
        if draw_hitboxes {
            for p in &self.platforms {
                draw_rectangle(p.rect.x, p.rect.y, p.rect.w, p.rect.h, COLOR_DARK_GREEN);
            }
            draw_rectangle(self.void.x, self.void.y, self.void.w, self.void.h, COLOR_DARK_GREEN);
        }

        // Flag (only the defender's is active).
        for &idx in &self.active_flags {
            let flag = &self.flags[idx];
            let source = Rect::new(
                flag.anim_frame as f32 * FLAG_FRAME_W,
                0.0,
                FLAG_FRAME_W,
                FLAG_FRAME_H,
            );
            draw_texture_ex(
                &flag.texture,
                flag.pos.x - FLAG_SIZE / 2.0,
                flag.pos.y - FLAG_SIZE,
                WHITE,
                DrawTextureParams {
                    source: Some(source),
                    dest_size: Some(vec2(FLAG_SIZE, FLAG_SIZE)),
                    ..Default::default()
                },
            );
        }

        // Players, spawn flash and names.
        if self.mode == GameMode::CoinRace {
            draw_texture_ex(
                &self.coin.texture,
                self.coin.pos.x - COIN_SIZE / 2.0,
                self.coin.pos.y - COIN_SIZE,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(self.coin.anim_frame as f32 * COIN_FRAME_SIZE, 0.0, COIN_FRAME_SIZE, COIN_FRAME_SIZE)),
                    dest_size: Some(vec2(COIN_SIZE, COIN_SIZE)),
                    ..Default::default()
                },
            );
            if self.coin_spawn_timer > 0.0 {
                let progress = (1.0 - self.coin_spawn_timer).clamp(0.0, 1.0);
                let flash = Color::new(GOLD.r, GOLD.g, GOLD.b, 1.0 - progress);
                let center = self.coin.pos - vec2(0.0, COIN_SIZE / 2.0);
                draw_circle_lines(center.x, center.y, progress * 80.0, 5.0, flash);
                draw_circle_lines(center.x, center.y, progress * 25.0, 3.0, flash);
            }
        }
        let font = self.font.as_ref();
        for p in self.players.iter() {
            let source = Rect::new(
                p.anim_frame as f32 * FRAME_W,
                p.anim_row as f32 * FRAME_H,
                FRAME_W,
                FRAME_H,
            );

            draw_texture_ex(
                &p.texture,
                p.pos.x - PLAYER_SIZE / 2.0,
                p.pos.y - PLAYER_SIZE,
                WHITE,
                DrawTextureParams {
                    source: Some(source),
                    dest_size: Some(vec2(PLAYER_SIZE, PLAYER_SIZE)),
                    flip_x: !p.facing_right,
                    ..Default::default()
                },
            );

            if p.spawn_timer > 0.0 {
                let progress = (1.0 - p.spawn_timer).clamp(0.0, 1.0);
                let alpha = 1.0 - progress;
                let flash = Color::new(p.spawn_color.r, p.spawn_color.g, p.spawn_color.b, alpha);
                let cx = p.pos.x;
                let cy = p.pos.y - PLAYER_SIZE / 2.0;
                draw_circle_lines(cx, cy, progress * 80.0, 5.0, flash);
                draw_circle_lines(cx, cy, progress * 25.0, 3.0, flash);
            }

            draw_label(&p.name, p.pos.x - 20.0, p.pos.y - PLAYER_SIZE - 12.0, 15, WHITE, font);
        }

        // Debug hitboxes.
        if draw_hitboxes {
            let objectives = if self.mode == GameMode::CoinRace {
                vec![self.coin_hitbox()]
            } else {
                self.active_flags.iter().map(|&i| hitbox_at(self.flags[i].pos)).collect()
            };
            for r in [self.p1_hitbox, self.p2_hitbox].into_iter().chain(objectives) {
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, WHITE);
            }
        }
    }

    /// HUD. `compact` = Tab released: just red score | timer | blue score.
    /// `compact` = false (Tab held): scores, roles and timer.
    pub fn draw_ui(&self, compact: bool) {
        let font = self.font.as_ref();
        let center_x = WIDTH / 2.0;

        let timer_secs = self.round_timer.max(0.0).ceil() as u32;
        let timer_text = format!("{:02}:{:02}", timer_secs / 60, timer_secs % 60);
        let timer_color = if timer_secs <= 10 { COLOR_RED } else { WHITE };

        if compact {
            let p1_text = format!("{}", self.p1_score);
            let p2_text = format!("{}", self.p2_score);
            let p1_dims = measure_text(&p1_text, font, 28, 1.0);
            let p2_dims = measure_text(&p2_text, font, 28, 1.0);
            let timer_dims = measure_text(&timer_text, font, 28, 1.0);

            let left = center_x - 90.0 - p1_dims.width / 2.0;
            let right = center_x + 90.0 + p2_dims.width / 2.0;
            draw_rectangle(left - 15.0, 20.0, (right - left) + 30.0, 54.0, Color::new(0.15, 0.15, 0.15, 0.75));

            draw_label(&p1_text, center_x - 90.0 - p1_dims.width / 2.0, 52.0, 28, COLOR_RED, font);
            draw_label(&timer_text, center_x - timer_dims.width / 2.0, 52.0, 28, timer_color, font);
            draw_label(&p2_text, center_x + 90.0 - p2_dims.width / 2.0, 52.0, 28, COLOR_BLUE, font);
        } else {
            let score_text = format!("P1: {}  |  P2: {}", self.p1_score, self.p2_score);
            let p1_role = if self.attacker_idx == 0 { "ATTACKER" } else { "DEFENDER" };
            let p2_role = if self.attacker_idx == 1 { "ATTACKER" } else { "DEFENDER" };
            let role_text = if self.mode == GameMode::CoinRace {
                "COIN RACE - First to the coin wins 10 points".to_string()
            } else { format!("P1: {}  |  P2: {}", p1_role, p2_role) };

            let score_dims = measure_text(&score_text, font, 36, 1.0);
            let role_dims = measure_text(&role_text, font, 24, 1.0);
            let timer_dims = measure_text(&timer_text, font, 32, 1.0);

            let max_w = score_dims.width.max(role_dims.width).max(timer_dims.width);
            draw_rectangle(
                center_x - max_w / 2.0 - 25.0,
                10.0,
                max_w + 50.0,
                145.0,
                Color::new(0.15, 0.15, 0.15, 0.8),
            );

            draw_label(&score_text, center_x - score_dims.width / 2.0, 50.0, 36, WHITE, font);
            draw_label(&role_text, center_x - role_dims.width / 2.0, 90.0, 24, YELLOW, font);
            draw_label(&timer_text, center_x - timer_dims.width / 2.0, 130.0, 32, timer_color, font);
        }

        // Role swap banner - always visible while active.
        if self.role_reversal_active {
            let text = "ROLES REVERSED!";
            let dims = measure_text(text, font, 64, 1.0);
            draw_label(text, center_x - dims.width / 2.0, HEIGHT / 2.0, 64, ORANGE, font);
        }
    }

    /// Respawn a player. The defender spawns on a platform near their flag,
    /// the attacker on a platform far from it.
    pub fn respawn(&mut self, idx: usize) {
        if self.mode == GameMode::CoinRace {
            self.place_player(idx, self.race_spawns[idx]);
            return;
        }
        let is_defender = idx == self.defender_idx;
        let flag_x = self.flags[self.defender_idx].pos.x;

        let chosen: Option<Platform> = if is_defender {
            self.platforms
                .iter()
                .filter(|p| (p.rect.x + p.rect.w / 2.0 - flag_x).abs() < 400.0 && p.rect.y < HEIGHT - 100.0)
                .max_by_key(|p| -(p.rect.y as i32))
                .copied()
        } else {
            self.platforms
                .iter()
                .filter(|p| (p.rect.x + p.rect.w / 2.0 - flag_x).abs() > 500.0 && p.rect.y < HEIGHT - 100.0)
                .min_by_key(|p| -(p.rect.y as i32))
                .copied()
        };

        let spawn_platform = chosen
            .or_else(|| self.platforms.iter().find(|p| p.rect.y < HEIGHT - 100.0).copied())
            .unwrap_or(self.platforms[0]);

        // Stay 20px away from the platform edges.
        let min_x = spawn_platform.rect.x + 20.0;
        let max_x = spawn_platform.rect.x + spawn_platform.rect.w - 20.0;
        let x = min_x + ::rand::random::<f32>() * (max_x - min_x);

        self.place_player(idx, vec2(x, spawn_platform.rect.y + 1.0));
    }

    fn place_player(&mut self, idx: usize, pos: Vec2) {
        let player = &mut self.players[idx];
        player.pos = pos;
        player.vel = Vec2::ZERO;
        player.acc = Vec2::ZERO;
        player.jumping = false;
        player.wooshing = false;
        player.spawn_timer = 1.0;
        player.spawn_color = if idx == 0 { COLOR_RED } else { COLOR_BLUE };
        player.hitbox = hitbox_at(player.pos);
        player.anim_row = ROW_IDLE;
        player.anim_frame = 0;
        player.anim_timer = 0.0;
    }

    fn coin_hitbox(&self) -> Rect {
        Rect::new(self.coin.pos.x - COIN_SIZE / 2.0, self.coin.pos.y - COIN_SIZE, COIN_SIZE, COIN_SIZE)
    }

    fn respawn_coin(&mut self) {
        self.active_flags.clear();
        self.respawn_flags();
        self.coin.pos = self.flags[self.defender_idx].pos;
        self.active_flags.clear();
        let pairs = race_spawn_pairs(&self.platforms, self.coin.pos);
        // Every upper-platform coin location in this arena has valid spawn pairs.
        let mut pair = pairs[::rand::random::<usize>() % pairs.len()];
        if ::rand::random::<bool>() { pair.swap(0, 1); }
        self.race_spawns = pair;
        self.coin.anim_frame = 0;
        self.coin.anim_timer = 0.0;
        self.coin_spawn_timer = 1.0;
    }

    /// Place the defender's flag on a random upper platform; only that flag is active.
    fn respawn_flags(&mut self) {
        const FLAG_SPAWN_MARGIN: f32 = 40.0; // keep the flag away from platform edges

        let upper: Vec<Platform> = self
            .platforms
            .iter()
            .filter(|p| p.rect.y < HEIGHT / 2.0)
            .copied()
            .collect();

        if !upper.is_empty() {
            let plat = upper[::rand::random::<usize>() % upper.len()];
            let min_x = plat.rect.x + FLAG_SPAWN_MARGIN;
            let max_x = plat.rect.x + plat.rect.w - FLAG_SPAWN_MARGIN;
            let spawn_x = if max_x > min_x {
                min_x + ::rand::random::<f32>() * (max_x - min_x)
            } else {
                plat.rect.x + plat.rect.w / 2.0
            };
            self.flags[self.defender_idx].pos = vec2(spawn_x, plat.rect.y);
        }

        self.active_flags = vec![self.defender_idx];
    }

    /// Randomly choose who attacks and who defends.
    pub fn randomize_roles(&mut self) {
        self.attacker_idx = if ::rand::random::<bool>() { 0 } else { 1 };
        self.defender_idx = 1 - self.attacker_idx;
    }

    /// Swap attacker/defender (used when the round timer runs out).
    pub fn swap_roles(&mut self) {
        std::mem::swap(&mut self.attacker_idx, &mut self.defender_idx);
        self.reset_round();
    }
}

/// Draw text with an optional custom font (None = macroquad's built-in font).
pub fn draw_label(text: &str, x: f32, y: f32, size: u16, color: Color, font: Option<&Font>) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font,
            font_size: size,
            color,
            ..Default::default()
        },
    );
}

/// 45x45 hitbox with its bottom-centre on `pos`.
fn hitbox_at(pos: Vec2) -> Rect {
    Rect::new(pos.x - HITBOX_SIZE / 2.0, pos.y - HITBOX_SIZE, HITBOX_SIZE, HITBOX_SIZE)
}

/// Axis-aligned rectangle overlap test.
fn rects_overlap(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

/// Platforms matched to background.png (all 20px thick).
pub fn arena_platforms() -> Vec<Platform> {
    vec![
        // Top row
        Platform { rect: Rect::new(245.0, 175.0, 270.0, 20.0) },
        Platform { rect: Rect::new(675.0, 160.0, 266.0, 20.0) },
        Platform { rect: Rect::new(1130.0, 175.0, 236.0, 20.0) },
        // Upper-middle row
        Platform { rect: Rect::new(458.0, 340.0, 222.0, 20.0) },
        Platform { rect: Rect::new(922.0, 340.0, 218.0, 20.0) },
        // Outer middle platforms
        Platform { rect: Rect::new(88.0, 417.0, 266.0, 20.0) },
        Platform { rect: Rect::new(1296.0, 427.0, 211.0, 20.0) },
        // Centre platform
        Platform { rect: Rect::new(680.0, 475.0, 247.0, 20.0) },
        // Lower-middle row
        Platform { rect: Rect::new(281.0, 570.0, 239.0, 20.0) },
        Platform { rect: Rect::new(1089.0, 575.0, 232.0, 20.0) },
        // Lower-centre platform
        Platform { rect: Rect::new(684.0, 660.0, 258.0, 20.0) },
        // Bottom row
        Platform { rect: Rect::new(78.0, 705.0, 500.0, 20.0) },
        Platform { rect: Rect::new(1040.0, 705.0, 479.0, 20.0) },
    ]
}

#[cfg(test)]
mod race_tests {
    use super::*;

    #[test]
    fn match_clock_stops_at_zero_including_large_frame_gaps() {
        let mut timer = ROUND_SECONDS;
        assert!(!advance_match_clock(&mut timer, 119.0));
        assert_eq!(timer, 1.0);
        assert!(advance_match_clock(&mut timer, 1.0));
        assert_eq!(timer, 0.0);
        assert!(advance_match_clock(&mut timer, 10.0));
        assert_eq!(timer, 0.0);
        timer = ROUND_SECONDS;
        assert!(advance_match_clock(&mut timer, 300.0));
        assert_eq!(timer, 0.0);
    }

    #[test]
    fn every_coin_location_has_safe_equidistant_spawns() {
        let platforms = arena_platforms();
        for platform in platforms.iter().filter(|p| p.rect.y < HEIGHT / 2.0) {
            let left = platform.rect.x + 40.0;
            let right = platform.rect.right() - 40.0;
            // Cover the full spawn range, including both edges, at subpixel intervals.
            for step in 0..=1000 {
                let coin = vec2(left + (right - left) * step as f32 / 1000.0, platform.rect.y);
                let pairs = race_spawn_pairs(&platforms, coin);
                assert!(!pairs.is_empty(), "no spawn pair for {coin:?}");
                for pair in pairs {
                    let [a, b] = pair;
                    assert!((a.distance(coin) - b.distance(coin)).abs() < 0.001);
                    assert!(a.distance(coin) >= COIN_MIN_SPAWN_DISTANCE - 0.001);
                    assert!(a.distance(b) >= COIN_MIN_SPAWN_DISTANCE);
                    for spawn in pair {
                        assert!(platforms.iter().any(|p| {
                            (spawn.y - p.rect.y - 1.0).abs() < 0.001
                                && spawn.x >= p.rect.x + HITBOX_SIZE / 2.0
                                && spawn.x <= p.rect.right() - HITBOX_SIZE / 2.0
                        }));
                        assert!(!rects_overlap(&hitbox_at(spawn), &hitbox_at(coin)));
                    }
                }
            }
        }
    }
}
