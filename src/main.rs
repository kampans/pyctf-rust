// main.rs - entry point for pyCTF (Rust/Macroquad port)
mod game;

use game::{draw_label, Game, COLOR_BLUE, COLOR_RED, HEIGHT, WIDTH};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "pyCTF - The RUST port".to_string(),
        window_width: 1600,
        window_height: 900,
        fullscreen: false,
        window_resizable: false,
        ..Default::default()
    }
}

/// Which screen is showing.
enum GameState {
    Lobby,
    Playing,
}

const P1_INPUT: Rect = Rect { x: 400.0, y: 280.0, w: 300.0, h: 48.0 };
const P2_INPUT: Rect = Rect { x: 900.0, y: 280.0, w: 300.0, h: 48.0 };
const NAME_MAX_CHARS: usize = 12;

struct App {
    game: Game,
    p1_keys: [bool; 6],
    p2_keys: [bool; 6],
    /// Debug overlay (key 9).
    show_hitboxes: bool,
    state: GameState,
    p1_name: String,
    p2_name: String,
    /// Which name box has focus: None, Some(0) = P1, Some(1) = P2.
    focused_field: Option<usize>,
    cursor_timer: f32,
    start_button_rect: Rect,
    start_button_hovered: bool,
}

fn point_in(r: &Rect, x: f32, y: f32) -> bool {
    x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h
}

/// Load a texture, or a plain coloured placeholder if the file is missing.
async fn load_or_placeholder(path: &str, w: u16, h: u16, color: Color) -> Texture2D {
    match load_texture(path).await {
        Ok(tex) => tex,
        Err(e) => {
            eprintln!("Could not load {path}: {e:?} - using a placeholder");
            Texture2D::from_image(&Image::gen_image_color(w, h, color))
        }
    }
}

fn draw_centered(text: &str, center_x: f32, y: f32, size: u16, color: Color, font: Option<&Font>) {
    let dims = measure_text(text, font, size, 1.0);
    draw_label(text, center_x - dims.width / 2.0, y, size, color, font);
}

impl App {
    async fn new() -> Self {
        let red_flag = load_or_placeholder("assets/red_flag_refined_45x45.png", 360, 45, COLOR_RED).await;
        let blue_flag = load_or_placeholder("assets/blue_flag_refined_45x45.png", 360, 45, COLOR_BLUE).await;
        let background =
            load_or_placeholder("assets/background.png", 1600, 900, Color::new(0.1, 0.2, 0.3, 1.0)).await;
        // Sprite sheets are 144x144 (6 columns x 6 rows of 24x24 frames).
        let red_player = load_or_placeholder("assets/player_red.png", 144, 144, COLOR_RED).await;
        let blue_player = load_or_placeholder("assets/player_blue.png", 144, 144, COLOR_BLUE).await;

        // Keep pixel art crisp when scaled up.
        red_flag.set_filter(FilterMode::Nearest);
        blue_flag.set_filter(FilterMode::Nearest);
        red_player.set_filter(FilterMode::Nearest);
        blue_player.set_filter(FilterMode::Nearest);

        // Optional custom font; if the file is missing macroquad's built-in font is used.
        let font = load_ttf_font("assets/DejaVuSans.ttf").await.ok();

        let start_button_rect = Rect::new(WIDTH / 2.0 - 120.0, 700.0, 240.0, 60.0);

        Self {
            game: Game::new(background, red_flag, blue_flag, red_player, blue_player, font),
            p1_keys: [false; 6],
            p2_keys: [false; 6],
            show_hitboxes: false,
            state: GameState::Lobby,
            p1_name: "Player1".to_string(),
            p2_name: "Player2".to_string(),
            focused_field: None,
            cursor_timer: 0.0,
            start_button_rect,
            start_button_hovered: false,
        }
    }

    fn update(&mut self) {
        let dt = get_frame_time();
        match self.state {
            GameState::Lobby => self.update_lobby(dt),
            GameState::Playing => self.update_playing(dt),
        }
    }

    fn update_lobby(&mut self, dt: f32) {
        self.cursor_timer += dt;
        if self.cursor_timer >= 0.5 {
            self.cursor_timer = 0.0;
        }

        let (mx, my) = mouse_position();
        self.start_button_hovered = point_in(&self.start_button_rect, mx, my);

        // Mouse: focus a name box, or press the start button.
        if is_mouse_button_pressed(MouseButton::Left) {
            if point_in(&P1_INPUT, mx, my) {
                self.focused_field = Some(0);
            } else if point_in(&P2_INPUT, mx, my) {
                self.focused_field = Some(1);
            } else if self.start_button_hovered {
                self.start_game();
                return;
            } else {
                self.focused_field = None;
            }
        }

        // Tab switches between the two name boxes.
        if is_key_pressed(KeyCode::Tab) {
            self.focused_field = match self.focused_field {
                Some(0) => Some(1),
                _ => Some(0),
            };
        }

        // Typing into the focused name box.
        if let Some(field) = self.focused_field {
            let name = if field == 0 { &mut self.p1_name } else { &mut self.p2_name };

            while let Some(c) = get_char_pressed() {
                if (c.is_alphanumeric() || c == ' ' || c == '-' || c == '_')
                    && name.chars().count() < NAME_MAX_CHARS
                {
                    name.push(c);
                }
            }

            if is_key_pressed(KeyCode::Backspace) {
                name.pop();
            }
        } else {
            // Throw away typed characters when no box is focused.
            while get_char_pressed().is_some() {}
        }

        // Enter always starts. Space starts only when you are not typing a name.
        if is_key_pressed(KeyCode::Enter)
            || (self.focused_field.is_none() && is_key_pressed(KeyCode::Space))
        {
            self.start_game();
        }
    }

