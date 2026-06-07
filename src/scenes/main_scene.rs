use quartz::prelude::*;
use std::sync::Arc;

const VW: f32 = 3840.0;
const VH: f32 = 2160.0;
const PLAYER_SIZE: f32 = 48.0;
const PLAYER_THRUST: f32 = 3.4;
const PLAYER_TURN_SPEED: f32 = 3.0;
const PLAYER_DRAG: f32 = 0.985;
const BULLET_SPEED: f32 = 420.0;
const BULLET_SIZE: f32 = 14.0;
const ASTEROID_MIN: f32 = 52.0;
const ASTEROID_MAX: f32 = 92.0;
const ASTEROID_SPEED_MIN: f32 = 60.0;
const ASTEROID_SPEED_MAX: f32 = 130.0;
const SPAWN_INTERVAL: f32 = 1.4;
const PLAYER_HIT_COOLDOWN_SECS: f32 = 0.45;
const LIVES_START: i32 = 3;
const LAYER_PLAYER: u32 = 1;
const LAYER_BULLET: u32 = 2;
const LAYER_ASTEROID: u32 = LAYER_PLAYER | LAYER_BULLET;
const FONT_BYTES: &[u8] = include_bytes!("../../assets/font.ttf");

pub fn setup_scene(canvas: &mut Canvas) {
    canvas.run(Action::EnableCrystalline);

    canvas.set_var("score", 0i32);
    canvas.set_var("lives", LIVES_START);
    canvas.set_var("spawn_timer", 0.0f32);
    canvas.set_var("bullet_cooldown", 0.0f32);
    canvas.set_var("player_hit_cooldown", 0.0f32);
    canvas.set_var("player_angle", 0.0f32);
    canvas.set_var("bullet_seq", 0i32);
    canvas.set_var("asteroid_seq", 0i32);

    let mut bg = GameObject::build("bg")
        .size(VW, VH)
        .position(0.0, 0.0)
        .layer(-10)
        .screen_space()
        .ignore_zoom()
        .no_collision()
        .finish();
    bg.set_drawable(Box::new(quartz::sprite::tint_overlay(VW, VH, Color(8, 12, 24, 255))));
    canvas.add_game_object("bg".to_owned(), bg);

    let mut player = GameObject::build("player")
        .size(PLAYER_SIZE, PLAYER_SIZE)
        .center_at(VW * 0.5, VH * 0.5)
        .layer(2)
        .solid_circle(PLAYER_SIZE * 0.5)
        .collision_layer(LAYER_PLAYER)
        .collision_mask(LAYER_ASTEROID)
        .gravity(0.0)
        .resistance(PLAYER_DRAG, PLAYER_DRAG)
        .tag("player")
        .finish();
    // Crystalline treats is_platform=true as static; keep circle collision but allow integration.
    player.is_platform = false;
    player.set_drawable(Box::new(quartz::sprite::solid_circle(PLAYER_SIZE, Color(84, 210, 255, 255))));
    canvas.add_game_object("player".to_owned(), player);

    let hud_font = Arc::new(Font::from_bytes(FONT_BYTES).expect("hud font"));
    let score_txt = canvas.make_text(
        "SCORE  0".into(),
        36.0,
        Color(255, 255, 255, 255),
        Align::Left,
        hud_font.clone(),
    );
    let lives_txt = canvas.make_text(
        "LIVES  ***".into(),
        36.0,
        Color(255, 120, 120, 255),
        Align::Left,
        hud_font.clone(),
    );
    let game_over_txt = canvas.make_text(
        "GAME OVER  -  PRESS R TO RESTART".into(),
        54.0,
        Color(255, 80, 80, 255),
        Align::Center,
        hud_font,
    );

    let mut hud_score = GameObject::build("hud_score")
        .size(540.0, 58.0)
        .position(20.0, 20.0)
        .layer(12)
        .screen_space()
        .ignore_zoom()
        .no_collision()
        .tag("hud")
        .finish();
    hud_score.set_drawable(Box::new(score_txt));
    canvas.add_game_object("hud_score".to_owned(), hud_score);

    let mut hud_lives = GameObject::build("hud_lives")
        .size(480.0, 58.0)
        .position(20.0, 86.0)
        .layer(12)
        .screen_space()
        .ignore_zoom()
        .no_collision()
        .tag("hud")
        .finish();
    hud_lives.set_drawable(Box::new(lives_txt));
    canvas.add_game_object("hud_lives".to_owned(), hud_lives);

    let mut overlay = GameObject::build("game_over_overlay")
        .size(1300.0, 120.0)
        .center_at(VW * 0.5, VH * 0.5)
        .layer(30)
        .screen_space()
        .ignore_zoom()
        .no_collision()
        .tag("overlay")
        .finish();
    overlay.visible = false;
    overlay.set_drawable(Box::new(game_over_txt));
    canvas.add_game_object("game_over_overlay".to_owned(), overlay);

    let mut camera = Camera::new((VW, VH), (VW, VH));
    camera.follow(Some(Target::name("player")));
    canvas.set_camera(camera);
}

