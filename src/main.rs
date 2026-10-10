//! Ragdoll Cricket — a physics batting game. See SPEC.md for the rules and numbers.
//!
//! The simulation works in "world pixels" (y down) divided by 40 to get metres,
//! exactly as described in the spec, so the tables below can be checked against it.

mod art;
mod audio;

use audio::Sfx;
use macroquad::prelude::*;
use rapier2d::prelude::*;

// ---------- spec constants ----------

const PX_PER_M: f32 = 40.0;
const GRAVITY: f32 = 10.0;
const FRAME_DT: f32 = 1.0 / 30.0; // real seconds per game frame
const SIM_PER_FRAME: f32 = 0.04; // simulated seconds per game frame
const SUBSTEPS: usize = 8;
/// Simulated seconds per physics substep.
const SUB_DT: f32 = SIM_PER_FRAME / SUBSTEPS as f32;
/// The original runs 0.04 s of physics per 1/30 s frame: 1.2× real time.
const TIME_SCALE: f32 = SIM_PER_FRAME / FRAME_DT;
const BALL_INTERVAL: f32 = 2.0;
const BALL_RADIUS_PX: f32 = 17.5;

const STAGE_W: f32 = 590.0;
const STAGE_H: f32 = 384.0;
const VIEW_ORIGIN: (f32, f32) = (278.0, 200.0);
const VIEW_SCALE: f32 = 0.431;
const VIEW_TARGET: (f32, f32) = (52.7, 92.8);

// Collision groups (see "Collision filtering" in the spec).
const G_GROUND: Group = Group::GROUP_1;
const G_BALL: Group = Group::GROUP_2;
const G_BODY: Group = Group::GROUP_3;
const G_WICKET: Group = Group::GROUP_4;
const G_BAT: Group = Group::GROUP_5;
const G_ZONE: Group = Group::GROUP_6;

fn m(px: f32) -> f32 {
    px / PX_PER_M
}

// ---------- level tables (world px, y down) ----------

#[derive(Clone, Copy, PartialEq)]
enum Zone {
    Runs(u32),
    Out,
}

