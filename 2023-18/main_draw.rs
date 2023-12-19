use std::cell::RefCell;
use std::cmp::{max, min};
use std::io;
use std::ops::{Mul};
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;
use anyhow::{Result, Ok, bail, Context};
use genawaiter::{rc::gen, yield_, GeneratorState, Generator};
use once_cell::sync::Lazy;
use regex::Regex;
use nannou::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug)]
struct Offset {
    x: i64,
    y: i64,
}

impl Mul<i64> for Offset {
    type Output = Offset;

    fn mul(self, rhs: i64) -> Self::Output {
        Offset {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

static UP: Offset = Offset { x: 0, y: -1 };
static LEFT: Offset = Offset { x: -1, y: 0 };
static DOWN: Offset = Offset { x: 0, y: 1 };
static RIGHT: Offset = Offset { x: 1, y: 0 };

#[derive(Copy, Clone, PartialEq, Debug, Hash, Eq, Default)]
struct Coord {
    x: i64,
    y: i64,
}

impl Coord {
    fn offset(self, offset: Offset) -> Coord {
        Coord {
            x: self.x + offset.x,
            y: self.y + offset.y,
        }
    }
}

#[derive(Copy, Clone)]
struct BoundingBox {
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
}

impl BoundingBox {
    fn on_coord(coord: &Coord) -> BoundingBox {
        BoundingBox {
            min_x: coord.x,
            min_y: coord.y,
            max_x: coord.x,
            max_y: coord.y,
        }
    }
    fn include(&self, coord: &Coord) -> BoundingBox {
        BoundingBox {
            min_x: min(self.min_x, coord.x),
            min_y: min(self.min_y, coord.y),
            max_x: max(self.max_x, coord.x),
            max_y: max(self.max_y, coord.y),
        }
    }

    fn is_border(&self, coord: &Coord) -> bool {
        coord.x == self.min_x || coord.y == self.min_y || coord.x == self.max_x || coord.y == self.max_y
    }

    fn area(&self) -> i64 {
        (self.max_x - self.min_x) * (self.max_y - self.min_y)
    }
}

struct Model {
    draw_state: Rc<RefCell<DrawState>>,
    since_last_progress: Duration,
    progress: Pin<Box<dyn Generator<Return=i64, Yield=()> + 'static>>,
    done: bool,
}

struct DrawState {
    bounds: BoundingBox,
    borders: Vec<Line>,
    areas: Vec<Area>,
    points: Vec<Coord>,
    recursive_current_bounds: Vec<BoundingBox>,
}

impl DrawState {
    fn t_x(&self, rect: Rect, x: i64) -> f32 {
        let int_width = self.bounds.max_x - self.bounds.min_x;
        let fraction = (x - self.bounds.min_x) as f32 / int_width as f32;
        rect.left() + fraction * rect.w()
    }

    fn t_y(&self, rect: Rect, y: i64) -> f32 {
        let int_height = self.bounds.max_y - self.bounds.min_y;
        let fraction = (y - self.bounds.min_y) as f32 / int_height as f32;
        rect.bottom() + (1.0 - fraction) * rect.h()
    }

    fn t_coord(&self, rect: Rect, coord: Coord) -> Point2 {
        pt2(self.t_x(rect, coord.x), self.t_y(rect, coord.y))
    }
}

struct Line {
    from: Coord,
    to: Coord,
}

struct Area {
    bounds: BoundingBox,
    area_type: AreaType,
    level: i64,
}

enum AreaType {
    Positive,
    Negative,
}

fn main() -> Result<()> {
    Lazy::force(&RE);

    nannou::app(model)
        .event(event)
        .simple_window(view)
        .run();

    Ok(())
}

fn model(_app: &App) -> Model {
    let instructions = parse().unwrap();

    let points = instructions_to_points(&instructions);

    let mut bounds: Option<BoundingBox> = None;
    for point in &points {
        bounds = Some(match bounds {
            None => BoundingBox::on_coord(point),
            Some(bounds) => bounds.include(point),
        });
    }
    let bounds = bounds.unwrap();

    let mut draw_state = DrawState {
        bounds,
        borders: Vec::new(),
        areas: vec![],
        points: vec![],
        recursive_current_bounds: vec![],
    };

    for points in points.windows(2).chain([[*points.last().unwrap(), *points.first().unwrap()].as_slice()]) {
        draw_state.borders.push(Line { from: points[0], to: points[1] });
    }

    let end = points.len() - 1;

    draw_state.points = points;

    let draw_state: Rc<RefCell<DrawState>> = Rc::from(RefCell::from(draw_state));

    let progress = area_rec(draw_state.clone(), 0, end, 1, 0);

    Model {
        draw_state,
        since_last_progress: Duration::default(),
        progress,
        done: false,
    }
}

fn event(_app: &App, model: &mut Model, event: Event) {
    match event {
        Event::Update(update) => {
            if model.done {
                return;
            }
            model.since_last_progress += update.since_last;
            if model.since_last_progress > Duration::from_millis(100) {
                model.since_last_progress -= Duration::from_millis(100);
                let pin = &mut model.progress;
                match pin.as_mut().resume() {
                    GeneratorState::Yielded(()) => {}
                    GeneratorState::Complete(_) => {
                        model.done = true;
                    }
                };
            }
        }
        _ => {}
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let rect = app.window_rect();

    draw.background().color(WHITE);

    let draw_state = model.draw_state.borrow();

    for area in &draw_state.areas {
        let color = match area.area_type {
            AreaType::Positive => { LIGHTBLUE }
            AreaType::Negative => { WHITE }
        };
        let x_center = (draw_state.t_x(rect, area.bounds.min_x) + draw_state.t_x(rect, area.bounds.max_x)) / 2.0;
        let y_center = (draw_state.t_y(rect, area.bounds.min_y) + draw_state.t_y(rect, area.bounds.max_y)) / 2.0;
        let w = draw_state.t_x(rect, area.bounds.max_x) - draw_state.t_x(rect, area.bounds.min_x);
        let h = draw_state.t_y(rect, area.bounds.min_y) - draw_state.t_y(rect, area.bounds.max_y);
        draw.rect()
            .color(color)
            .x(x_center)
            .y(y_center)
            .w(w)
            .h(h);
    }

    for inner_bound in &draw_state.recursive_current_bounds {
        let x_center = (draw_state.t_x(rect, inner_bound.min_x) + draw_state.t_x(rect, inner_bound.max_x)) / 2.0;
        let y_center = (draw_state.t_y(rect, inner_bound.min_y) + draw_state.t_y(rect, inner_bound.max_y)) / 2.0;
        let w = draw_state.t_x(rect, inner_bound.max_x) - draw_state.t_x(rect, inner_bound.min_x);
        let h = draw_state.t_y(rect, inner_bound.min_y) - draw_state.t_y(rect, inner_bound.max_y);
        draw.rect()
            .no_fill()
            .stroke_weight(2.0)
            .stroke_color(GREEN)
            .x(x_center)
            .y(y_center)
            .w(w)
            .h(h);
    }

    for line in &draw_state.borders {
        draw.line()
            .color(BLACK)
            .start(draw_state.t_coord(rect, line.from))
            .end(draw_state.t_coord(rect, line.to))
            .weight(1.0);
    };

    draw.to_frame(app, &frame).unwrap();
}

fn instructions_to_points(instructions: &Vec<Input>) -> Vec<Coord> {
    let mut result = Vec::<Coord>::new();
    let mut current = Coord::default();
    for instruction in instructions {
        result.push(current);
        current = current.offset(instruction.direction * instruction.count);
    }
    result
}

fn area_rec<'a>(draw_state: Rc<RefCell<DrawState>>, start: usize, end: usize, multiplier: i64, level: i64) -> Pin<Box<dyn Generator<Return=i64, Yield=()>>> {
    let g = gen!({
        let mut bounds: Option<BoundingBox> = None;
        let mut i = start;
        loop {
            let point = &draw_state.borrow().points[i];
            bounds = Some(match bounds {
                None => BoundingBox::on_coord(point),
                Some(bounds) => bounds.include(point),
            });
            if i == end {
                break;
            }
            i = (i + 1) % draw_state.borrow().points.len();
        }
        let bounds = bounds.unwrap();
        let own_area = multiplier * bounds.area();
        let mut sub_areas_sum = 0;
        draw_state.borrow_mut().recursive_current_bounds.push(bounds);
        let pos = match draw_state.borrow().areas.binary_search_by_key(&(level + 1), |x| x.level) {
            Result::Ok(i) => i,
            Err(i) => i
        };
        draw_state.borrow_mut().areas.insert(pos, Area {
            bounds,
            area_type: if multiplier > 0 { AreaType::Positive } else { AreaType::Negative },
            level,
        });
        yield_!(());

        let mut prev_on_border: Option<usize> = None;
        // initialize with 1 before start, so that `prev_on_border` is properly initialized for points[start]
        // (that is, prev_on_border.is_some() if points[start] is the first point to go off border)
        let mut i = (start + draw_state.borrow().points.len() - 1) % draw_state.borrow().points.len();
        let mut initializing = true;
        loop {
            let point = draw_state.borrow().points[i];
            if bounds.is_border(&point) {
                prev_on_border = Some(i);
            } else {
                if let Some(prev_i) = prev_on_border {
                    // find where this line attaches back to border
                    let mut next_i = i;
                    loop {
                        next_i = (next_i + 1) % draw_state.borrow().points.len();
                        let next_point = draw_state.borrow().points[next_i];
                        if bounds.is_border(&next_point) {
                            let mut inner_gen = area_rec(draw_state.clone(), prev_i, next_i, -multiplier, level + 1);
                            let sub_area: i64;
                            loop {
                                match inner_gen.as_mut().resume() {
                                    GeneratorState::Yielded(()) => {
                                        yield_!(())
                                    }
                                    GeneratorState::Complete(result) => {
                                        sub_area = result;
                                        break;
                                    }
                                }
                            }
                            sub_areas_sum += sub_area;
                            break;
                        }
                    }
                }
                prev_on_border = None;
            }
            if i == end && !initializing {
                break;
            }
            i = (i + 1) % draw_state.borrow().points.len();
            initializing = false;
        }
        draw_state.borrow_mut().recursive_current_bounds.pop();

        own_area + sub_areas_sum
    });

    Box::pin(g)
}

struct Input {
    direction: Offset,
    count: i64,
}

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\w) (\w+) \(#(\w{6})\)").unwrap());

fn parse() -> Result<Vec<Input>> {
    let stdin = io::stdin();
    let mut result = Vec::<Input>::new();
    for line in stdin.lines() {
        let line = line?;
        let captures = RE.captures(line.as_str()).context("invalid line")?;
        let (_, groups) = captures.extract::<3>();
        let (hex_len, dir_digit) = groups[2].split_at(5);
        result.push(Input {
            // part 1
            // direction: match groups[0] {
            //     "R" => RIGHT,
            //     "D" => DOWN,
            //     "L" => LEFT,
            //     "U" => UP,
            //     _ => bail!("invalid direction")
            // },
            // count: groups[1].parse()?,

            // part 2
            direction: match dir_digit {
                "0" => RIGHT,
                "1" => DOWN,
                "2" => LEFT,
                "3" => UP,
                _ => bail!("invalid direction")
            },
            count: i64::from_str_radix(hex_len, 16)?,
        })
    }


    Ok(result)
}