pub fn register_logic(canvas: &mut Canvas) {
    canvas.on_update(|canvas| {
        const DT: f32 = 1.0 / 60.0;
        let game_over = canvas.has_var("game_over_true");

        if !game_over {
            let spawn_timer = canvas.get_f32("spawn_timer") + DT;
            if spawn_timer >= SPAWN_INTERVAL {
                canvas.set_var("spawn_timer", 0.0f32);
                spawn_asteroid(canvas);
            } else {
                canvas.set_var("spawn_timer", spawn_timer);
            }
        }

        if !game_over {
            let thrusting = canvas.is_key_held(&Key::Character('w'.to_string()))
                || canvas.is_key_held(&Key::Named(NamedKey::ArrowUp));
            if thrusting {
                let angle = canvas.get_f32("player_angle").to_radians();
                let mx = angle.sin() * PLAYER_THRUST;
                let my = -angle.cos() * PLAYER_THRUST;
                canvas.run(Action::ApplyMomentum {
                    target: Target::name("player"),
                    value: (mx, my),
                });
            }
        }

        let cooldown = (canvas.get_f32("bullet_cooldown") - DT).max(0.0);
        canvas.set_var("bullet_cooldown", cooldown);
        let hit_cooldown = (canvas.get_f32("player_hit_cooldown") - DT).max(0.0);
        canvas.set_var("player_hit_cooldown", hit_cooldown);
        if !game_over && cooldown <= 0.0 && canvas.is_key_held(&Key::Named(NamedKey::Space)) {
            canvas.set_var("bullet_cooldown", 0.17f32);
            fire_bullet(canvas);
        }

        if !game_over {
            wrap_player(canvas);
            resolve_bullet_asteroid_hits(canvas);
            cleanup_offscreen_projectiles(canvas);
        }

        refresh_hud(canvas);
    });
}

fn resolve_bullet_asteroid_hits(canvas: &mut Canvas) {
    for (left, right) in canvas.last_collision_pairs.clone() {
        let bullet_is_left = left.starts_with("bullet_");
        let bullet_is_right = right.starts_with("bullet_");
        let asteroid_is_left = left.starts_with("asteroid_");
        let asteroid_is_right = right.starts_with("asteroid_");

        if !(bullet_is_left && asteroid_is_right || bullet_is_right && asteroid_is_left) {
            continue;
        }

        let bullet_id = if bullet_is_left { &left } else { &right };
        let asteroid_id = if asteroid_is_left { &left } else { &right };

        if canvas.get_game_object(bullet_id).is_none() || canvas.get_game_object(asteroid_id).is_none() {
            continue;
        }

        canvas.run(Action::Multi(vec![
            Action::Remove {
                target: Target::name(bullet_id),
            },
            Action::Remove {
                target: Target::name(asteroid_id),
            },
            Action::ModVar {
                name: "score".into(),
                op: MathOp::Add,
                operand: Expr::i32(10),
            },
            Action::CameraFlash {
                color: Color(255, 200, 90, 110),
                duration: 0.10,
            },
        ]));
    }
}

