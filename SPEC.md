# Ragdoll Cricket — functional spec

Clean-room spec: facts (layout, numbers, rules) observed from the original game,
written in our own words. The implementation is written from this document only;
no original code, art or sound is used.

Coordinates below are the original's world pixels: +x right, +y **down**,
origin roughly at the middle of the play area. Rotations are clockwise-positive
(screen convention). Sizes are half-extents.

## Stage
- 590 × 384 px stage, 30 fps.

## Static scenery
| object | center | half size | notes |
|---|---|---|---|
| ground (static box) | (180.6, 307.1) | (854.9, 63.0) | top surface at y ≈ 244. friction 0.7, restitution 0.2 |

## Scoring walls (static sensors)
The field is a closed box. A hit ball scores by **which wall zone it touches first**.

| zone | center | half size | effect |
|---|---|---|---|
| right wall, low | (780.7, 170.1) | (25.4, 76.0) | 1 run |
| right wall, mid | (780.7, 44.1) | (25.4, 76.0) | 2 runs |
| right wall, high | (780.6, -158.6) | (25.4, 126.7) | 4 runs |
| right wall, top | (780.5, -310.6) | (25.4, 25.4) | 6 runs |
| ceiling, right end | (653.8, -361.3) | (152.0, 25.4) | 6 runs |
| ceiling | (-55.6, -361.4) | (557.5, 25.5) | OUT (caught) |
| left wall, low | (-587.8, 43.4) | (25.4, 202.7) | OUT (caught behind) |
| left wall, high | (-587.9, -184.0) | (25.4, 50.7) | 4 runs |
| left wall, top | (-587.8, -285.2) | (25.4, 50.7) | 6 runs |

## Wicket
| object | shape | center | half size | physics |
|---|---|---|---|---|
| stumps | box | (-411.4, 159.1) | (4.4, 93.9) | dynamic, starts asleep, bullet, density 1, friction 0.5, restitution 0.3 |
| bail | circle r≈4.7 | (-411.5, 67.8) | — | dynamic, fixed rotation, starts asleep, bullet, density 1, friction 0.2, restitution 0.3 |

## Batsman ragdoll
All parts are boxes (head too, with fixed rotation) except the bat, which is a
tapered blade polygon about 23 px wide and 178 px long, with the handle at the top.
Body parts never collide with one another (shared negative group), but they do
collide with the ground and the ball. The bat collides with the ball and the
wicket, but not with the ground.

| part | center | half size | rot | density | ang. damping | gravity |
|---|---|---|---|---|---|---|
| back leg (shin) | (-278.2, 179.9) | (90.2, 26.3) | -45° | 2 | 0.5 | yes |
| back thigh | (-201.0, 84.0) | (69.6, 29.9) | -45° | 1 | 0 | yes |
| front leg (shin) | (-110.3, 167.5) | (30.3, 80.8) | 0° | 2 | 0.5 | yes |
| front thigh | (-138.0, 87.2) | (27.3, 58.2) | -30° | 1 | 0 | yes |
| chest | (-173.2, 7.5) | (41.0, 51.6) | 0° | 1 | 0 | yes |
| head | (-150.2, -41.2) | (33.8, 31.7) | 0° | 1 | 0 | yes, fixed rotation |
| top shoulder (upper arm) | (-126.8, -23.3) | (47.0, 25.5) | 0° | 1.4 | 0 | **no** |
| top forearm | (-68.0, 2.0) | (15.6, 45.3) | -30° | 1 | 0 | **no** |
| bottom shoulder (upper arm) | (-159.8, 18.3) | (43.2, 18.0) | 30° | 1.4 | 0 | **no** |
| bottom forearm | (-91.7, 43.6) | (49.9, 15.7) | 0° | 1 | 0 | **no** |
| bat | (-53.5, 93.8) | (11.7, 88.9) | 0° | 2 | 0 | **no**, bullet, restitution 0.5 |

Friction 0.2 and restitution 0 on all body parts unless noted.

## Joints
All joints are revolute pins at the given world point, with angle limits (radians)
and a rotational spring (stiffness, damping) that pulls back to the rest angle.

