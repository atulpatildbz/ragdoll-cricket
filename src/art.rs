//! All drawing: a cartoon stadium, an outlined ragdoll batsman, and the UI.
//! The scene is drawn procedurally; the batsman is painted parts from assets/batsman/.

use super::*;

// Palette.
const INK: Color = Color::new(0.11, 0.13, 0.21, 1.0);
const KIT: Color = Color::new(0.72, 0.15, 0.25, 1.0);
const KIT_DARK: Color = Color::new(0.55, 0.11, 0.19, 1.0);
const GOLD: Color = Color::new(1.0, 0.78, 0.27, 1.0);
const CREAM: Color = Color::new(0.96, 0.94, 0.89, 1.0);
const CREAM_SHADE: Color = Color::new(0.84, 0.82, 0.76, 1.0);
const PAD: Color = Color::new(0.99, 0.99, 0.97, 1.0);
const PAD_RIB: Color = Color::new(0.82, 0.83, 0.8, 1.0);
const SHOE: Color = Color::new(0.93, 0.92, 0.87, 1.0);
const SOLE: Color = Color::new(0.62, 0.63, 0.62, 1.0);
const STRAP: Color = Color::new(0.45, 0.5, 0.56, 1.0);

pub struct Art {
    font: Font,
    kit: Kit,
}

/// A painted body part, pinned to the rig by two pixels of its image.
struct Sprite {
    tex: Texture2D,
    a: Vec2,
    b: Vec2,
}

impl Sprite {
    fn new(png: &[u8], a: (f32, f32), b: (f32, f32)) -> Self {
        let tex = Texture2D::from_file_with_format(png, Some(ImageFormat::Png));
        tex.set_filter(FilterMode::Linear);
        Self { tex, a: vec2(a.0, a.1), b: vec2(b.0, b.1) }
    }

    /// Draw so that image pixels `a` and `b` land on screen points `pa` and
    /// `pb`: uniform scale plus rotation, so the art never squashes.
    fn draw(&self, pa: Vec2, pb: Vec2, tint: Color) {
        let (img, scr) = (self.b - self.a, pb - pa);
        let k = scr.length() / img.length();
        let rotation = scr.y.atan2(scr.x) - img.y.atan2(img.x);
        let top_left = pa - self.a * k;
        draw_texture_ex(&self.tex, top_left.x, top_left.y, tint, DrawTextureParams {
            dest_size: Some(self.tex.size() * k),
            rotation,
            pivot: Some(pa),
            ..Default::default()
        });
    }
}

/// The batsman's parts, cut from assets/batsman/. Anchors are joint centres
/// (limbs) or a pair of points matched to body-local positions (head, torso, bat).
struct Kit {
    head: Sprite,
    torso: Sprite,
    upper_arm: Sprite,
    forearm: Sprite,
    bat: Sprite,
}

impl Kit {
    fn load() -> Self {
        Self {
            head: Sprite::new(include_bytes!("../assets/batsman/head.png"), (84.0, 85.0), (84.0, 185.0)),
            torso: Sprite::new(include_bytes!("../assets/batsman/torso.png"), (74.0, 2.0), (80.0, 162.0)),
            upper_arm: Sprite::new(include_bytes!("../assets/batsman/upper_arm.png"), (23.0, 23.0), (58.0, 93.0)),
            forearm: Sprite::new(include_bytes!("../assets/batsman/forearm.png"), (19.0, 25.0), (96.0, 47.0)),
            bat: Sprite::new(include_bytes!("../assets/batsman/bat.png"), (28.5, 2.0), (28.5, 391.0)),
        }
    }
}

impl Art {
    pub fn load() -> Self {
        let mut font = load_ttf_font_from_bytes(include_bytes!("../assets/LilitaOne-Regular.ttf"))
            .expect("bundled font loads");
        font.set_filter(FilterMode::Linear);
        Self { font, kit: Kit::load() }
    }

    /// Text centred on `center` (screen px), with an optional outline.
    fn text(&self, s: &str, center: Vec2, size: f32, fill: Color, outline: Option<(Color, f32)>) {
        let size_px = size.max(1.0) as u16;
        let dims = measure_text(s, Some(&self.font), size_px, 1.0);
        let x = center.x - dims.width / 2.0;
        let y = center.y - dims.height / 2.0 + dims.offset_y;
        if let Some((oc, w)) = outline {
            let steps = 12;
            for i in 0..steps {
                let a = i as f32 / steps as f32 * std::f32::consts::TAU;
                self.raw_text(s, x + a.cos() * w, y + a.sin() * w, size_px, oc);
            }
        }
        self.raw_text(s, x, y, size_px, fill);
    }