pub fn register_events(canvas: &mut Canvas) {
    canvas.add_event(
        GameEvent::KeyHold {
            key: Key::Character('a'.to_string()),
            action: Action::Conditional {
                condition: Condition::Not(Box::new(Condition::VarExists("game_over_true".into()))),
                if_true: Box::new(Action::Multi(vec![
                    Action::AddRotation {
                        target: Target::name("player"),
                        value: -PLAYER_TURN_SPEED,
                    },
                    Action::ModVar {
                        name: "player_angle".into(),
                        op: MathOp::Sub,
                        operand: Expr::f32(PLAYER_TURN_SPEED),
                    },
                ])),
                if_false: None,
            },
            target: Target::name("player"),
            modifiers: None,
        },
        Target::name("player"),
    );

    canvas.add_event(
        GameEvent::KeyHold {
            key: Key::Named(NamedKey::ArrowLeft),
            action: Action::Conditional {
                condition: Condition::Not(Box::new(Condition::VarExists("game_over_true".into()))),
                if_true: Box::new(Action::Multi(vec![
                    Action::AddRotation {
                        target: Target::name("player"),
                        value: -PLAYER_TURN_SPEED,
                    },
                    Action::ModVar {
                        name: "player_angle".into(),
                        op: MathOp::Sub,
                        operand: Expr::f32(PLAYER_TURN_SPEED),
                    },
                ])),
                if_false: None,
            },
            target: Target::name("player"),
            modifiers: None,
        },
        Target::name("player"),
    );

    canvas.add_event(
        GameEvent::KeyHold {
            key: Key::Character('d'.to_string()),
            action: Action::Conditional {
                condition: Condition::Not(Box::new(Condition::VarExists("game_over_true".into()))),
                if_true: Box::new(Action::Multi(vec![
                    Action::AddRotation {
                        target: Target::name("player"),
                        value: PLAYER_TURN_SPEED,
                    },
                    Action::ModVar {
                        name: "player_angle".into(),
                        op: MathOp::Add,
                        operand: Expr::f32(PLAYER_TURN_SPEED),
                    },
                ])),
                if_false: None,
            },
            target: Target::name("player"),
            modifiers: None,
        },
        Target::name("player"),
    );

    canvas.add_event(
        GameEvent::KeyHold {
            key: Key::Named(NamedKey::ArrowRight),
            action: Action::Conditional {
                condition: Condition::Not(Box::new(Condition::VarExists("game_over_true".into()))),
                if_true: Box::new(Action::Multi(vec![
                    Action::AddRotation {
                        target: Target::name("player"),
                        value: PLAYER_TURN_SPEED,
                    },
                    Action::ModVar {
                        name: "player_angle".into(),
                        op: MathOp::Add,
                        operand: Expr::f32(PLAYER_TURN_SPEED),
                    },
                ])),
                if_false: None,
            },
            target: Target::name("player"),
            modifiers: None,
        },
        Target::name("player"),
    );

    canvas.add_event(
        GameEvent::Collision {
            action: Action::Conditional {
                condition: Condition::Not(Box::new(Condition::VarExists("game_over_true".into()))),
                if_true: Box::new(Action::Custom {
                    name: "on_bullet_asteroid_collision".into(),
                }),
                if_false: None,
            },
            target: Target::tag("asteroid"),
        },
        Target::tag("bullet"),
    );

    canvas.add_event(
        GameEvent::Collision {
            action: Action::Conditional {
                condition: Condition::Not(Box::new(Condition::VarExists("game_over_true".into()))),
                if_true: Box::new(Action::Custom {
                    name: "on_player_asteroid_collision".into(),
                }),
                if_false: None,
            },
            target: Target::tag("asteroid"),
        },
        Target::name("player"),
    );

    canvas.add_event(
        GameEvent::KeyPress {
            key: Key::Character('r'.to_string()),
            action: Action::Conditional {
                condition: Condition::VarExists("game_over_true".into()),
                if_true: Box::new(Action::Custom {
                    name: "do_restart".into(),
                }),
                if_false: None,
            },
            target: Target::name("player"),
            modifiers: None,
        },
        Target::name("player"),
    );

    canvas.register_custom_event("do_restart".into(), |canvas| {
        canvas.set_var("score", 0i32);
        canvas.set_var("lives", LIVES_START);
        canvas.set_var("spawn_timer", 0.0f32);
        canvas.set_var("bullet_cooldown", 0.0f32);
        canvas.set_var("player_hit_cooldown", 0.0f32);
        canvas.set_var("player_angle", 0.0f32);
        canvas.remove_var("game_over_true");

        canvas.run(Action::Multi(vec![
            Action::Show {
                target: Target::name("player"),
            },
            Action::Hide {
                target: Target::name("game_over_overlay"),
            },
            Action::ClearTint {
                target: Target::name("player"),
            },
            Action::SetMomentum {
                target: Target::name("player"),
                value: (0.0, 0.0),
            },
            Action::SetRotation {
                target: Target::name("player"),
                value: 0.0,
            },
            Action::Teleport {
                target: Target::name("player"),
                location: Location::at(VW * 0.5 - PLAYER_SIZE * 0.5, VH * 0.5 - PLAYER_SIZE * 0.5),
            },
            Action::Remove {
                target: Target::tag("asteroid"),
            },
            Action::Remove {
                target: Target::tag("bullet"),
            },
        ]));
    });

    canvas.register_custom_event("on_bullet_asteroid_collision".into(), |canvas| {
        for (left, right) in canvas.last_collision_pairs.clone() {
            let bullet_is_left = left.starts_with("bullet_");
            let bullet_is_right = right.starts_with("bullet_");
            let asteroid_is_left = left.starts_with("asteroid_");
            let asteroid_is_right = right.starts_with("asteroid_");

            if !(bullet_is_left && asteroid_is_right || bullet_is_right && asteroid_is_left) {
                continue;
            }

            let bullet_id = if bullet_is_left { &left } else { &right };
            let asteroid_id = if asteroid_is_left { &left } else { &right };

            if canvas.get_game_object(bullet_id).is_none() || canvas.get_game_object(asteroid_id).is_none() {
                continue;
            }

            canvas.run(Action::Multi(vec![
                Action::Remove {
                    target: Target::name(bullet_id),
                },
                Action::Remove {
                    target: Target::name(asteroid_id),
                },
                Action::ModVar {
                    name: "score".into(),
                    op: MathOp::Add,
                    operand: Expr::i32(10),
                },
                Action::CameraFlash {
                    color: Color(255, 200, 90, 110),
                    duration: 0.10,
                },
            ]));
        }
    });

    canvas.register_custom_event("on_player_asteroid_collision".into(), |canvas| {
        if canvas.has_var("game_over_true") || canvas.get_f32("player_hit_cooldown") > 0.0 {
            return;
        }

        let player_asteroid_hit = canvas.last_collision_pairs.iter().any(|(left, right)| {
            (left == "player" && right.starts_with("asteroid_"))
                || (right == "player" && left.starts_with("asteroid_"))
        });

        if !player_asteroid_hit {
            return;
        }

        canvas.set_var("player_hit_cooldown", PLAYER_HIT_COOLDOWN_SECS);
        canvas.run(Action::Multi(vec![
            Action::ModVar {
                name: "lives".into(),
                op: MathOp::Sub,
                operand: Expr::i32(1),
            },
            Action::CameraShake {
                intensity: 16.0,
                duration: 0.35,
            },
            Action::SetTint {
                target: Target::name("player"),
                color: Color(255, 120, 120, 220),
            },
            Action::Conditional {
                condition: Expr::var("lives").lte(Expr::i32(0)),
                if_true: Box::new(Action::Multi(vec![
                    Action::SetVar {
                        name: "game_over_true".into(),
                        value: Expr::bool(true),
                    },
                    Action::Show {
                        target: Target::name("game_over_overlay"),
                    },
                    Action::Hide {
                        target: Target::name("player"),
                    },
                    Action::CameraFlash {
                        color: Color(255, 60, 60, 180),
                        duration: 0.45,
                    },
                ])),
                if_false: None,
            },
        ]));
    });
}