| bodies | anchor | limits | spring k / damping |
|---|---|---|---|
| ground ↔ front shin | (-101.5, 239.85) | -0.5 … 0.5 | 150 / 0.8 |
| front shin ↔ front thigh | (-105.9, 123.75) | -0.8 … 0.3 | 80 / 0.5 |
| front thigh ↔ chest | (-164.15, 52.3) | -0.3 … 0.5 | 80 / 0.8 |
| ground ↔ back shin | (-329.2, 241.55) | -0.2 … 0.5 | 150 / 0.8 |
| back shin ↔ back thigh | (-232.1, 130.15) | -1.2 … 0.1 | 80 / 0.8 |
| back thigh ↔ chest | (-192.9, 54.35) | -0.6 … 0.3 | 80 / 0.8 |
| chest ↔ head | (-172.75, -30.25) | locked (0 … 0) | — |
| chest ↔ top shoulder | (-163.85, -27.0) | -0.5 … 1.1 | 50 / 0.5 |
| top shoulder ↔ top forearm | (-86.5, -22.95) | -0.1 … 2.0 | 15 / 0.5 |
| top forearm ↔ bat | (-58.7, 21.9) | -0.5 … 0.5 | 20 / 0.5 |
| chest ↔ bottom shoulder | (-183.5, 4.55) | -3 … 3 | 50 / 0.5 |
| bottom shoulder ↔ bottom forearm | (-131.25, 37.8) | -2.5 … 0.3 | 15 / 0.5 |
| bottom forearm ↔ bat | (-59.55, 47.9) | -1.5 … 1.5 | 20 / 0.5 |

## Rules
- A ball only scores after it has touched the bat.
- After a hit, the first scoring zone it touches adds its runs, with a chime and a
  burst showing the number. The ball is then removed.
- After a hit, touching an OUT zone ends the innings (burst "OUT" + appeal shout).
- At any time, the ball **or the bat** touching the stumps or bail ends the innings
  (stumps sound + appeal); the bail is knocked loose.
- Sounds: "thud" on the first bat contact, "bump" whenever the ball hits the ground.

## Collision filtering (who touches whom)
- Ball: ground, bat, wicket/bail, other balls, sensors. **Passes through the batsman's body.**
- Bat: only the ball (the wicket rejects the bat, so "hit wicket" can't actually happen).
- Body parts: nothing at all, not even the ground. The ragdoll is held up purely by
  the two legs being pinned to the ground.
- Wicket/bail: ground, ball, each other.
- Balls touching each other: both count as "hit".

## World
- 40 px per metre. Gravity +10 m/s² (downwards). One physics step of 0.04 s per
  30 fps frame, so the game runs at 1.2× real time. Every part feels gravity
  (the "no gravity" flags on the arms and bat are never used by the original).
- Camera fixed. Stage pixel = (278, 200) + 0.431 × (world px − (52.7, 92.8)).
  The visible world spans x ≈ −592…776 and y ≈ −371…519.

## Joint springs
- Each frame, each spring joint applies a torque that pushes the joint angle back toward 0
  (the angle it started at). Its magnitude is |k·angle + d·angular speed|, and its
  direction depends only on the sign of the angle.

## Bat control
- The bat is dragged by a soft "mouse joint" toward the cursor in world space.
  The grab point on the bat is world (−102, 121) at start, which is just off the blade.
  The joint has 5 Hz stiffness, 0.7 damping, and a maximum force of 200 × the bat's mass.

## Bowling
- A new ball arrives every 2.0 simulated seconds while the innings is live, and
  any number of balls can be in play at once. Balls you miss stay in the world.
- Spawn position (px): x = 750 + 100·r₁, y = −150 + 200·(r₂ − 0.5).
- Let θ = atan((y + 150) / (x − 750)). The initial velocity (m/s) is
  vx = −10·cos θ − 10 and vy = −5·sin θ + 2.5.
- A spin kick is applied once at spawn: torque = 20 × (velocity angle in degrees) − 10·vx + 100.
- Ball: radius 17.5 px, density 2, friction 0.6, restitution 0.5, angular damping 0.8.
  It has a highlight that stays fixed while the ball rotates.

## HUD and flow
- Title screen with a Play button.
- In play, the HUD shows Runs, Strike rate and Best.
- Strike rate = runs ÷ (number of the ball that last scored), floored to 2 decimals.
- On an out, an end panel appears immediately while the physics keeps running. It shows:
  - Runs, Strike rate, and Score = floor(runs × strike rate).
  - A Try Again button, which rebuilds the whole world.
- Best = the highest Score this session.

## Bursts
- A 20-frame pop at the ball's position, about 70 px wide.
- A starburst swells to about 1.35× by frame 4, then shrinks to about 0.7× while fading out.
- The number ("1", "2", "4", "6" or "OUT") pops to about 1.85× by frame 4, settles back
  to about 1.1×, and fades out in the last few frames.