    fn raw_text(&self, s: &str, x: f32, y: f32, size: u16, color: Color) {
        draw_text_ex(s, x, y, TextParams { font: Some(&self.font), font_size: size, color, ..Default::default() });
    }
}

// ---------- primitives (screen space) ----------

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t)
}

fn shade(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}

fn capsule(a: Vec2, b: Vec2, r: f32, c: Color) {
    draw_line(a.x, a.y, b.x, b.y, r * 2.0, c);
    draw_circle(a.x, a.y, r, c);
    draw_circle(b.x, b.y, r, c);
}

fn inked_capsule(a: Vec2, b: Vec2, r: f32, ol: f32, c: Color) {
    capsule(a, b, r + ol, INK);
    capsule(a, b, r, c);
}

fn inked_circle(p: Vec2, r: f32, ol: f32, c: Color) {
    draw_circle(p.x, p.y, r + ol, INK);
    draw_circle(p.x, p.y, r, c);
}

/// Convex polygon fill.
fn poly(pts: &[Vec2], c: Color) {
    for i in 1..pts.len() - 1 {
        draw_triangle(pts[0], pts[i], pts[i + 1], c);
    }
}

/// Convex polygon with a rounded ink outline.
fn inked_poly(pts: &[Vec2], ol: f32, c: Color) {
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        draw_line(a.x, a.y, b.x, b.y, ol * 2.0, INK);
        draw_circle(a.x, a.y, ol, INK);
    }
    poly(pts, INK);
    // Shrinking isn't needed: the fill sits inside the stroked edge.
    poly(pts, c);
}

/// Rounded rectangle built from non-overlapping pieces, so translucent fills stay even.
fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    draw_rectangle(x + r, y, w - 2.0 * r, h, c);
    draw_rectangle(x, y + r, r, h - 2.0 * r, c);
    draw_rectangle(x + w - r, y + r, r, h - 2.0 * r, c);
    let n = 8;
    for (cx, cy, start) in [(x + r, y + r, 2.0), (x + w - r, y + r, 3.0), (x + w - r, y + h - r, 0.0), (x + r, y + h - r, 1.0)] {
        let center = vec2(cx, cy);
        for i in 0..n {
            let a0 = (start + i as f32 / n as f32) * std::f32::consts::FRAC_PI_2;
            let a1 = (start + (i + 1) as f32 / n as f32) * std::f32::consts::FRAC_PI_2;
            draw_triangle(center, center + vec2(a0.cos(), a0.sin()) * r, center + vec2(a1.cos(), a1.sin()) * r, c);
        }
    }
}

fn inked_rrect(x: f32, y: f32, w: f32, h: f32, r: f32, ol: f32, c: Color) {
    rrect(x - ol, y - ol, w + 2.0 * ol, h + 2.0 * ol, r + ol, INK);
    rrect(x, y, w, h, r, c);
}

fn ellipse(c: Vec2, rx: f32, ry: f32, color: Color) {
    let n = 28;
    for i in 0..n {
        let a0 = i as f32 / n as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / n as f32 * std::f32::consts::TAU;
        draw_triangle(c, c + vec2(a0.cos() * rx, a0.sin() * ry), c + vec2(a1.cos() * rx, a1.sin() * ry), color);
    }
}

/// Body-local (world px) point to screen.
fn at(view: &View, origin: Vec2, angle: f32, x: f32, y: f32) -> Vec2 {
    view.world(local_to_world(origin, angle, vec2(x, y)))
}

fn body_pose(world: &World, h: RigidBodyHandle) -> (Vec2, f32) {
    let rb = &world.bodies[h];
    (vec2(rb.translation().x, rb.translation().y) * PX_PER_M, rb.rotation().angle())
}

fn part_pose(world: &World, p: Part) -> (Vec2, f32) {
    let h = world.parts.iter().find(|(q, ..)| *q == p).unwrap().1;
    body_pose(world, h)
}