    /// Apply the lobby names, reset the match and switch to the game screen.
    fn start_game(&mut self) {
        let p1 = self.p1_name.trim().to_string();
        let p2 = self.p2_name.trim().to_string();
        self.game.players[0].name = if p1.is_empty() { "Player1".to_string() } else { p1 };
        self.game.players[1].name = if p2.is_empty() { "Player2".to_string() } else { p2 };
        self.game.reset();
        self.state = GameState::Playing;
        self.focused_field = None;
    }

    fn update_playing(&mut self, _dt: f32) {
        // [left, right, jump, fall through, dash left, dash right]
        self.p1_keys = [
            is_key_down(KeyCode::A),
            is_key_down(KeyCode::D),
            is_key_down(KeyCode::W),
            is_key_down(KeyCode::S),
            is_key_down(KeyCode::Q),
            is_key_down(KeyCode::E),
        ];
        self.p2_keys = [
            is_key_down(KeyCode::J),
            is_key_down(KeyCode::L),
            is_key_down(KeyCode::I),
            is_key_down(KeyCode::K),
            is_key_down(KeyCode::U),
            is_key_down(KeyCode::O),
        ];

        self.game.move_player(0, &self.p1_keys);
        self.game.move_player(1, &self.p2_keys);

        // Hitboxes, void death, scoring, animation, round timer.
        self.game.update(get_frame_time());

        if is_key_pressed(KeyCode::Key9) {
            self.show_hitboxes = !self.show_hitboxes;
        }
    }

    fn draw(&self) {
        clear_background(Color::new(25.0 / 255.0, 51.0 / 255.0, 7.0 / 255.0, 1.0));
        match self.state {
            GameState::Lobby => self.draw_lobby(),
            GameState::Playing => self.draw_playing(),
        }
    }

    fn draw_playing(&self) {
        self.game.draw(self.show_hitboxes);
        self.game.draw_ui(!is_key_down(KeyCode::Tab));
    }

    /// One name input box (background, border, text and blinking cursor).
    fn draw_name_box(&self, rect: &Rect, text: &str, focused: bool, accent: Color, font: Option<&Font>) {
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0.15, 0.15, 0.2, 1.0));
        let border = if focused { accent } else { Color::new(0.5, 0.5, 0.5, 1.0) };
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, border);

        let cursor = if focused && self.cursor_timer < 0.25 { "_" } else { "" };
        let shown = format!("{}{}", text, cursor);
        let dims = measure_text(&shown, font, 28, 1.0);
        draw_label(&shown, rect.x + 12.0, rect.y + rect.h / 2.0 + dims.height / 2.0 - 2.0, 28, WHITE, font);
    }

    fn draw_lobby(&self) {
        let center_x = WIDTH / 2.0;
        let font = self.game.font.as_ref();

        draw_centered("pyCTF - The RUST port", center_x, 120.0, 64, WHITE, font);

        // ---- Name inputs ----
        draw_centered("PLAYER 1 (RED)", P1_INPUT.x + P1_INPUT.w / 2.0, 250.0, 28, COLOR_RED, font);
        self.draw_name_box(&P1_INPUT, &self.p1_name, self.focused_field == Some(0), COLOR_RED, font);

        draw_centered("PLAYER 2 (BLUE)", P2_INPUT.x + P2_INPUT.w / 2.0, 250.0, 28, COLOR_BLUE, font);
        self.draw_name_box(&P2_INPUT, &self.p2_name, self.focused_field == Some(1), COLOR_BLUE, font);

        // ---- Controls table ----
        let table_y = 420.0;
        let col1_x = 450.0;
        let col2_x = 1050.0;
        let row_height = 30.0;

        draw_label("PLAYER 1 (RED)", col1_x, table_y, 24, COLOR_RED, font);
        draw_label("PLAYER 2 (BLUE)", col2_x, table_y, 24, COLOR_BLUE, font);

        let controls = [
            ("Move", "A / D", "J / L"),
            ("Jump", "W", "I"),
            ("Fall through", "S", "K"),
            ("Dash left", "Q", "U"),
            ("Dash right", "E", "O"),
        ];

        for (i, (action, p1_key, p2_key)) in controls.iter().enumerate() {
            let y = table_y + 35.0 + i as f32 * row_height;
            draw_label(action, col1_x - 160.0, y, 20, Color::new(0.8, 0.8, 0.8, 1.0), font);
            draw_label(p1_key, col1_x, y, 20, COLOR_RED, font);
            draw_label(p2_key, col2_x, y, 20, COLOR_BLUE, font);
        }

        // ---- Start button ----
        let r = &self.start_button_rect;
        let btn_color = if self.start_button_hovered {
            Color::new(0.2, 0.5, 0.8, 1.0)
        } else {
            Color::new(0.1, 0.3, 0.5, 1.0)
        };
        draw_rectangle(r.x, r.y, r.w, r.h, btn_color);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, WHITE);
        let btn_dims = measure_text("START GAME", font, 32, 1.0);
        draw_label(
            "START GAME",
            r.x + r.w / 2.0 - btn_dims.width / 2.0,
            r.y + r.h / 2.0 + btn_dims.height / 2.0 - 2.0,
            32,
            WHITE,
            font,
        );

        draw_centered(
            "[ENTER] to start (or [SPACE] when no name box is selected)",
            center_x,
            r.y + r.h + 25.0,
            18,
            Color::new(0.6, 0.6, 0.6, 1.0),
            font,
        );

        draw_centered(
            "Click a name box to edit  -  Tab to switch  -  Max 12 characters",
            center_x,
            HEIGHT - 50.0,
            16,
            Color::new(0.5, 0.5, 0.5, 1.0),
            font,
        );
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::new().await;
    loop {
        app.update();
        app.draw();
        next_frame().await;
    }
}