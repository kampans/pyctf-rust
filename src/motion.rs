//! Shared, texture-free movement for live players and NPC route prediction.
use crate::game::{ACC, FRICT, GRAVITY, HITBOX_SIZE, PLAYER_SIZE, Platform, Player, WIDTH};
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Motion {
    pub pos: Vec2,
    pub vel: Vec2,
    pub acc: Vec2,
    pub jumping: bool,
    pub wooshing: bool,
}

impl From<&Player> for Motion {
    fn from(player: &Player) -> Self {
        Self {
            pos: player.pos,
            vel: player.vel,
            acc: player.acc,
            jumping: player.jumping,
            wooshing: player.wooshing,
        }
    }
}

impl Motion {
    pub fn standing(pos: Vec2) -> Self {
        Self {
            pos,
            vel: Vec2::ZERO,
            acc: Vec2::ZERO,
            jumping: false,
            wooshing: false,
        }
    }

    pub fn support(&self, platforms: &[Platform]) -> Option<usize> {
        platforms.iter().position(|p| {
            self.vel.y.abs() < 0.1
                && (self.pos.y - p.rect.y - 1.0).abs() < 2.0
                && self.pos.x + PLAYER_SIZE / 2.0 > p.rect.x
                && self.pos.x - PLAYER_SIZE / 2.0 < p.rect.right()
        })
    }

    pub fn step(&mut self, keys: &[bool; 6], platforms: &[Platform]) {
        let hitbox = Rect::new(
            self.pos.x - HITBOX_SIZE / 2.0,
            self.pos.y - HITBOX_SIZE,
            HITBOX_SIZE,
            HITBOX_SIZE,
        );
        let platform = platforms.iter().find(|p| {
            hitbox.x < p.rect.right()
                && hitbox.right() > p.rect.x
                && hitbox.y < p.rect.bottom()
                && hitbox.bottom() > p.rect.y
        });
        self.acc = vec2(0.0, GRAVITY);
        if keys[0] {
            self.acc.x -= ACC;
        }
        if keys[1] {
            self.acc.x += ACC;
        }
        if keys[2] && platform.is_some() && !self.jumping {
            self.vel.y = -17.5;
            self.jumping = true;
        }
        if let Some(p) = platform.filter(|_| keys[3]) {
            self.vel.y = 10.5;
            self.pos.y = p.rect.y + 44.0;
        }
        if platform.is_none() && !self.wooshing {
            if keys[4] {
                self.vel.x = -50.0;
                self.wooshing = true;
            } else if keys[5] {
                self.vel.x = 50.0;
                self.wooshing = true;
            }
        }
        self.acc.x += self.vel.x * FRICT;
        self.vel += self.acc;
        self.pos += self.vel + 0.5 * self.acc;

        // Keep the player on screen horizontally.
        if self.pos.x >= WIDTH - 20.0 {
            self.pos.x = WIDTH - 30.0;
        }
        if self.pos.x <= 20.0 {
            self.pos.x = 35.0;
        }

        // Land on any platform from above. `pos.y` is the bottom of the player.
        for p in platforms {
            let player_left = self.pos.x - PLAYER_SIZE / 2.0;
            let player_right = self.pos.x + PLAYER_SIZE / 2.0;
            let plat_left = p.rect.x;
            let plat_right = p.rect.x + p.rect.w;

            if self.vel.y > 0.0
                && self.pos.y >= p.rect.y
                && self.pos.y < p.rect.y + PLAYER_SIZE
                && player_left < plat_right
                && player_right > plat_left
            {
                self.jumping = false;
                self.wooshing = false;
                self.vel.y = 0.0;
                self.pos.y = p.rect.y + 1.0;
                break;
            }
        }
    }
}
