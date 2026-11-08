//! P2 controller. Routes are verified with the same physics used by human players.
use crate::game::{Game, GameMode, Platform};
use crate::motion::Motion;
use macroquad::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug)]
struct Leg {
    platform: usize,
    dash: bool,
}

pub struct Npc {
    next_legs: Vec<Vec<Option<Leg>>>,
    leg: Option<Leg>,
    launched: bool,
    last_spawn_timer: f32,
}

fn center(platform: &Platform) -> Vec2 {
    vec2(platform.rect.center().x, platform.rect.y + 1.0)
}

/// Brake before reaching the target rather than oscillating around it.
fn steer(body: &Motion, x: f32) -> [bool; 6] {
    let error = x - body.pos.x - body.vel.x * 7.0;
    [error < -1.5, error > 1.5, false, false, false, false]
}

fn travel(body: &Motion, target: &Platform, dash: bool, launch: bool) -> [bool; 6] {
    let mut keys = steer(body, target.rect.center().x);
    if launch {
        // A lower platform directly underneath is reached by dropping through.
        let drop = target.rect.y > body.pos.y + 45.0
            && body.pos.x > target.rect.x + 20.0
            && body.pos.x < target.rect.right() - 20.0;
        keys[2] = !drop;
        keys[3] = drop;
    }
    // Dash near the apex only when there is still a substantial horizontal gap.
    if dash && !launch && body.vel.y > -4.0 && !body.wooshing {
        let dx = target.rect.center().x - body.pos.x;
        if dx.abs() > 200.0 {
            keys[4] = dx < 0.0;
            keys[5] = dx > 0.0;
        }
    }
    keys
}

fn can_reach(platforms: &[Platform], from: usize, leg: Leg) -> bool {
    let mut body = Motion::standing(center(&platforms[from]));
    for frame in 0..150 {
        let keys = travel(&body, &platforms[leg.platform], leg.dash, frame == 0);
        body.step(&keys, platforms);
        if let Some(landed) = body.support(platforms).filter(|_| frame > 0) {
            return landed == leg.platform;
        }
        if body.pos.y > 870.0 {
            break;
        }
    }
    false
}