// custom code fallback block: runtime_helpers
fn fire_bullet(canvas: &mut Canvas) {
    let seq = canvas.get_i32("bullet_seq") + 1;
    canvas.set_var("bullet_seq", seq);
    let id = format!("bullet_{}", seq);

    let angle = canvas.get_f32("player_angle").to_radians();
    let vx = angle.sin() * BULLET_SPEED / 60.0;
    let vy = -angle.cos() * BULLET_SPEED / 60.0;

    let (x, y) = canvas
        .get_game_object("player")
        .map(|obj| {
            (
                obj.position.0 + obj.size.0 * 0.5 - BULLET_SIZE * 0.5,
                obj.position.1 + obj.size.1 * 0.5 - BULLET_SIZE * 0.5,
            )
        })
        .unwrap_or((VW * 0.5, VH * 0.5));

    let mut bullet = GameObject::build(id.clone())
        .size(BULLET_SIZE, BULLET_SIZE)
        .position(x, y)
        .layer(4)
        .solid_circle(BULLET_SIZE * 0.5)
        .collision_layer(LAYER_BULLET)
        .collision_mask(LAYER_ASTEROID)
        .momentum(vx, vy)
        .gravity(0.0)
        .resistance(1.0, 1.0)
        .tag("bullet")
        .finish();
    // Dynamic projectile under crystalline (not a static platform body).
    bullet.is_platform = false;
    bullet.set_drawable(Box::new(quartz::sprite::solid_circle(BULLET_SIZE, Color(255, 240, 90, 255))));
    canvas.add_game_object(id, bullet);
}

