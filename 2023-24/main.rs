use std::io;
use std::ops::{Add, Mul, Sub};
use anyhow::{Result, Ok, Context, bail};
use mathru::{
    algebra::linear::{
        matrix::{General, Solve},
        vector::Vector,
    },
    matrix, vector,
};

#[derive(Debug)]
struct Intersection2D {
    position: [f64; 2],
    time: [f64; 2],
}

#[derive(Debug, Copy, Clone)]
struct D3 {
    x: f64,
    y: f64,
    z: f64,
}

impl D3 {
    fn zero() -> D3 {
        D3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    fn sum(&self) -> f64 {
        self.x + self.y + self.z
    }
}

impl Sub for D3 {
    type Output = D3;

    fn sub(self, rhs: Self) -> Self::Output {
        D3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Add for D3 {
    type Output = D3;

    fn add(self, rhs: Self) -> Self::Output {
        D3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Mul for D3 {
    type Output = D3;

    fn mul(self, rhs: Self) -> Self::Output {
        D3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Mul<f64> for D3 {
    type Output = D3;

    fn mul(self, rhs: f64) -> D3 {
        D3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl D3 {
    fn from_array(array: [f64; 3]) -> D3 {
        D3 {
            x: array[0],
            y: array[1],
            z: array[2],
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct State {
    position: D3,
    velocity: D3,
}

impl Add for State {
    type Output = State;

    fn add(self, rhs: Self) -> Self::Output {
        State {
            position: self.position + rhs.position,
            velocity: self.velocity + rhs.velocity,
        }
    }
}

impl Sub for State {
    type Output = State;

    fn sub(self, rhs: Self) -> Self::Output {
        State {
            position: self.position - rhs.position,
            velocity: self.velocity - rhs.velocity,
        }
    }
}

fn main() -> Result<()> {
    let states = parse()?;

    let test_area_bounds = 200000000000000f64..=400000000000000f64;

    let mut collisions = 0;

    for (i, state1) in states.iter().enumerate() {
        for state2 in &states[i + 1..] {
            let intersection = path_intersection_2d(state1, state2);
            let Some(intersection) = intersection else {
                continue;
            };
            if intersection.time.iter().any(|&t| t < 0f64) {
                continue;
            }
            if intersection.position.iter().any(|p| !test_area_bounds.contains(p)) {
                continue;
            }
            collisions += 1;
        }
    }

    println!("{}", collisions);

    for (i, &state1) in states.iter().enumerate() {
        for (j, &state2) in states.iter().enumerate() {
            for (k, &state3) in states.iter().enumerate() {
                if i == j || i == k || j == k {
                    continue;
                }
                let rock = hit_all(vec![state1, state2, state3].as_slice());
                if (rock.position.x.round() - rock.position.x).abs() < f64::EPSILON &&
                    (rock.position.y.round() - rock.position.y).abs() < f64::EPSILON &&
                    (rock.position.z.round() - rock.position.z).abs() < f64::EPSILON &&
                    (rock.velocity.x.round() - rock.velocity.x).abs() < f64::EPSILON &&
                    (rock.velocity.y.round() - rock.velocity.y).abs() < f64::EPSILON &&
                    (rock.velocity.z.round() - rock.velocity.z).abs() < f64::EPSILON {
                    println!("{} {} {} {:?}", i, j, k, rock);
                    println!("{:?}", rock.position.sum());
                    return Ok(())
                }
            }
        }
    }

    Ok(())
}

#[derive(Copy, Clone)]
struct Plane {
    normal: D3,
    offset: f64,
}

fn points_to_plane(p0: D3, p1: D3, p2: D3) -> Plane {
    let a = p1 - p0;
    let b = p2 - p0;
    let normal = cross_product(a, b);
    Plane {
        normal,
        offset: normal.x * p0.x + normal.y * p0.y + normal.z * p0.z,
    }
}

fn cross_product(v0: D3, v1: D3) -> D3 {
    D3 {
        x: (v0.y * v1.z) - (v0.z * v1.y),
        y: (v0.z * v1.x) - (v0.x * v1.z),
        z: (v0.x * v1.y) - (v0.y * v1.x),
    }
}

fn hit_all(states: &[State]) -> State {
    let state0 = states[0];
    let perspective_state1 = states[1] - state0;
    let perspective_state2 = states[2] - state0;
    let perspective_plane1 = points_to_plane(D3::zero(), perspective_state1.position, perspective_state1.position + perspective_state1.velocity);
    let perspective_plane2 = points_to_plane(D3::zero(), perspective_state2.position, perspective_state2.position + perspective_state2.velocity);

    // a*x + b*y + c*z = offset
    // a(px+vx*t) + b(py+vy*t) + c(pz+vz*t = offset
    // a*px + a*vx*t + b*py + b*vy*t + c*pz + c*vz*t = offset
    // a*vx*t + b*vy*t + c*vz*t = offset - (a*px+ b*py + c*pz)
    // t = (offset - (a*px+ b*py + c*pz)) / (a*vx + b*vy + c*vz)
    fn collision_time(state: State, plane: Plane) -> f64 {
        (plane.offset - (plane.normal * state.position).sum()) /
            (plane.normal * state.velocity).sum()
    }
    let collision_time1 = collision_time(perspective_state1, perspective_plane2);
    let collision_time2 = collision_time(perspective_state2, perspective_plane1);
    let time_diff = collision_time2 - collision_time1;

    let perspective_collision_position1 = perspective_state1.position + perspective_state1.velocity * collision_time1;
    let perspective_collision_position2 = perspective_state2.position + perspective_state2.velocity * collision_time2;

    let perspective_rock_velocity = (perspective_collision_position2 - perspective_collision_position1) * (1f64 / time_diff);
    let perspective_rock_position = perspective_collision_position1 - perspective_rock_velocity * collision_time1;

    State {
        position: perspective_rock_position,
        velocity: perspective_rock_velocity,
    } + state0
}

fn path_intersection_2d(state1: &State, state2: &State) -> Option<Intersection2D> {
    // px1 + vx1*t1 = px2 + vx2*t2
    // py1 + vy1*t1 = py2 + vy2*t2
    //
    // vx1*t1 - vx2*t2 = px2 - px1
    // vy1*t1 - vy2*t2 = py2 - py1

    let a: General<f64> = matrix![
        state1.velocity.x, -state2.velocity.x;
        state1.velocity.y, -state2.velocity.y];
    let b: Vector<f64> = vector![
        state2.position.x - state1.position.x;
        state2.position.y - state1.position.y];

    let x = match a.solve(&b) {
        Result::Ok(x) => x,
        Err(_) => return None
    };
    let (t1, t2) = (x[0], x[1]);

    Some(Intersection2D {
        position: [state1.position.x + state1.velocity.x * t1, state1.position.y + state1.velocity.y * t1],
        time: [t1, t2],
    })
}

fn parse() -> Result<Vec<State>> {
    let stdin = io::stdin();
    let mut states: Vec<State> = Vec::new();
    for line in stdin.lines() {
        let line = line?;
        let (positions, velocities) = line.split_once('@').context("missing @ in line")?;
        let [position, velocity] = [positions, velocities]
            .map(|s| {
                let v: Vec<f64> = s
                    .split(',')
                    .map(|n| Ok(n.trim().parse::<f64>()?))
                    .collect::<Result<_>>()?;
                let &[x, y, z, ..] = v.as_slice() else { bail!("expected 3 coordinates") };
                Ok([x, y, z])
            });
        states.push(State {
            position: D3::from_array(position?),
            velocity: D3::from_array(velocity?),
        })
    }
    Ok(states)
}