// ---------- scene ----------

pub fn draw_scene(art: &Art, world: &World, view: &View, time: f32, best: u32) {
    clear_background(INK);
    draw_sky(view, time);
    draw_stadium(art, view, time);
    draw_ground(view);
    draw_zones(art, view);
    draw_shadows(world, view);
    draw_wicket(world, view);
    draw_batsman(art, world, view);
    draw_balls(world, view);
    draw_bursts(art, world, view);
    draw_hud(art, world, view, best);
}

fn stage_rect(view: &View, x: f32, y: f32, w: f32, h: f32, c: Color) {
    let p = view.stage(vec2(x, y));
    draw_rectangle(p.x, p.y, w * view.k + 0.5, h * view.k + 0.5, c);
}

fn draw_sky(view: &View, time: f32) {
    let top = Color::from_hex(0x3a78e0);
    let bottom = Color::from_hex(0xc7e3ff);
    let bands = 48;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        stage_rect(view, 0.0, 270.0 * t, STAGE_W, 270.0 / bands as f32, mix(top, bottom, t.powf(1.3)));
    }
    // Sun with a soft glow.
    let sun = view.stage(vec2(120.0, 62.0));
    for (r, a) in [(46.0, 0.08), (34.0, 0.12), (24.0, 0.2)] {
        draw_circle(sun.x, sun.y, r * view.k, Color::new(1.0, 0.97, 0.82, a));
    }
    draw_circle(sun.x, sun.y, 15.0 * view.k, Color::from_hex(0xfff3c4));

    // Clouds drift slowly; opaque so overlapping puffs don't seam.
    for (i, &(x, y, s)) in [(60.0, 48.0, 1.0), (260.0, 30.0, 0.75), (420.0, 78.0, 0.9), (560.0, 40.0, 0.6)].iter().enumerate() {
        let span = STAGE_W + 140.0;
        let x = (x + time * (3.0 + i as f32 * 1.5)).rem_euclid(span) - 70.0;
        cloud(view, vec2(x, y), s);
    }
}

fn cloud(view: &View, at: Vec2, s: f32) {
    let k = view.k * s;
    let puffs = [(-22.0, 4.0, 12.0), (-6.0, -4.0, 16.0), (14.0, -1.0, 13.0), (28.0, 6.0, 9.0), (2.0, 8.0, 12.0)];
    let base = view.stage(at);
    for (dx, dy, r) in puffs {
        draw_circle(base.x + dx * k, base.y + (dy + 3.0) * k, r * k, Color::from_hex(0xd5e4fb));
    }
    for (dx, dy, r) in puffs {
        draw_circle(base.x + dx * k, base.y + dy * k, r * k, WHITE);
    }
}

