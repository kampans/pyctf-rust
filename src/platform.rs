//! Logical viewport and nonblocking browser lifecycle. Native builds stay active.
use crate::game::{HEIGHT, WIDTH};
use macroquad::prelude::*;

#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_arch = "wasm32")]
static ACTIVE: AtomicBool = AtomicBool::new(false);
#[cfg(target_arch = "wasm32")]
static READY: AtomicBool = AtomicBool::new(false);
#[cfg(target_arch = "wasm32")]
static RESET_CLOCK: AtomicBool = AtomicBool::new(false);

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pyctf_set_active(active: i32) {
    ACTIVE.store(active != 0, Ordering::Relaxed);
    RESET_CLOCK.store(true, Ordering::Relaxed);
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pyctf_ready() -> i32 {
    i32::from(READY.load(Ordering::Relaxed))
}

pub fn mark_ready() {
    #[cfg(target_arch = "wasm32")]
    READY.store(true, Ordering::Relaxed);
}

pub fn active() -> bool {
    #[cfg(target_arch = "wasm32")]
    return ACTIVE.load(Ordering::Relaxed);
    #[cfg(not(target_arch = "wasm32"))]
    true
}

pub fn take_clock_reset() -> bool {
    #[cfg(target_arch = "wasm32")]
    return RESET_CLOCK.swap(false, Ordering::Relaxed);
    #[cfg(not(target_arch = "wasm32"))]
    false
}

/// Both rendering and pointer input use this same letterboxed viewport.
fn viewport(width: f32, height: f32) -> Rect {
    let scale = (width / WIDTH).min(height / HEIGHT).max(0.001);
    let w = (WIDTH * scale).min(width);
    let h = (HEIGHT * scale).min(height);
    Rect::new((width - w) / 2.0, (height - h) / 2.0, w, h)
}

pub fn camera() -> Camera2D {
    camera_for_size(screen_width(), screen_height())
}

fn camera_for_size(width: f32, height: f32) -> Camera2D {
    let view = viewport(width, height);
    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, WIDTH, HEIGHT));
    // The arena uses top-left origin and downward-positive Y, including sprites.
    camera.zoom.y = camera.zoom.y.abs();
    camera.viewport = Some((view.x as i32, view.y as i32, view.w as i32, view.h as i32));
    camera
}

pub fn mouse_position() -> (f32, f32) {
    let mouse = Vec2::from(macroquad::input::mouse_position());
    let point = camera().screen_to_world(mouse);
    (point.x, point.y)
}

pub const STEP: f32 = 1.0 / 60.0;

/// Bound catch-up after slow frames; never simulate the time spent in another tab.
#[derive(Default)]
pub struct FixedClock(f32);

impl FixedClock {
    pub fn steps(&mut self, dt: f32) -> usize {
        self.0 += dt.clamp(0.0, 0.1);
        let count = ((self.0 + f32::EPSILON) / STEP) as usize;
        self.0 = (self.0 - count as f32 * STEP).max(0.0);
        count
    }

    pub fn reset(&mut self) {
        self.0 = 0.0;
    }

    pub fn interpolation(&self) -> f32 {
        (self.0 / STEP).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_keeps_the_arena_and_pointer_mapping_in_proportion() {
        for (w, h) in [(1600.0, 900.0), (800.0, 450.0), (1920.0, 1080.0), (800.0, 800.0), (400.0, 900.0)] {
            let view = viewport(w, h);
            assert!((view.w / view.h - WIDTH / HEIGHT).abs() < 0.001);
            assert!(view.x >= 0.0 && view.y >= 0.0);
            assert!((view.center() - vec2(w / 2.0, h / 2.0)).length() < 0.001);
            let camera = camera_for_size(w, h);
            let origin = camera.matrix().transform_point3(vec3(0.0, 0.0, 0.0));
            let bottom_right = camera.matrix().transform_point3(vec3(WIDTH, HEIGHT, 0.0));
            assert!((origin - vec3(-1.0, 1.0, 0.0)).length() < 0.001);
            assert!((bottom_right - vec3(1.0, -1.0, 0.0)).length() < 0.001);
        }
    }

    #[test]
    fn simulation_runs_at_60hz_on_different_displays() {
        for fps in [30, 60, 120, 144, 240] {
            let mut clock = FixedClock::default();
            let steps: usize = (0..fps * 10).map(|_| clock.steps(1.0 / fps as f32)).sum();
            assert!((steps as i32 - 600).abs() <= 1, "{fps} FPS: {steps} ticks");
        }
        let mut clock = FixedClock::default();
        assert!(clock.steps(120.0) <= 6);
        clock.reset();
        assert_eq!(clock.steps(0.0), 0);
    }
}