fn spawn_asteroid(canvas: &mut Canvas) {
    let seq = canvas.get_i32("asteroid_seq") + 1;
    canvas.set_var("asteroid_seq", seq);
    let id = format!("asteroid_{}", seq);

    let size = canvas.entropy.range(ASTEROID_MIN, ASTEROID_MAX);
    let speed = canvas.entropy.range(ASTEROID_SPEED_MIN, ASTEROID_SPEED_MAX) / 60.0;
    let angle = canvas.entropy.range(0.0f32, 360.0f32).to_radians();
    let vx = angle.sin() * speed;
    let vy = -angle.cos() * speed;

    let side = canvas.entropy.range(0.0f32, 4.0f32) as i32;
    let (sx, sy) = match side {
        0 => (canvas.entropy.range(0.0f32, VW), -size),
        1 => (canvas.entropy.range(0.0f32, VW), VH + size),
        2 => (-size, canvas.entropy.range(0.0f32, VH)),
        _ => (VW + size, canvas.entropy.range(0.0f32, VH)),
    };

    let mut asteroid = GameObject::build(id.clone())
        .size(size, size)
        .position(sx, sy)
        .layer(1)
        .solid_circle(size * 0.5)
        .collision_layer(LAYER_ASTEROID)
        .collision_mask(LAYER_PLAYER | LAYER_BULLET)
        .momentum(vx, vy)
        .gravity(0.0)
        .resistance(1.0, 1.0)
        .tag("asteroid")
        .finish();
    // Dynamic asteroid under crystalline (not a static platform body).
    asteroid.is_platform = false;
    asteroid.set_drawable(Box::new(quartz::sprite::solid_circle(size, Color(162, 108, 66, 255))));
    canvas.add_game_object(id, asteroid);
}

fn wrap_player(canvas: &mut Canvas) {
    if let Some(player) = canvas.get_game_object("player") {
        let (px, py) = player.position;
        let (pw, ph) = player.size;
        let cx = px + pw * 0.5;
        let cy = py + ph * 0.5;
        let wx = if cx < -pw {
            VW + pw * 0.5
        } else if cx > VW + pw {
            -pw * 0.5
        } else {
            cx
        };
        let wy = if cy < -ph {
            VH + ph * 0.5
        } else if cy > VH + ph {
            -ph * 0.5
        } else {
            cy
        };
        if (wx - cx).abs() > 0.5 || (wy - cy).abs() > 0.5 {
            canvas.run(Action::Teleport {
                target: Target::name("player"),
                location: Location::at(wx - pw * 0.5, wy - ph * 0.5),
            });
        }
    }
}

fn cleanup_offscreen_projectiles(canvas: &mut Canvas) {
    let margin = 140.0;
    let out_of_bounds = |x: f32, y: f32, w: f32, h: f32| {
        x + w < -margin || x > VW + margin || y + h < -margin || y > VH + margin
    };

    let to_remove: Vec<String> = canvas
        .object_names()
        .to_vec()
        .into_iter()
        .filter(|name| name.starts_with("bullet_") || name.starts_with("asteroid_"))
        .filter(|name| {
            canvas
                .get_game_object(name)
                .map(|obj| out_of_bounds(obj.position.0, obj.position.1, obj.size.0, obj.size.1))
                .unwrap_or(false)
        })
        .collect();

    for name in to_remove {
        canvas.run(Action::Remove {
            target: Target::name(&name),
        });
    }
}

fn refresh_hud(canvas: &mut Canvas) {
    let score = canvas.get_i32("score");
    let lives = canvas.get_i32("lives").max(0);
    let hearts = "* ".repeat(lives as usize);

    let font = match Font::from_bytes(FONT_BYTES) {
        Ok(font) => Arc::new(font),
        Err(_) => return,
    };

    let score_text = canvas.make_text(
        format!("SCORE  {}", score),
        36.0,
        Color(255, 255, 255, 255),
        Align::Left,
        font.clone(),
    );
    let lives_text = canvas.make_text(
        format!("LIVES  {}", if hearts.trim().is_empty() { "---" } else { hearts.trim_end() }),
        36.0,
        Color(255, 120, 120, 255),
        Align::Left,
        font,
    );

    if let Some(hud_score) = canvas.get_game_object_mut("hud_score") {
        hud_score.set_drawable(Box::new(score_text));
    }
    if let Some(hud_lives) = canvas.get_game_object_mut("hud_lives") {
        hud_lives.set_drawable(Box::new(lives_text));
    }
}