fn draw_stadium(art: &Art, view: &View, time: f32) {
    // Floodlight towers behind everything.
    for x in [34.0, 556.0] {
        stage_rect(view, x - 2.0, 72.0, 4.0, 110.0, Color::from_hex(0x6b7896));
        stage_rect(view, x - 15.0, 60.0, 30.0, 16.0, Color::from_hex(0x46526e));
        for i in 0..4 {
            for j in 0..2 {
                stage_rect(view, x - 13.0 + i as f32 * 7.0, 62.0 + j as f32 * 7.0, 5.0, 5.0, Color::from_hex(0xfff6d8));
            }
        }
        let p = view.stage(vec2(x, 68.0));
        draw_circle(p.x, p.y, 26.0 * view.k, Color::new(1.0, 0.97, 0.85, 0.12));
    }

    // Stand: roof, tiers and a gently bobbing crowd.
    let (top, bottom) = (168.0, 246.0);
    let roof = [vec2(-10.0, 152.0), vec2(600.0, 152.0), vec2(600.0, top), vec2(-10.0, top)].map(|p| view.stage(p));
    poly(&roof, Color::from_hex(0x2b3452));
    stage_rect(view, -10.0, top - 2.0, 610.0, 3.0, Color::from_hex(0x8590ad));
    stage_rect(view, -10.0, top, 610.0, bottom - top, Color::from_hex(0x45557c));
    let tiers = 4;
    let crowd = [0xf25f5c, 0xffe066, 0x70c1b3, 0xffffff, 0xf7a072, 0x9bc1ff];
    let mut seed = 11u32;
    for row in 0..tiers {
        let y0 = top + 4.0 + row as f32 * 18.0;
        stage_rect(view, -10.0, y0 + 13.0, 610.0, 2.0, Color::from_hex(0x3a496b));
        let mut x = 2.0;
        while x < STAGE_W {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let c = mix(Color::from_hex(crowd[(seed >> 16) as usize % crowd.len()]), Color::from_hex(0x45557c), 0.55);
            let bob = ((time * 4.0 + x * 0.21 + row as f32 * 1.7).sin()).max(0.0) * 1.2;
            let head = view.stage(vec2(x, y0 + 4.0 - bob));
            let body = view.stage(vec2(x - 2.2, y0 + 6.5 - bob));
            draw_rectangle(body.x, body.y, 4.4 * view.k, 6.0 * view.k, shade(c, 0.8));
            draw_circle(head.x, head.y, 2.2 * view.k, c);
            x += 5.5 + ((seed >> 8) % 3) as f32;
        }
    }

    // Advertising hoardings along the boundary.
    let words = ["HOWZAT!", "RAGDOLL", "SIXER", "CRICKET", "WILLOW", "GOOD LENGTH"];
    let colors = [(0x1f4fa0, 0xffffff), (0xffc845, 0x8d1b30), (0xe94e3c, 0xffffff), (0xffffff, 0x1f4fa0)];
    let w = 98.0;
    for i in 0..7 {
        let x = -20.0 + i as f32 * w;
        let (bg, fg) = colors[i % colors.len()];
        stage_rect(view, x, 246.0, w - 1.0, 19.5, Color::from_hex(bg));
        stage_rect(view, x, 246.0, w - 1.0, 2.0, shade(Color::from_hex(bg), 1.15));
        art.text(words[i % words.len()], view.stage(vec2(x + w / 2.0, 256.0)), 12.0 * view.k, Color::from_hex(fg), None);
    }
}

fn draw_ground(view: &View) {
    let top = 265.0;
    // Turf shading, darker towards the camera.
    let bands = 16;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        stage_rect(view, 0.0, top + (STAGE_H - top) * t, STAGE_W, (STAGE_H - top) / bands as f32, mix(Color::from_hex(0x5cb85c), Color::from_hex(0x2f7d3a), t));
    }
    // Mowing stripes.
    let mut x = -30.0;
    let mut i = 0;
    while x < STAGE_W {
        if i % 2 == 0 {
            let pts = [vec2(x, top), vec2(x + 40.0, top), vec2(x + 40.0 - 26.0, STAGE_H), vec2(x - 26.0, STAGE_H)].map(|p| view.stage(p));
            poly(&pts, Color::new(1.0, 1.0, 1.0, 0.05));
        }
        x += 40.0;
        i += 1;
    }
    // Pitch, receding slightly.
    let pitch = [vec2(40.0, top), vec2(STAGE_W + 10.0, top), vec2(STAGE_W + 10.0, top + 16.0), vec2(30.0, top + 16.0)].map(|p| view.stage(p));
    poly(&pitch, Color::from_hex(0xd9c48e));
    stage_rect(view, 30.0, top + 14.0, STAGE_W, 2.0, Color::from_hex(0xbfa96f));
    // Popping crease.
    let crease = view.world(vec2(-330.0, 244.0)).x;
    let c0 = (crease - view.off.x) / view.k;
    let pts = [vec2(c0 - 1.0, top), vec2(c0 + 1.0, top), vec2(c0 - 2.0, top + 16.0), vec2(c0 - 4.0, top + 16.0)].map(|p| view.stage(p));
    poly(&pts, WHITE);
    stage_rect(view, 0.0, top, STAGE_W, 1.0, Color::new(0.0, 0.0, 0.0, 0.15));
}

fn zone_color(z: Zone) -> (Color, &'static str) {
    match z {
        Zone::Runs(1) => (Color::from_hex(0x3fbf6a), "1"),
        Zone::Runs(2) => (Color::from_hex(0x2f9fe0), "2"),
        Zone::Runs(4) => (Color::from_hex(0xf5a524), "4"),
        Zone::Runs(_) => (Color::from_hex(0xe0399a), "6"),
        Zone::Out => (Color::from_hex(0xe23b2e), "OUT"),
    }
}

