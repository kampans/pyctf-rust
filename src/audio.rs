//! Nonblocking audio playback, with music controls on menu screens.
use macroquad::audio::{load_sound_from_bytes, play_sound, play_sound_once, set_sound_volume, PlaySoundParams, Sound};
use macroquad::prelude::*;
use crate::platform::mouse_position;

const MUTE: Rect = Rect::new(1180.0, 28.0, 170.0, 36.0);
const SLIDER: Rect = Rect::new(1370.0, 28.0, 180.0, 36.0);

pub struct Audio {
    music: Option<Sound>,
    jump: Option<Sound>,
    dash: Option<Sound>,
    volume: f32,
    muted: bool,
    started: bool,
    dragging: bool,
    active: bool,
}

async fn load_optional(path: &str, bytes: &[u8]) -> Option<Sound> {
    match load_sound_from_bytes(bytes).await {
        Ok(sound) => Some(sound),
        Err(error) => {
            eprintln!("Could not load {path}: {error:?}");
            None
        }
    }
}

impl Audio {
    pub async fn new() -> Self {
        Self {
            // Bundle audio for desktop and WebAssembly alike: no working-directory
            // dependency or missing audio requests when the game is hosted online.
            music: load_optional("gamemusic.ogg", include_bytes!("../assets/audio/gamemusic.ogg")).await,
            jump: load_optional("jump.ogg", include_bytes!("../assets/audio/jump.ogg")).await,
            dash: load_optional("woosh.ogg", include_bytes!("../assets/audio/woosh.ogg")).await,
            volume: 0.35,
            muted: true,
            started: false,
            dragging: false,
            active: false,
        }
    }

    pub fn update(&mut self, show_controls: bool) {
        let mouse = Vec2::from(mouse_position());
        let clicked = is_mouse_button_pressed(MouseButton::Left);
        let active = crate::platform::active();
        let mut changed = self.active != active;
        if self.active && !active {
            if let Some(sound) = &self.jump { macroquad::audio::stop_sound(sound); }
            if let Some(sound) = &self.dash { macroquad::audio::stop_sound(sound); }
        }
        self.active = active;
        if show_controls && clicked && MUTE.contains(mouse) {
            self.muted = !self.muted;
            changed = true;
        }
        if show_controls && clicked && SLIDER.contains(mouse) {
            self.dragging = true;
        }
        if !show_controls {
            self.dragging = false;
        }
        if self.dragging {
            self.volume = ((mouse.x - SLIDER.x) / SLIDER.w).clamp(0.0, 1.0);
            changed = true;
        }
        if !is_mouse_button_down(MouseButton::Left) {
            self.dragging = false;
        }
        let volume = if self.muted || !active { 0.0 } else { self.volume };
        // Native playback starts immediately. The web backend starts after a
        // gesture, allowing its AudioContext to resume under autoplay restrictions.
        let can_start = active;
        if !self.started && can_start {
            if let Some(music) = &self.music {
                play_sound(music, PlaySoundParams { looped: true, volume });
                self.started = true;
            }
        } else if let Some(music) = self.music.as_ref().filter(|_| changed) {
            set_sound_volume(music, volume);
        }
    }

    pub fn movement(&self, jumped: bool, dashed: bool) {
        if let Some(sound) = self.jump.as_ref().filter(|_| jumped) {
            play_sound_once(sound);
        }
        if let Some(sound) = self.dash.as_ref().filter(|_| dashed) {
            play_sound_once(sound);
        }
    }

    pub fn draw(&self, font: Option<&Font>) {
        draw_rectangle(1165.0, 15.0, 410.0, 80.0, Color::new(0.15, 0.15, 0.15, 0.85));
        let hovered = MUTE.contains(Vec2::from(mouse_position()));
        let shade = if hovered { 0.4 } else { 0.25 };
        draw_rectangle(MUTE.x, MUTE.y, MUTE.w, MUTE.h, Color::new(shade, shade, shade, 0.9));
        draw_rectangle_lines(MUTE.x, MUTE.y, MUTE.w, MUTE.h, 1.0, LIGHTGRAY);
        crate::game::draw_label(if self.muted { "UNMUTE MUSIC" } else { "MUTE MUSIC" }, MUTE.x + 10.0, MUTE.y + 24.0, 20, WHITE, font);
        let y = SLIDER.center().y;
        draw_line(SLIDER.x, y, SLIDER.right(), y, 6.0, GRAY);
        let x = SLIDER.x + self.volume * SLIDER.w;
        draw_line(SLIDER.x, y, x, y, 6.0, GOLD);
        draw_circle(x, y, 9.0, if self.muted { GRAY } else { GOLD });
        let label = if self.music.is_none() {
            "Music unavailable".to_string()
        } else {
            format!("Music: {}%{}", (self.volume * 100.0).round() as u32, if self.muted { " (muted)" } else { "" })
        };
        crate::game::draw_label(&label, SLIDER.x, 84.0, 16, LIGHTGRAY, font);
    }
}