/// (center, half size, effect)
const ZONES: [((f32, f32), (f32, f32), Zone); 9] = [
    ((780.7, 170.1), (25.4, 76.0), Zone::Runs(1)),
    ((780.7, 44.1), (25.4, 76.0), Zone::Runs(2)),
    ((780.6, -158.6), (25.4, 126.7), Zone::Runs(4)),
    ((780.5, -310.6), (25.4, 25.4), Zone::Runs(6)),
    ((653.8, -361.3), (152.0, 25.4), Zone::Runs(6)),
    ((-55.6, -361.4), (557.5, 25.5), Zone::Out),
    ((-587.8, 43.4), (25.4, 202.7), Zone::Out),
    ((-587.9, -184.0), (25.4, 50.7), Zone::Runs(4)),
    ((-587.8, -285.2), (25.4, 50.7), Zone::Runs(6)),
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Part {
    BackShin,
    BackThigh,
    FrontShin,
    FrontThigh,
    Chest,
    Head,
    TopUpper,
    TopFore,
    BottomUpper,
    BottomFore,
}

struct PartDef {
    part: Part,
    center: (f32, f32),
    half: (f32, f32),
    rot_deg: f32,
    density: f32,
    ang_damping: f32,
}

const PARTS: [PartDef; 10] = [
    PartDef {
        part: Part::BackShin,
        center: (-278.2, 179.9),
        half: (90.2, 26.3),
        rot_deg: -45.0,
        density: 2.0,
        ang_damping: 0.5,
    },
    PartDef {
        part: Part::BackThigh,
        center: (-201.0, 84.0),
        half: (69.6, 29.9),
        rot_deg: -45.0,
        density: 1.0,
        ang_damping: 0.0,
    },
    PartDef {
        part: Part::FrontShin,
        center: (-110.3, 167.5),
        half: (30.3, 80.8),
        rot_deg: 0.0,
        density: 2.0,
        ang_damping: 0.5,
    },
    PartDef {
        part: Part::FrontThigh,
        center: (-138.0, 87.2),
        half: (27.3, 58.2),
        rot_deg: -30.0,
        density: 1.0,
        ang_damping: 0.0,
    },
    PartDef {
        part: Part::Chest,
        center: (-173.2, 7.5),
        half: (41.0, 51.6),
        rot_deg: 0.0,
        density: 1.0,
        ang_damping: 0.0,
    },
    PartDef {
        part: Part::Head,
        center: (-150.2, -41.2),
        half: (33.8, 31.7),
        rot_deg: 0.0,
        density: 1.0,
        ang_damping: 0.0,
    },
    PartDef {
        part: Part::TopUpper,
        center: (-126.8, -23.3),
        half: (47.0, 25.5),
        rot_deg: 0.0,
        density: 1.4,
        ang_damping: 0.0,
    },
    PartDef {
        part: Part::TopFore,
        center: (-68.0, 2.0),
        half: (15.6, 45.3),
        rot_deg: -30.0,
        density: 1.0,
        ang_damping: 0.0,
    },
    PartDef {
        part: Part::BottomUpper,
        center: (-159.8, 18.3),
        half: (43.2, 18.0),
        rot_deg: 30.0,
        density: 1.4,
        ang_damping: 0.0,
    },
    PartDef {
        part: Part::BottomFore,
        center: (-91.7, 43.6),
        half: (49.9, 15.7),
        rot_deg: 0.0,
        density: 1.0,
        ang_damping: 0.0,
    },
];

#[derive(Clone, Copy)]
enum Anchor {
    Ground,
    Part(Part),
    Bat,
}

/// (body1, body2, world anchor px, limits, spring k/damping)
const JOINTS: [(Anchor, Anchor, (f32, f32), (f32, f32), Option<(f32, f32)>); 13] = [
    (
        Anchor::Ground,
        Anchor::Part(Part::FrontShin),
        (-101.5, 239.85),
        (-0.5, 0.5),
        Some((150.0, 0.8)),
    ),
    (
        Anchor::Part(Part::FrontShin),
        Anchor::Part(Part::FrontThigh),
        (-105.9, 123.75),
        (-0.8, 0.3),
        Some((80.0, 0.5)),
    ),
    (
        Anchor::Part(Part::FrontThigh),
        Anchor::Part(Part::Chest),
        (-164.15, 52.3),
        (-0.3, 0.5),
        Some((80.0, 0.8)),
    ),
    (
        Anchor::Ground,
        Anchor::Part(Part::BackShin),
        (-329.2, 241.55),
        (-0.2, 0.5),
        Some((150.0, 0.8)),
    ),
    (
        Anchor::Part(Part::BackShin),
        Anchor::Part(Part::BackThigh),
        (-232.1, 130.15),
        (-1.2, 0.1),
        Some((80.0, 0.8)),
    ),
    (
        Anchor::Part(Part::BackThigh),
        Anchor::Part(Part::Chest),
        (-192.9, 54.35),
        (-0.6, 0.3),
        Some((80.0, 0.8)),
    ),
    (
        Anchor::Part(Part::Chest),
        Anchor::Part(Part::Head),
        (-172.75, -30.25),
        (0.0, 0.0),
        None,
    ),
    (
        Anchor::Part(Part::Chest),
        Anchor::Part(Part::TopUpper),
        (-163.85, -27.0),
        (-0.5, 1.1),
        Some((50.0, 0.5)),
    ),
    (
        Anchor::Part(Part::TopUpper),
        Anchor::Part(Part::TopFore),
        (-86.5, -22.95),
        (-0.1, 2.0),
        Some((15.0, 0.5)),
    ),
    (
        Anchor::Part(Part::TopFore),
        Anchor::Bat,
        (-58.7, 21.9),
        (-0.5, 0.5),
        Some((20.0, 0.5)),
    ),
    (
        Anchor::Part(Part::Chest),
        Anchor::Part(Part::BottomUpper),
        (-183.5, 4.55),
        (-3.0, 3.0),
        Some((50.0, 0.5)),
    ),
    (
        Anchor::Part(Part::BottomUpper),
        Anchor::Part(Part::BottomFore),
        (-131.25, 37.8),
        (-2.5, 0.3),
        Some((15.0, 0.5)),
    ),
    (
        Anchor::Part(Part::BottomFore),
        Anchor::Bat,
        (-59.55, 47.9),
        (-1.5, 1.5),
        Some((20.0, 0.5)),
    ),
];

/// Bat origin (top-left of the bat's local frame) in world px.
const BAT_ORIGIN: (f32, f32) = (-65.25, 4.9);
/// Our bat: a handle and a blade, in bat-local px (y down, handle at the top).
const BAT_HANDLE: [(f32, f32); 4] = [(8.0, 0.0), (15.5, 0.0), (15.5, 60.0), (8.0, 60.0)];
const BAT_BLADE: [(f32, f32); 6] = [
    (1.5, 56.0),
    (22.0, 56.0),
    (23.4, 168.0),
    (18.0, 177.85),
    (5.0, 177.85),
    (0.0, 168.0),
];
const BAT_GRAB: (f32, f32) = (-102.0, 121.0);

// ---------- runtime state ----------

/// A revolute joint's angular behaviour, tracked on the *unwrapped* angle so
/// limits still hold when a limb whips past ±π (rapier only reports the
/// wrapped angle, which lets fast limbs slip through a limit).
struct Hinge {
    b1: RigidBodyHandle,
    b2: RigidBodyHandle,
    rest: f32,
    limits: (f32, f32),
    spring: Option<(f32, f32)>,
    angle: f32,
    last_wrapped: f32,
}

struct Ball {
    body: RigidBodyHandle,
    collider: ColliderHandle,
    hit: bool,
    scored: bool,
    on_ground: bool,
}

struct Burst {
    pos: Vec2, // world px
    label: &'static str,
    color: Color,
    /// Age in simulated seconds (the pop lasts 20 of the original's frames).
    age: f32,
}

#[derive(PartialEq, Clone, Copy)]
enum Screen {
    Title,
    Playing,
    Ended,
}

struct World {
    pipeline: PhysicsPipeline,
    params: IntegrationParameters,
    islands: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd: CCDSolver,

    ground_collider: ColliderHandle,
    zones: Vec<(ColliderHandle, Zone)>,
    stumps: RigidBodyHandle,
    bail: RigidBodyHandle,
    wicket_colliders: [ColliderHandle; 2],
    parts: Vec<(Part, RigidBodyHandle, Vec2)>, // part, body, half size (px)
    bat: RigidBodyHandle,
    bat_colliders: Vec<ColliderHandle>,
    bat_grab_local: Point<Real>,
    hinges: Vec<Hinge>,
    /// Each joint's anchor in its second body's frame (JOINTS order), for drawing.
    joint_points: Vec<(RigidBodyHandle, Point<Real>)>,

    balls: Vec<Ball>,
    bursts: Vec<Burst>,
    time: f32,
    sim_debt: f32,
    last_ball_time: f32,
    ball_count: u32,
    ball_played_count: u32,
    score: u32,
    game_over: bool,
}

fn rot_of(bodies: &RigidBodySet, h: RigidBodyHandle) -> f32 {
    bodies[h].rotation().angle()
}

impl World {
    fn new() -> Self {
        let mut bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();
        let mut joints = ImpulseJointSet::new();

        // Ground: static box, and the anchor for the batsman's feet.
        let ground = bodies.insert(RigidBodyBuilder::fixed());
        let ground_collider = colliders.insert_with_parent(
            ColliderBuilder::cuboid(m(854.9), m(63.0))
                .translation(vector![m(180.6), m(307.1)])
                .friction(0.7)
                .restitution(0.2)
                .restitution_combine_rule(CoefficientCombineRule::Max)
                .collision_groups(InteractionGroups::new(
                    G_GROUND,
                    Group::ALL.difference(G_BODY),
                )),
            ground,
            &mut bodies,
        );

        let zones = ZONES
            .iter()
            .map(|&((cx, cy), (hx, hy), z)| {
                let c = colliders.insert(
                    ColliderBuilder::cuboid(m(hx), m(hy))
                        .translation(vector![m(cx), m(cy)])
                        .sensor(true)
                        .collision_groups(InteractionGroups::new(G_ZONE, G_BALL)),
                );
                (c, z)
            })
            .collect();

        // Wicket.
        let wicket_groups = InteractionGroups::new(G_WICKET, G_GROUND | G_BALL | G_WICKET);
        let stumps = bodies.insert(
            RigidBodyBuilder::dynamic()
                .translation(vector![m(-411.4), m(159.1)])
                .sleeping(true)
                .ccd_enabled(true),
        );
        let stumps_col = colliders.insert_with_parent(
            ColliderBuilder::cuboid(m(4.4), m(93.9))
                .density(1.0)
                .friction(0.5)
                .restitution(0.3)
                .restitution_combine_rule(CoefficientCombineRule::Max)
                .collision_groups(wicket_groups),
            stumps,
            &mut bodies,
        );
        let bail = bodies.insert(
            RigidBodyBuilder::dynamic()
                .translation(vector![m(-411.5), m(67.8)])
                .lock_rotations()
                .sleeping(true)
                .ccd_enabled(true),
        );
        let bail_col = colliders.insert_with_parent(
            ColliderBuilder::ball(m(4.7))
                .density(1.0)
                .friction(0.2)
                .restitution(0.3)
                .restitution_combine_rule(CoefficientCombineRule::Max)
                .collision_groups(wicket_groups),
            bail,
            &mut bodies,
        );

        // Batsman. Body parts collide with nothing; joints hold them together.
        let mut parts = Vec::new();
        for def in &PARTS {
            let mut rb = RigidBodyBuilder::dynamic()
                .translation(vector![m(def.center.0), m(def.center.1)])
                .rotation(def.rot_deg.to_radians())
                .angular_damping(def.ang_damping)
                .can_sleep(false);
            if def.part == Part::Head {
                rb = rb.lock_rotations();
            }
            let h = bodies.insert(rb);
            colliders.insert_with_parent(
                ColliderBuilder::cuboid(m(def.half.0), m(def.half.1))
                    .density(def.density)
                    .friction(0.2)
                    .collision_groups(InteractionGroups::new(G_BODY, Group::NONE)),
                h,
                &mut bodies,
            );
            parts.push((def.part, h, vec2(def.half.0, def.half.1)));
        }

        // Bat.
        let bat = bodies.insert(
            RigidBodyBuilder::dynamic()
                .translation(vector![m(BAT_ORIGIN.0), m(BAT_ORIGIN.1)])
                .ccd_enabled(true)
                .can_sleep(false),
        );
        let bat_colliders = [&BAT_HANDLE[..], &BAT_BLADE[..]]
            .iter()
            .map(|poly| {
                let pts: Vec<Point<Real>> = poly.iter().map(|&(x, y)| point![m(x), m(y)]).collect();
                colliders.insert_with_parent(
                    ColliderBuilder::convex_hull(&pts)
                        .expect("bat outline is convex")
                        .density(2.0)
                        .friction(0.2)
                        .restitution(0.5)
                        .restitution_combine_rule(CoefficientCombineRule::Max)
                        .collision_groups(InteractionGroups::new(G_BAT, G_BALL)),
                    bat,
                    &mut bodies,
                )
            })
            .collect();
        let bat_grab_local = bodies[bat]
            .position()
            .inverse_transform_point(&point![m(BAT_GRAB.0), m(BAT_GRAB.1)]);

        // Joints. Limits are relative to the starting pose, so the reference
        // rotation goes into body1's joint frame.
        let handle_of = |a: Anchor| match a {
            Anchor::Ground => ground,
            Anchor::Bat => bat,
            Anchor::Part(p) => parts.iter().find(|(q, ..)| *q == p).unwrap().1,
        };
        let mut hinges = Vec::new();
        let mut joint_points = Vec::new();
        for &(a1, a2, (ax, ay), (lo, hi), spring) in &JOINTS {
            let (h1, h2) = (handle_of(a1), handle_of(a2));
            let anchor = point![m(ax), m(ay)];
            let p1 = bodies[h1].position().inverse_transform_point(&anchor);
            let p2 = bodies[h2].position().inverse_transform_point(&anchor);
            joint_points.push((h2, p2));
            let rest = rot_of(&bodies, h2) - rot_of(&bodies, h1);
            let mut builder = RevoluteJointBuilder::new()
                .local_anchor1(p1)
                .local_anchor2(p2);
            if lo == hi {
                // A locked joint never gets near ±π, so the engine's limit is fine.
                builder = builder.limits([lo, hi]);
            }
            let mut joint: GenericJoint = builder.build().into();
            joint.local_frame1 = Isometry::new(p1.coords, rest);
            joint.local_frame2 = Isometry::new(p2.coords, 0.0);
            joints.insert(h1, h2, joint, true);
            if lo != hi {
                hinges.push(Hinge {
                    b1: h1,
                    b2: h2,
                    rest,
                    limits: (lo, hi),
                    spring,
                    angle: 0.0,
                    last_wrapped: 0.0,
                });
            }
        }

        Self {
            pipeline: PhysicsPipeline::new(),
            params: IntegrationParameters {
                dt: SUB_DT,
                ..Default::default()
            },
            islands: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies,
            colliders,
            joints,
            multibody_joints: MultibodyJointSet::new(),
            ccd: CCDSolver::new(),
            ground_collider,
            zones,
            stumps,
            bail,
            wicket_colliders: [stumps_col, bail_col],
            parts,
            bat,
            bat_colliders,
            bat_grab_local,
            hinges,
            joint_points,
            balls: Vec::new(),
            bursts: Vec::new(),
            time: 0.0,
            sim_debt: 0.0,
            last_ball_time: 0.0,
            ball_count: 0,
            ball_played_count: 0,
            score: 0,
            game_over: false,
        }
    }

    fn strike_rate(&self) -> f32 {
        if self.ball_played_count == 0 {
            return 0.0;
        }
        ((self.score as f32 / self.ball_played_count as f32) * 100.0).floor() / 100.0
    }

    fn total_score(&self) -> u32 {
        (self.score as f32 * self.strike_rate()).floor() as u32
    }

    fn bowl(&mut self) {
        self.ball_count += 1;
        let x: f32 = 750.0 + rand::gen_range(0.0, 1.0) * 100.0;
        let y: f32 = -150.0 + (rand::gen_range(0.0, 1.0) - 0.5) * 200.0;
        let theta = ((y + 150.0) / (x - 750.0)).atan();
        let vx = -10.0 * theta.cos() - 10.0;
        let vy = -5.0 * theta.sin() + 2.5;

        let body = self.bodies.insert(
            RigidBodyBuilder::dynamic()
                .translation(vector![m(x), m(y)])
                .linvel(vector![vx, vy])
                .angular_damping(0.8)
                .ccd_enabled(true),
        );
        let collider = self.colliders.insert_with_parent(
            ColliderBuilder::ball(m(BALL_RADIUS_PX))
                .density(2.0)
                .friction(0.6)
                .restitution(0.5)
                .restitution_combine_rule(CoefficientCombineRule::Max)
                .collision_groups(InteractionGroups::new(
                    G_BALL,
                    G_GROUND | G_BAT | G_WICKET | G_BALL | G_ZONE,
                ))
                .active_events(ActiveEvents::COLLISION_EVENTS),
            body,
            &mut self.bodies,
        );

        // One-off spin kick, limited to the quarter turn per step a classic
        // Box2D solver allows.
        let rb = &mut self.bodies[body];
        let torque = vy.atan2(vx).to_degrees() * 20.0 - vx * 10.0 + 100.0;
        let inv_i = rb
            .mass_properties()
            .effective_world_inv_inertia_sqrt
            .powi(2);
        let max_spin = std::f32::consts::FRAC_PI_2 / SIM_PER_FRAME;
        rb.set_angvel(
            (torque * SIM_PER_FRAME * inv_i).clamp(-max_spin, max_spin),
            true,
        );

        self.balls.push(Ball {
            body,
            collider,
            hit: false,
            scored: false,
            on_ground: false,
        });
    }

    fn remove_ball(&mut self, i: usize) {
        let b = self.balls.remove(i);
        self.bodies.remove(
            b.body,
            &mut self.islands,
            &mut self.colliders,
            &mut self.joints,
            &mut self.multibody_joints,
            true,
        );
    }

    fn touching(&self, a: ColliderHandle, b: ColliderHandle) -> bool {
        self.narrow_phase
            .contact_pair(a, b)
            .is_some_and(|p| p.has_any_active_contact)
    }

    fn sensor_hit(&self, sensor: ColliderHandle, c: ColliderHandle) -> bool {
        self.narrow_phase.intersection_pair(sensor, c) == Some(true)
    }

    /// Soft mouse joint (5 Hz, damping 0.7, max force 200 × mass) pulling the
    /// grab point on the bat towards `target` (metres).
    fn drive_bat(&mut self, target: Point<Real>, h: f32) {
        let rb = &mut self.bodies[self.bat];
        let mass = rb.mass();
        let inv_m = 1.0 / mass;
        let inv_i = rb
            .mass_properties()
            .effective_world_inv_inertia_sqrt
            .powi(2);
        let p = rb.position() * self.bat_grab_local;
        let r = p - rb.center_of_mass();
        let w = rb.angvel();
        let v = rb.linvel() + vector![-w * r.y, w * r.x];

        let omega = std::f32::consts::TAU * 5.0;
        let (k, c) = (mass * omega * omega, 2.0 * mass * 0.7 * omega);
        let gamma = 1.0 / (h * (c + h * k));
        let beta = h * k * gamma;

        let k11 = inv_m + inv_i * r.y * r.y + gamma;
        let k12 = -inv_i * r.x * r.y;
        let k22 = inv_m + inv_i * r.x * r.x + gamma;
        let det = k11 * k22 - k12 * k12;
        let rhs = v + (p - target) * beta;
        let mut impulse = -vector![
            (k22 * rhs.x - k12 * rhs.y) / det,
            (k11 * rhs.y - k12 * rhs.x) / det
        ];
        let max = 200.0 * mass * h;
        if impulse.norm() > max {
            impulse *= max / impulse.norm();
        }
        rb.apply_impulse_at_point(impulse, p, true);
    }

    /// Joint springs and limits, on the unwrapped joint angle.
    /// Springs: torque of size |k·angle + d·speed| pushing the joint back to rest.
    /// Limits: an angular impulse that stops (and gently undoes) over-rotation.
    fn apply_hinges(&mut self, h: f32) {
        use std::f32::consts::{PI, TAU};
        for j in &mut self.hinges {
            let (b1, b2) = (&self.bodies[j.b1], &self.bodies[j.b2]);
            let wrapped = b2.rotation().angle() - b1.rotation().angle() - j.rest;
            let delta = (wrapped - j.last_wrapped + PI).rem_euclid(TAU) - PI;
            j.last_wrapped = wrapped;
            j.angle += delta;

            let w_rel = b2.angvel() - b1.angvel();
            let inv_i1 = b1
                .mass_properties()
                .effective_world_inv_inertia_sqrt
                .powi(2);
            let inv_i2 = b2
                .mass_properties()
                .effective_world_inv_inertia_sqrt
                .powi(2);
            let mut impulse = 0.0;

            if let Some((k, d)) = j.spring {
                impulse += (j.angle * k + w_rel * d).abs() * -j.angle.signum() * h;
            }
            let (lo, hi) = j.limits;
            let error = if j.angle < lo {
                j.angle - lo
            } else if j.angle > hi {
                j.angle - hi
            } else {
                0.0
            };
            if error != 0.0 && inv_i1 + inv_i2 > 0.0 {
                let w_after = w_rel + impulse * (inv_i1 + inv_i2);
                // Only resist motion that makes the violation worse, plus a little push back.
                let target = -0.3 * error / h;
                let needed = (target - w_after) / (inv_i1 + inv_i2);
                impulse += if error < 0.0 {
                    needed.max(0.0)
                } else {
                    needed.min(0.0)
                };
            }
            if impulse != 0.0 {
                self.bodies[j.b2].apply_torque_impulse(impulse, true);
                self.bodies[j.b1].apply_torque_impulse(-impulse, true);
            }
        }
    }

    fn burst(&mut self, pos: Vec2, label: &'static str, color: Color) {
        self.bursts.push(Burst {
            pos,
            label,
            color,
            age: 0.0,
        });
    }

    /// Apply the scoring rules to every ball after a physics substep.
    fn judge(&mut self, sfx: &Sfx) {
        let mut i = 0;
        while i < self.balls.len() {
            let c = self.balls[i].collider;
            let t = *self.bodies[self.balls[i].body].translation();
            let pos = vec2(t.x, t.y) * PX_PER_M;

            if !self.balls[i].hit && self.bat_colliders.iter().any(|&b| self.touching(b, c)) {
                self.balls[i].hit = true;
                sfx.play(&sfx.thud, 0.9);
            }
            let grounded = self.touching(self.ground_collider, c);
            if grounded && !self.balls[i].on_ground {
                sfx.play(&sfx.bump, 0.5);
            }
            self.balls[i].on_ground = grounded;

            // Balls that touch each other both count as hit.
            for j in 0..self.balls.len() {
                if j != i && self.balls[i].hit && self.touching(c, self.balls[j].collider) {
                    self.balls[j].hit = true;
                }
            }

            if !self.game_over && !self.balls[i].scored {
                let zone = self
                    .zones
                    .iter()
                    .find(|(z, _)| self.sensor_hit(*z, c))
                    .map(|&(_, z)| z);
                let hit = self.balls[i].hit;
                match zone {
                    Some(Zone::Runs(n)) if hit => {
                        self.score += n;
                        self.ball_played_count = self.ball_count;
                        let (label, color, s) = match n {
                            1 => ("1", Color::from_hex(0x7ee081), &sfx.chime1),
                            2 => ("2", Color::from_hex(0x5ec8ff), &sfx.chime2),
                            4 => ("4", Color::from_hex(0xffc93c), &sfx.chime4),
                            _ => ("6", Color::from_hex(0xff5fa2), &sfx.chime6),
                        };
                        sfx.play(s, 0.8);
                        self.burst(pos, label, color);
                        self.remove_ball(i);
                        continue;
                    }
                    Some(Zone::Out) if hit => {
                        self.game_over = true;
                        self.balls[i].scored = true;
                        self.ball_played_count = self.ball_count;
                        sfx.play(&sfx.appeal, 0.9);
                        self.burst(pos, "OUT", Color::from_hex(0xff4a3d));
                        self.remove_ball(i);
                        continue;
                    }
                    _ => {}
                }
                if self.wicket_colliders.iter().any(|&w| self.touching(w, c)) {
                    self.game_over = true;
                    self.balls[i].scored = true;
                    self.ball_played_count = self.ball_count;
                    self.bodies[self.bail].wake_up(true);
                    self.bodies[self.stumps].wake_up(true);
                    sfx.play(&sfx.stumps, 1.0);
                    sfx.play(&sfx.appeal, 0.9);
                }
            }

            // Tidy up balls that have rolled or fallen out of the world.
            if pos.y > 1200.0 || pos.x.abs() > 1200.0 {
                self.remove_ball(i);
                continue;
            }
            i += 1;
        }
    }

    /// One physics substep: bat drive, joint springs, then the solver.
    fn substep(&mut self, cursor_m: Point<Real>) {
        let h = self.params.dt;
        self.drive_bat(cursor_m, h);
        self.apply_hinges(h);
        self.pipeline.step(
            &vector![0.0, GRAVITY],
            &self.params,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &mut self.multibody_joints,
            &mut self.ccd,
            None,
            &(),
            &(),
        );
    }

    /// Advance by `real_dt` seconds of wall-clock time, in fixed physics
    /// substeps, so motion is smooth at any display refresh rate.
    fn advance(&mut self, real_dt: f32, cursor_m: Point<Real>, sfx: &Sfx) {
        self.sim_debt += real_dt * TIME_SCALE;
        while self.sim_debt >= SUB_DT {
            self.sim_debt -= SUB_DT;
            self.time += SUB_DT;
            if !self.game_over && self.time - self.last_ball_time > BALL_INTERVAL {
                self.last_ball_time = self.time;
                self.bowl();
            }
            self.substep(cursor_m);
            self.judge(sfx);
            for b in &mut self.bursts {
                b.age += SUB_DT;
            }
        }
        self.bursts.retain(|b| b.age < 20.0 * SIM_PER_FRAME);
    }
}

// ---------- view ----------

/// Maps world px -> stage px -> window px (letterboxed, aspect preserved).
#[derive(Clone, Copy)]
struct View {
    k: f32,
    off: Vec2,
}

impl View {
    fn current() -> Self {
        let k = (screen_width() / STAGE_W).min(screen_height() / STAGE_H);
        let off = vec2(
            (screen_width() - STAGE_W * k) / 2.0,
            (screen_height() - STAGE_H * k) / 2.0,
        );
        Self { k, off }
    }
    fn stage(&self, p: Vec2) -> Vec2 {
        self.off + p * self.k
    }
    fn world(&self, p: Vec2) -> Vec2 {
        let s = vec2(
            VIEW_ORIGIN.0 + VIEW_SCALE * (p.x - VIEW_TARGET.0),
            VIEW_ORIGIN.1 + VIEW_SCALE * (p.y - VIEW_TARGET.1),
        );
        self.stage(s)
    }
    fn len(&self, world_px: f32) -> f32 {
        world_px * VIEW_SCALE * self.k
    }
    fn screen_to_world(&self, p: Vec2) -> Vec2 {
        let s = (p - self.off) / self.k;
        vec2(
            (s.x - VIEW_ORIGIN.0) / VIEW_SCALE + VIEW_TARGET.0,
            (s.y - VIEW_ORIGIN.1) / VIEW_SCALE + VIEW_TARGET.1,
        )
    }
}

fn local_to_world(center: Vec2, angle: f32, p: Vec2) -> Vec2 {
    let (s, c) = angle.sin_cos();
    center + vec2(p.x * c - p.y * s, p.x * s + p.y * c)
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Ragdoll Cricket".to_owned(),
        window_width: (STAGE_W * 2.0) as i32,
        window_height: (STAGE_H * 2.0) as i32,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

/// Snapshot mode's "player": keep the bat's grab point on the most urgent ball.
fn autoplay_cursor(world: &World) -> Vec2 {
    let target = world
        .balls
        .iter()
        .filter(|b| !b.hit)
        .map(|b| {
            let t = world.bodies[b.body].translation();
            vec2(t.x, t.y) * PX_PER_M
        })
        .filter(|p| p.x > -60.0)
        .min_by(|a, b| a.x.total_cmp(&b.x));
    match target {
        Some(p) if p.x < 250.0 => vec2(-20.0, p.y.clamp(-80.0, 200.0) + 30.0),
        _ => vec2(-110.0, 110.0),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(miniquad::date::now() as u64);
    let sfx = Sfx::load().await;
    let art = art::Art::load();
    let mut world = World::new();
    let mut screen = Screen::Title;
    let mut best = 0;
    let mut clock = 0.0;
    // Dev aid: RC_SNAPSHOT=<dir> auto-plays a short innings and saves screenshots.
    let snapshot_dir = std::env::var("RC_SNAPSHOT").ok();
    let mut frame_no = 0u32;
    // Dev aid: RC_PROFILE=1 prints where frame time goes.
    let profile = std::env::var("RC_PROFILE").is_ok();
    let (mut t_phys, mut t_draw, mut t_frame, mut n_prof) = (0.0f64, 0.0f64, 0.0f64, 0u32);
    let mut shots_after_out = 0u32;

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        let view = View::current();
        let mut cursor = view.screen_to_world(Vec2::from(mouse_position()));
        if snapshot_dir.is_some() {
            cursor = autoplay_cursor(&world);
        }
        let cursor_m = point![m(cursor.x), m(cursor.y)];

        let dt = if snapshot_dir.is_some() { FRAME_DT } else { get_frame_time().min(0.1) };
        clock += dt;
        if screen != Screen::Title {
            let t0 = std::time::Instant::now();
            world.advance(dt, cursor_m, &sfx);
            t_phys += t0.elapsed().as_secs_f64();
            if world.game_over && screen == Screen::Playing {
                best = best.max(world.total_score());
                screen = Screen::Ended;
            }
        }

        let t0 = std::time::Instant::now();
        art::draw_scene(&art, &world, &view, clock, best);
        t_draw += t0.elapsed().as_secs_f64();

        match screen {
            Screen::Title => {
                if art::draw_title(&art, &view) {
                    world = World::new();
                    screen = Screen::Playing;
                }
            }
            Screen::Ended => {
                if art::draw_end(&art, &world, &view) {
                    world = World::new();
                    screen = Screen::Playing;
                }
            }
            Screen::Playing => {}
        }

        if let Some(dir) = &snapshot_dir {
            frame_no += 1;
            let shot = |name: &str| get_screen_data().export_png(&format!("{dir}/{name}.png"));
            match (screen, frame_no) {
                (Screen::Title, 5) => shot("title"),
                (Screen::Title, 6) => {
                    world = World::new();
                    screen = Screen::Playing;
                }
                (Screen::Playing, f) if f % 45 == 0 => shot(&format!("play_{f:04}")),
                (Screen::Ended, _) => {
                    shots_after_out += 1;
                    if shots_after_out == 2 {
                        shot("end");
                    }
                    if shots_after_out > 2 {
                        break;
                    }
                }
                _ => {}
            }
            if frame_no > 900 {
                break;
            }
        }
        if profile {
            t_frame += get_frame_time() as f64;
            n_prof += 1;
            if n_prof == 120 {
                let ms = |t: f64| t * 1000.0 / n_prof as f64;
                println!(
                    "fps {:.0} | frame {:.2} ms | physics {:.2} ms | draw (CPU) {:.2} ms | balls {}",
                    n_prof as f64 / t_frame, ms(t_frame), ms(t_phys), ms(t_draw), world.balls.len()
                );
                (t_phys, t_draw, t_frame, n_prof) = (0.0, 0.0, 0.0, 0);
            }
        }
        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chest_pos(w: &World) -> Vec2 {
        let h = w.parts.iter().find(|(p, ..)| *p == Part::Chest).unwrap().1;
        let t = w.bodies[h].translation();
        vec2(t.x, t.y) * PX_PER_M
    }

    fn bat_pos(w: &World) -> Vec2 {
        let t = w.bodies[w.bat].translation();
        vec2(t.x, t.y) * PX_PER_M
    }

    /// Simulate `seconds` of game time with the cursor following `path(t)` (world px).
    fn run(w: &mut World, seconds: f32, path: impl Fn(f32) -> Vec2) {
        let frames = (seconds / SIM_PER_FRAME) as usize;
        for f in 0..frames {
            let c = path(f as f32 * SIM_PER_FRAME);
            for _ in 0..SUBSTEPS {
                w.substep(point![m(c.x), m(c.y)]);
            }
        }
    }

    #[test]
    fn batsman_stays_upright_when_idle() {
        let mut w = World::new();
        let start = chest_pos(&w);
        run(&mut w, 5.0, |_| vec2(BAT_GRAB.0, BAT_GRAB.1));
        let end = chest_pos(&w);
        println!("chest {start:?} -> {end:?}, bat {:?}", bat_pos(&w));
        assert!(end.is_finite());
        assert!(
            start.distance(end) < 60.0,
            "chest drifted {}",
            start.distance(end)
        );
    }

    #[test]
    fn bat_follows_cursor_and_ragdoll_survives_wild_swings() {
        let mut w = World::new();
        run(&mut w, 1.0, |_| vec2(BAT_GRAB.0, BAT_GRAB.1));
        // Wave the cursor around for a few seconds.
        run(&mut w, 4.0, |t| {
            vec2(
                -60.0 + 150.0 * (t * 6.0).sin(),
                40.0 + 120.0 * (t * 4.3).cos(),
            )
        });
        // Then come back to a comfortable spot in front of the pads.
        let target = vec2(-40.0, 100.0);
        run(&mut w, 1.5, |_| target);
        let grab = w.bodies[w.bat].position() * w.bat_grab_local * PX_PER_M;
        let grab = vec2(grab.x, grab.y);
        println!(
            "grab point {grab:?} target {target:?} chest {:?}",
            chest_pos(&w)
        );
        for (_, h, _) in &w.parts {
            assert!(w.bodies[*h].translation().x.is_finite());
        }
        assert!(
            grab.distance(target) < 30.0,
            "bat didn't follow: {}",
            grab.distance(target)
        );
        assert!(chest_pos(&w).distance(vec2(-173.2, 7.5)) < 150.0);
    }

    #[test]
    fn bowled_balls_reach_the_batsman() {
        rand::srand(42);
        for _ in 0..20 {
            let mut w = World::new();
            w.bowl();
            let mut met_bat = false;
            let frames = (6.0 / SIM_PER_FRAME) as usize;
            for _ in 0..frames {
                for _ in 0..SUBSTEPS {
                    w.substep(point![m(BAT_GRAB.0), m(BAT_GRAB.1)]);
                    let c = w.balls[0].collider;
                    let t = w.bodies[w.balls[0].body].translation();
                    met_bat |=
                        w.bat_colliders.iter().any(|&b| w.touching(b, c)) || t.x * PX_PER_M < -60.0;
                }
            }
            if !met_bat {
                let b = &w.bodies[w.balls[0].body];
                println!(
                    "miss: ball ended at {:?} vel {:?}",
                    b.translation() * PX_PER_M,
                    b.linvel()
                );
            }
            assert!(met_bat, "ball never reached the batsman");
        }
    }
}

#[cfg(test)]
mod diag {
    use super::*;
    #[test]
    fn joint_report() {
        let mut w = World::new();
        let frames = (4.0 / SIM_PER_FRAME) as usize;
        let mut worst: f32 = 0.0;
        for f in 0..frames {
            let t = f as f32 * SIM_PER_FRAME;
            let c = vec2(
                -60.0 + 150.0 * (t * 6.0).sin(),
                40.0 + 120.0 * (t * 4.3).cos(),
            );
            for _ in 0..SUBSTEPS {
                w.substep(point![m(c.x), m(c.y)]);
            }
            for j in &w.hinges {
                worst = worst.max(j.limits.0 - j.angle).max(j.angle - j.limits.1);
            }
        }
        println!("worst limit violation over wild swings: {worst:.3} rad");
        assert!(worst < 0.5);
    }
}