fn draw_zones(art: &Art, view: &View) {
    let to_stage = |x: f32, y: f32| {
        vec2(VIEW_ORIGIN.0 + VIEW_SCALE * (x - VIEW_TARGET.0), VIEW_ORIGIN.1 + VIEW_SCALE * (y - VIEW_TARGET.1))
    };
    for &((cx, cy), (hx, hy), z) in &ZONES {
        let (color, label) = zone_color(z);
        // Visible part of the zone, in stage px.
        let a = to_stage(cx - hx, cy - hy).max(vec2(0.0, 0.0));
        let b = to_stage(cx + hx, cy + hy).min(vec2(STAGE_W, STAGE_H));
        if a.x >= b.x || a.y >= b.y {
            continue;
        }
        stage_rect(view, a.x, a.y, b.x - a.x, b.y - a.y, Color::new(color.r, color.g, color.b, 0.88));
        // Net texture.
        let mut d = a.x - (b.y - a.y);
        while d < b.x {
            let p0 = vec2(d.max(a.x), a.y + (d.max(a.x) - d));
            let p1 = vec2((d + (b.y - a.y)).min(b.x), a.y + ((d + (b.y - a.y)).min(b.x) - d));
            let (s0, s1) = (view.stage(p0), view.stage(p1));
            draw_line(s0.x, s0.y, s1.x, s1.y, 0.6 * view.k, Color::new(1.0, 1.0, 1.0, 0.25));
            d += 6.0;
        }
        // Edge facing the field.
        let horizontal = hx > hy;
        let edge = if horizontal {
            (vec2(a.x, b.y), vec2(b.x, b.y))
        } else if cx > 0.0 {
            (vec2(a.x, a.y), vec2(a.x, b.y))
        } else {
            (vec2(b.x, a.y), vec2(b.x, b.y))
        };
        let (e0, e1) = (view.stage(edge.0), view.stage(edge.1));
        draw_line(e0.x, e0.y, e1.x, e1.y, 1.5 * view.k, Color::new(1.0, 1.0, 1.0, 0.85));

        // Badges, poking into the field so they're readable.
        let badge_w = if label == "OUT" { 30.0 } else { 18.0 };
        let centers: Vec<Vec2> = if horizontal {
            let n = ((b.x - a.x) / 120.0).round().max(1.0) as i32;
            (0..n).map(|i| vec2(a.x + (b.x - a.x) * (i as f32 + 0.5) / n as f32, (a.y + b.y) / 2.0 + 2.0)).collect()
        } else {
            let x = if cx > 0.0 { a.x - 4.0 } else { b.x + 4.0 };
            vec2(x, ((a.y + b.y) / 2.0).clamp(26.0, 240.0)).into_iter_once()
        };
        for c in centers {
            let p = view.stage(c - vec2(badge_w / 2.0, 7.0));
            inked_rrect(p.x, p.y, badge_w * view.k, 14.0 * view.k, 5.0 * view.k, 1.2 * view.k, shade(color, 0.85));
            art.text(label, view.stage(c), 12.0 * view.k, WHITE, Some((INK, 1.0 * view.k)));
        }
    }
}

trait OnceVec {
    fn into_iter_once(self) -> Vec<Vec2>;
}
impl OnceVec for Vec2 {
    fn into_iter_once(self) -> Vec<Vec2> {
        vec![self]
    }
}

fn draw_shadows(world: &World, view: &View) {
    let shadow = Color::new(0.05, 0.15, 0.05, 0.28);
    let g = view.world(vec2(-215.0, 247.0));
    ellipse(g, view.len(170.0), view.len(14.0), shadow);
    for b in &world.balls {
        let (p, _) = body_pose(world, b.body);
        let height = (244.0 - p.y).max(0.0);
        let k = (1.0 - height / 500.0).clamp(0.2, 1.0);
        let c = view.world(vec2(p.x, 247.0));
        ellipse(c, view.len(BALL_RADIUS_PX * 1.1) * k, view.len(5.0) * k, Color::new(0.05, 0.15, 0.05, 0.3 * k));
    }
}

