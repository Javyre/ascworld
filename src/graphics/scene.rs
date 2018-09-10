use std::default::Default;
use std::cmp::Ordering;

use super::*;
use super::camera::Camera;

use termion::color::*;
use termion::cursor;

use std::f64;

pub struct Object {
    pub bounds: [Point3; 2],
    pub polygons: Vec<Polygon>,
    pub color: Rgb,
}

pub struct Scene {
    pub objects: Vec<Object>,
    pub camera: Camera,
}

#[derive(Clone)]
pub enum Cell {
    Empty,
    Hit{ color: Rgb, dist: f64 },
}

pub struct RenderedScene(Vec<Vec<Cell>>);

impl Object {
    pub fn new(color: Rgb, polygons: Vec<Polygon>) -> Self {
        let mut r = Self {
            bounds: [Point3::new(0.,0.,0.), Point3::new(0.,0.,0.)],
            polygons,
            color,
        };
        r.recalc_bounds();
        r
    }

    pub fn recalc_bounds(&mut self) {
        self.bounds =
            if self.polygons.len() > 0 {
                let mut b = self.polygons.iter().map(|py| py.bounds());
                let acc = b.next().unwrap();
                b.fold(acc, |mut a, x| unsafe { [
                    a.get_unchecked_mut(0).lower_bound(x.get_unchecked(0).clone()),
                    a.get_unchecked_mut(1).upper_bound(x.get_unchecked(1).clone()),
                ]})
            } else {
                [Point3::new(0.,0.,0.), Point3::new(0.,0.,0.)]
            }
    }
}

impl Transformable for Object {
    fn apply(&mut self, t: &Transform) -> &mut Self {
        self.polygons.iter_mut().for_each(|p| { p.apply(t); });
        // self.bounds[0].apply(t);
        // self.bounds[1].apply(t);
        // TODO: This should instead apply transform to bounds
        // NOTE: actually, im not sure this
        // would have the same effect
        self.recalc_bounds();
        self
    }
}

#[inline]
fn fake_cmp<T: PartialOrd>(a: &T, b: &T) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

impl Intersectable for Object {
    fn intersect(&self, ray: &Ray3) -> Option<(Point3, f64)> {
        // TODO: should do bounds check first here...

        // perform bounds check before
        // actual polygon intersections
        if ray.collides_box(&self.bounds) {
            self.polygons.iter().filter_map(|poly| poly.intersect(&ray))
                .min_by(|a, b| fake_cmp(&a.1, &b.1))
        } else {
            None
        }
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            camera: Camera::default(),
        }
    }

    pub fn test_ray(&self, cell: (usize, usize)) -> Cell {
        self.camera.get_ray(cell)
            .and_then(|ray| {
                self.objects.iter()
                    .filter_map(|obj|
                        obj.intersect(&ray).map(|ip| (obj.color, ip))
                    )
                    .min_by(|a, b| fake_cmp(&(a.1).1, &(b.1).1))
                    .map(|(c, (_, d))| Cell::Hit { color: c, dist: d })

            }).unwrap_or(Cell::Empty)
    }

    pub fn render(&self, out: &mut RenderedScene) {
        self.camera.get_screen_centers()
            .indexed_iter()
            .for_each(|((y, x), _)|{
                out.0[y][x] = self.test_ray((x, y));
            });
    }

    pub fn empty_render(&self) -> RenderedScene {
        let (x, y) = self.camera.get_screen_size().clone();
        RenderedScene(vec![vec![Cell::Empty; x]; y])
    }
}

fn mul_rgb(Rgb(r, g, b): Rgb, n: f64) -> Rgb {
    Rgb(
        (r as f64*n) as u8,
        (g as f64*n) as u8,
        (b as f64*n) as u8,
    )
}

impl RenderedScene {
    pub fn display(&self, o: &mut impl ::std::io::Write) {
        write!(o, "{}", cursor::Hide).unwrap();
        self.0.chunks(2).for_each(|rows| {
            match rows {
                &[ref rowa, ref rowb] => {
                    rowa.iter().zip(rowb.iter()).for_each(|(cella, cellb)| {
                        let mut fg = None;
                        let mut bg = None;
                        match cella {
                            Cell::Empty => {},
                            Cell::Hit{color: c, dist: d} =>
                                bg = Some(mul_rgb(*c, (50.-*d)/50.)),

                        }
                        match cellb {
                            Cell::Empty => {},
                            Cell::Hit{color: c, dist: d} =>
                                fg = Some(mul_rgb(*c, (50.-*d)/50.)),
                        }
                        match (fg, bg) {
                            (Some(fg), Some(bg)) => write!(o, "{}{}▄", Fg(fg), Bg(bg)),
                            (Some(fg), None)     => write!(o, "{}{}▄", Fg(fg), Bg(Reset)),
                            (None,     Some(bg)) => write!(o, "{}{}▀", Fg(bg), Bg(Reset)),
                            (None,     None)     => write!(o, "{} ", Bg(Reset)),
                        }.unwrap()
                    });
                },
                _ => {},
            }

            write!(o, "\n\r").unwrap();
        });
        write!(o, "{}", cursor::Show).unwrap();
    }
}