impl Npc {
    pub fn new(platforms: &[Platform]) -> Self {
        let routes: Vec<Vec<Leg>> = (0..platforms.len())
            .map(|from| {
                (0..platforms.len())
                    .filter(|&to| to != from)
                    .filter_map(|to| {
                        [false, true]
                            .into_iter()
                            .map(|dash| Leg { platform: to, dash })
                            .find(|&leg| can_reach(platforms, from, leg))
                    })
                    .collect()
            })
            .collect();
        // The arena graph never changes. Solve every source once instead of
        // allocating a queue and searching it on every grounded simulation tick.
        let next_legs = (0..routes.len()).map(|from| {
            let mut queue = VecDeque::from([from]);
            let mut seen = vec![false; routes.len()];
            let mut first = vec![None; routes.len()];
            seen[from] = true;
            while let Some(node) = queue.pop_front() {
                for &leg in &routes[node] {
                    if seen[leg.platform] { continue; }
                    seen[leg.platform] = true;
                    first[leg.platform] = if node == from { Some(leg) } else { first[node] };
                    queue.push_back(leg.platform);
                }
            }
            first
        }).collect();
        Self {
            next_legs,
            leg: None,
            launched: false,
            last_spawn_timer: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.leg = None;
        self.launched = false;
        self.last_spawn_timer = 0.0;
    }

    fn next_leg(&self, from: usize, to: usize) -> Option<Leg> {
        self.next_legs[from][to]
    }

    pub fn keys(&mut self, game: &Game) -> [bool; 6] {
        let p2 = &game.players[1];
        // Scoring, void deaths and role swaps all respawn players.
        if p2.spawn_timer > self.last_spawn_timer {
            self.leg = None;
            self.launched = false;
        }
        self.last_spawn_timer = p2.spawn_timer;
        let target = if game.mode == GameMode::CoinRace {
            game.coin.pos
        } else if game.attacker_idx == 1 {
            game.flags[game.defender_idx].pos
        } else {
            game.players[0].pos
        };
        self.control(Motion::from(p2), target, &game.platforms)
    }

    fn control(&mut self, body: Motion, target: Vec2, platforms: &[Platform]) -> [bool; 6] {
        let support = body.support(platforms);
        if let Some(from) = support {
            // Replan after landing, including an unexpected landing or a moving opponent.
            if self.launched {
                self.leg = None;
                self.launched = false;
            }
            let destination = platforms
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let distance = |p: &Platform| {
                        let dx = (target.x - target.x.clamp(p.rect.x, p.rect.right())).abs();
                        dx + (target.y - p.rect.y).abs() * 2.0
                    };
                    distance(a).total_cmp(&distance(b))
                })
                .map(|(idx, _)| idx)
                .unwrap_or(from);
            if from == destination {
                self.leg = None;
                let mut keys = steer(
                    &body,
                    target.x.clamp(
                        platforms[from].rect.x + 22.0,
                        platforms[from].rect.right() - 22.0,
                    ),
                );
                keys[2] = target.y < body.pos.y - 45.0;
                return keys;
            }
            self.leg = self.next_leg(from, destination);
            if let Some(leg) = self.leg {
                // Routes are predicted from platform centres with low horizontal speed.
                let x = center(&platforms[from]).x;
                if (body.pos.x - x).abs() > 4.0 {
                    return steer(&body, x);
                }
                if body.vel.x.abs() > 0.2 {
                    return [false; 6];
                }
                self.launched = true;
                return travel(&body, &platforms[leg.platform], leg.dash, true);
            }
            return steer(&body, center(&platforms[from]).x);
        }
        if let Some(leg) = self.leg {
            return travel(&body, &platforms[leg.platform], leg.dash, false);
        }
        // Falling after a spawn or an interrupted jump: aim for the nearest safe landing.
        let landing = platforms
            .iter()
            .filter(|p| p.rect.y >= body.pos.y)
            .min_by(|a, b| {
                (a.rect.center().x - body.pos.x)
                    .abs()
                    .total_cmp(&(b.rect.center().x - body.pos.x).abs())
            });
        landing
            .map(|p| steer(&body, p.rect.center().x))
            .unwrap_or([false; 6])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::arena_platforms;

    #[test]
    fn every_platform_has_a_route_to_every_other_platform() {
        let platforms = arena_platforms();
        let npc = Npc::new(&platforms);
        for from in 0..platforms.len() {
            for to in 0..platforms.len() {
                if from != to {
                    assert!(npc.next_leg(from, to).is_some(), "no route {from} -> {to}");
                }
            }
        }
    }

    #[test]
    fn npc_reaches_targets_from_varied_spawns_without_falling_into_void() {
        let platforms = arena_platforms();
        let mut npc = Npc::new(&platforms);
        for from in 0..platforms.len() {
            for to in 0..platforms.len() {
                for offset in [
                    -platforms[from].rect.w / 2.0 + 20.0,
                    0.0,
                    platforms[from].rect.w / 2.0 - 20.0,
                ] {
                    for target_offset in [
                        -platforms[to].rect.w / 2.0 + 40.0,
                        0.0,
                        platforms[to].rect.w / 2.0 - 40.0,
                    ] {
                        npc.reset();
                        let mut body =
                            Motion::standing(center(&platforms[from]) + vec2(offset, 0.0));
                        let target = center(&platforms[to]) + vec2(target_offset, -1.0);
                        let mut reached = false;
                        for _ in 0..3000 {
                            let keys = npc.control(body, target, &platforms);
                            body.step(&keys, &platforms);
                            assert!(body.pos.y < 850.0, "void: {from} -> {to} offset {offset}");
                            if (body.pos.x - target.x).abs() < 35.0
                                && (body.pos.y - target.y).abs() < 40.0
                            {
                                reached = true;
                                break;
                            }
                        }
                        assert!(
                            reached,
                            "stuck: {from} -> {to} offset {offset}, at {:?}",
                            body.pos
                        );
                    }
                }
            }
        }
    }
}