fn draw_wicket(world: &World, view: &View) {
    let ol = 1.1 * view.k;
    let (c, a) = body_pose(world, world.stumps);
    for (dx, col, top) in [(-9.0, CREAM_SHADE, -88.0), (9.0, CREAM_SHADE, -88.0), (0.0, Color::from_hex(0xf3e2b3), -93.9)] {
        let pts = [(dx - 4.4, top), (dx + 4.4, top), (dx + 4.4, 93.9), (dx - 4.4, 93.9)].map(|(x, y)| at(view, c, a, x, y));
        inked_poly(&pts, ol, col);
    }
    let (bp, _) = body_pose(world, world.bail);
    let l = view.world(bp - vec2(13.0, 0.0));
    let r = view.world(bp + vec2(13.0, 0.0));
    inked_capsule(l, r, view.len(4.0), ol, Color::from_hex(0xd8a75a));
}

fn draw_batsman(art: &Art, world: &World, view: &View) {
    let kit = &art.kit;
    let ol = 1.3 * view.k;
    let jp = |i: usize| {
        let (h, local) = world.joint_points[i];
        let p = world.bodies[h].position() * local * PX_PER_M;
        view.world(vec2(p.x, p.y))
    };
    // A sprite pinned to two body-local points (world px) of a rigid body.
    let pinned = |s: &Sprite, (o, a): (Vec2, f32), la: (f32, f32), lb: (f32, f32)| {
        s.draw(at(view, o, a, la.0, la.1), at(view, o, a, lb.0, lb.1), WHITE);
    };
    let shade = Color::new(0.88, 0.88, 0.9, 1.0);

    // Back leg and back upper arm, in shade behind the body.
    leg(view, jp(3), jp(4), jp(5), ol, 0.86);
    kit.upper_arm.draw(jp(10), jp(11), shade);
    pinned(&kit.torso, part_pose(world, Part::Chest), (0.0, -52.0), (2.0, 50.0));
    leg(view, jp(0), jp(1), jp(2), ol, 1.0);

    // The bat (pinned to its own collider outline), the bottom hand gripping
    // it, then the front arm over everything.
    pinned(&kit.bat, body_pose(world, world.bat), (11.7, 0.0), (11.7, 177.85));
    kit.forearm.draw(jp(11), jp(12), shade);
    kit.upper_arm.draw(jp(7), jp(8), WHITE);
    kit.forearm.draw(jp(8), jp(9), WHITE);

    // Head last, so the front arm tucks under the chin rather than hiding the face.
    pinned(&kit.head, part_pose(world, Part::Head), (2.0, -2.0), (2.0, 48.0));
}


/// Thigh, pad (with ribs, straps and knee roll), then the shoe in front.
fn leg(view: &View, foot: Vec2, knee: Vec2, hip: Vec2, ol: f32, light: f32) {
    let r = |px: f32| view.len(px);
    inked_capsule(hip, knee, r(19.0), ol, shade(CREAM, light));
    let top = knee;
    let bottom = knee.lerp(foot, 0.8);
    inked_capsule(top, bottom, r(19.0), ol, shade(PAD, light));
    let axis = (bottom - top).normalize_or_zero();
    let side = vec2(-axis.y, axis.x);
    for off in [-9.0, 0.0, 9.0] {
        let a = top + axis * r(16.0) + side * r(off);
        let b = bottom + side * r(off);
        draw_line(a.x, a.y, b.x, b.y, r(2.5), shade(PAD_RIB, light));
    }
    for t in [0.45, 0.8] {
        let m = top.lerp(bottom, t);
        let (a, b) = (m - side * r(19.0), m + side * r(19.0));
        draw_line(a.x, a.y, b.x, b.y, r(4.0), shade(STRAP, light));
    }
    inked_circle(knee, r(15.0), ol, shade(PAD, light));
    draw_circle(knee.x, knee.y, r(8.0), shade(PAD_RIB, light));
    // Shoe on the turf, toe towards the bowler.
    let sole = foot + vec2(0.0, r(3.0));
    inked_rrect(sole.x - r(18.0), sole.y - r(17.0), r(50.0), r(18.0), r(8.0), ol, shade(SHOE, light.max(0.95)));
    draw_rectangle(sole.x - r(16.0), sole.y - r(4.0), r(46.0), r(4.0), shade(SOLE, light));
}

fn draw_balls(world: &World, view: &View) {
    let ol = 1.1 * view.k;
    for b in &world.balls {
        let (p, a) = body_pose(world, b.body);
        let c = view.world(p);
        let r = view.len(BALL_RADIUS_PX);
        inked_circle(c, r, ol, Color::from_hex(0x8e1219));
        draw_circle(c.x - r * 0.12, c.y - r * 0.12, r * 0.82, Color::from_hex(0xc81d25));
        // Seam turns with the ball; the shine doesn't.
        let (s, co) = a.sin_cos();
        let d = vec2(co, s) * r * 0.95;
        let n = vec2(-s, co) * r * 0.14;
        for off in [-1.0, 1.0] {
            let (p0, p1) = (c - d + n * off, c + d + n * off);
            draw_line(p0.x, p0.y, p1.x, p1.y, 0.9 * view.k, Color::from_hex(0xf6e7c6));
        }
        draw_circle(c.x - r * 0.38, c.y - r * 0.4, r * 0.22, Color::new(1.0, 1.0, 1.0, 0.6));
    }
}

fn draw_bursts(art: &Art, world: &World, view: &View) {
    for b in &world.bursts {
        let f = b.age / SIM_PER_FRAME; // in the original's 30 fps frames
        let (star_s, star_a) = if f < 4.0 { (1.0 + 0.35 * f / 4.0, 1.0) } else { let t = (f - 4.0) / 16.0; (1.35 - 0.65 * t, 1.0 - 0.96 * t) };
        let label_s = if f < 4.0 { 1.0 + 0.85 * f / 4.0 } else { 1.85 - 0.75 * ((f - 4.0) / 16.0).min(1.0) };
        let label_a = if f > 16.0 { (20.0 - f) / 4.0 } else { 1.0 };

        let p = view.world(b.pos);
        let radius = view.len(35.0) * star_s * 1.6;
        let spikes = 10;
        let star = |rad: f32| -> Vec<Vec2> {
            (0..spikes * 2)
                .map(|i| {
                    let ang = i as f32 / (spikes * 2) as f32 * std::f32::consts::TAU + f * 0.04;
                    let rr = if i % 2 == 0 { rad } else { rad * 0.62 };
                    p + vec2(ang.cos(), ang.sin()) * rr
                })
                .collect()
        };
        let draw_star = |pts: &[Vec2], c: Color| {
            for i in 0..pts.len() {
                draw_triangle(p, pts[i], pts[(i + 1) % pts.len()], c);
            }
        };
        let mut ink = INK;
        ink.a = star_a;
        let mut col = b.color;
        col.a = star_a;
        draw_star(&star(radius + 2.5 * view.k), ink);
        draw_star(&star(radius), col);
        let mut glow = WHITE;
        glow.a = 0.35 * star_a;
        draw_circle(p.x, p.y, radius * 0.45, glow);
        let mut white = WHITE;
        white.a = label_a;
        let mut outline = INK;
        outline.a = label_a;
        art.text(b.label, p, 26.0 * view.k * label_s, white, Some((outline, 2.0 * view.k)));
    }
}

fn draw_hud(art: &Art, world: &World, view: &View, best: u32) {
    let (x, y, w, h) = (160.0, 330.0, 270.0, 36.0);
    let p = view.stage(vec2(x, y));
    inked_rrect(p.x, p.y + 3.0 * view.k, w * view.k, h * view.k, 10.0 * view.k, 0.0, Color::new(0.0, 0.0, 0.0, 0.25));
    inked_rrect(p.x, p.y, w * view.k, h * view.k, 10.0 * view.k, 1.5 * view.k, Color::from_hex(0x18213d));
    let entries = [("RUNS", world.score.to_string()), ("STRIKE RATE", format!("{:.2}", world.strike_rate())), ("BEST", best.to_string())];
    for (i, (label, value)) in entries.iter().enumerate() {
        let cx = x + w * (i as f32 + 0.5) / 3.0;
        art.text(label, view.stage(vec2(cx, y + 10.0)), 9.0 * view.k, GOLD, None);
        art.text(value, view.stage(vec2(cx, y + 24.0)), 17.0 * view.k, WHITE, None);
        if i > 0 {
            stage_rect(view, x + w * i as f32 / 3.0, y + 6.0, 1.0, h - 12.0, Color::from_hex(0x34406a));
        }
    }
}

// ---------- menus ----------

fn dim(view: &View) {
    stage_rect(view, 0.0, 0.0, STAGE_W, STAGE_H, Color::new(0.03, 0.05, 0.12, 0.45));
}

fn card(view: &View, x: f32, y: f32, w: f32, h: f32) {
    let p = view.stage(vec2(x, y));
    let (pw, ph, r) = (w * view.k, h * view.k, 16.0 * view.k);
    rrect(p.x + 5.0 * view.k, p.y + 6.0 * view.k, pw, ph, r, Color::new(0.0, 0.0, 0.0, 0.3));
    inked_rrect(p.x, p.y, pw, ph, r, 2.5 * view.k, KIT_DARK);
    rrect(p.x + 5.0 * view.k, p.y + 5.0 * view.k, pw - 10.0 * view.k, ph * 0.45, r * 0.8, KIT);
}

/// Returns true when clicked.
fn button(art: &Art, view: &View, label: &str, center: Vec2, size: Vec2) -> bool {
    let a = view.stage(center - size / 2.0);
    let wh = size * view.k;
    let m = Vec2::from(mouse_position());
    let hover = m.x >= a.x && m.x <= a.x + wh.x && m.y >= a.y && m.y <= a.y + wh.y;
    let lift = if hover { 2.0 } else { 0.0 } * view.k;
    inked_rrect(a.x, a.y + 4.0 * view.k, wh.x, wh.y, 12.0 * view.k, 2.0 * view.k, Color::from_hex(0xc98a12));
    let face = if hover { Color::from_hex(0xffd65c) } else { GOLD };
    inked_rrect(a.x, a.y - lift, wh.x, wh.y, 12.0 * view.k, 2.0 * view.k, face);
    rrect(a.x + 4.0 * view.k, a.y - lift + 3.0 * view.k, wh.x - 8.0 * view.k, wh.y * 0.38, 9.0 * view.k, Color::new(1.0, 1.0, 1.0, 0.25));
    art.text(label, view.stage(center) - vec2(0.0, lift), 20.0 * view.k, WHITE, Some((INK, 2.0 * view.k)));
    hover && is_mouse_button_pressed(MouseButton::Left)
}

pub fn draw_title(art: &Art, view: &View) -> bool {
    dim(view);
    card(view, 120.0, 48.0, 350.0, 262.0);
    art.text("RAGDOLL", view.stage(vec2(295.0, 92.0)), 46.0 * view.k, GOLD, Some((INK, 3.0 * view.k)));
    art.text("CRICKET", view.stage(vec2(295.0, 136.0)), 46.0 * view.k, WHITE, Some((INK, 3.0 * view.k)));
    let lines = [
        "Move the mouse to swing the bat",
        "Hit the far wall: higher scores more",
        "Avoid the roof, the keeper and your stumps!",
    ];
    for (i, l) in lines.iter().enumerate() {
        art.text(l, view.stage(vec2(295.0, 176.0 + i as f32 * 17.0)), 12.0 * view.k, Color::from_hex(0xffe7ea), None);
    }
    button(art, view, "PLAY", vec2(295.0, 268.0), vec2(130.0, 38.0))
}

pub fn draw_end(art: &Art, world: &World, view: &View) -> bool {
    dim(view);
    card(view, 135.0, 56.0, 320.0, 250.0);
    art.text("HOWZAT!", view.stage(vec2(295.0, 98.0)), 42.0 * view.k, GOLD, Some((INK, 3.0 * view.k)));
    let rows = [
        ("RUNS", world.score.to_string()),
        ("STRIKE RATE", format!("{:.2}", world.strike_rate())),
        ("SCORE", world.total_score().to_string()),
    ];
    for (i, (label, value)) in rows.iter().enumerate() {
        let y = 140.0 + i as f32 * 30.0;
        let p = view.stage(vec2(165.0, y - 12.0));
        rrect(p.x, p.y, 260.0 * view.k, 24.0 * view.k, 8.0 * view.k, Color::new(0.0, 0.0, 0.0, 0.22));
        art.text(label, view.stage(vec2(235.0, y)), 13.0 * view.k, Color::from_hex(0xffe7ea), None);
        art.text(value, view.stage(vec2(370.0, y)), 19.0 * view.k, WHITE, Some((INK, 1.5 * view.k)));
    }
    button(art, view, "TRY AGAIN", vec2(295.0, 266.0), vec2(160.0, 38.0))
}
