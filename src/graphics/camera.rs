use std::default::Default;

use ndarray::linalg::*;

use ndarray::{
    Array,
    Array1,
    Array2,
    ArrayBase,
};


use super::*;


#[derive(Debug)]
pub struct Camera {
    coord_sys: SharedCoordSys,

    pub eye: Relative<Point3>,
    screen:  Relative<Screen>,
}

#[derive(Debug)]
pub struct Screen {
    cell_size: (f64, f64),
    screen_size: (usize, usize),

    corners: (Point3, Point3, Point3, Point3),
    centers: Array2<Point3>,
}
impl Camera {
    pub fn new(
        cell_size: (f64, f64),
        screen_size: (usize, usize),
        eye_pos: Point3

        ) -> Self {

        let coord_sys = SharedCoordSys::new();
        Self {
            eye:    Relative::new(&coord_sys, eye_pos),
            screen: Relative::new(&coord_sys,
                                  Screen::new(cell_size, screen_size)),
            coord_sys,
        }
    }

    pub fn apply_rel(&mut self, t: Transform) -> &mut Self {
        self.coord_sys.apply_rel(t);
        self
    }

    pub fn get_pivot(&self) -> Point3 {
        let (x, y) = self.get_screen_size();
        (self.eye.get_abs() + self.get_center((x/2, y/2)).unwrap()) / 2.
    }

    pub fn get_screen_size(&self) -> &(usize, usize) {
        &self.screen.as_rel().screen_size
    }

    pub fn get_screen_centers(&self) -> Array2<Point3> {
        self.screen
             .abs_map_ref(
                 |s, cs| s.centers
                          .map(|c| cs.apply_to(c.clone()))
             )
    }

    pub fn get_center(&self, coords: (usize, usize)) -> Option<Point3> {
        self.screen
            .abs_map_ref(
                |s, cs| s.get_center(coords)
                         .map(|c| cs.apply_to(c.clone()))
            )
    }

    pub fn get_ray(&self, coords: (usize, usize)) -> Option<Ray3> {
        self.get_center(coords).map(|origin|
            Ray3::new(
                origin.clone(),
                Vec3::from(origin - self.eye.get_abs()),
            )
        )
    }
}

impl Default for Camera {
    fn default() -> Camera {
        let screen = Screen::default();
        let total_dims = (
            screen.screen_size.0 as f64 * screen.cell_size.0,
            screen.screen_size.1 as f64 * screen.cell_size.1,
        );
        let coord_sys = SharedCoordSys::new();
        Camera {
            eye: Relative::new(
                &coord_sys,
                Point3::new(total_dims.0/2., total_dims.1/2., 45.),
            ),
            screen: Relative::new(&coord_sys, screen),
            coord_sys,
        }
    }
}

impl Screen {
    pub fn new(cell_size: (f64, f64), screen_size: (usize, usize)) -> Self {
        let total_dims = (
            screen_size.0 as f64 * cell_size.0,
            screen_size.1 as f64 * cell_size.1,
        );
        Screen {
            cell_size,
            screen_size,

            corners: (
                Point3::new(0.,           0., 0.),
                Point3::new(total_dims.0, 0., 0.),
                Point3::new(total_dims.0, total_dims.1, 0.),
                Point3::new(0.,           total_dims.1, 0.),
            ),

            centers: {
                let cz = cell_size;
                let sz = screen_size;
                ArrayBase::from_shape_fn((sz.1, sz.0), |(y, x)| {
                    Point3::new(x as f64 * cz.0 + cell_size.0/2.,
                                y as f64 * cz.1 + cell_size.1/2.,
                                0.)
                })
            },
        }
    }

    pub fn get_center(&self, (x, y): (usize, usize)) -> Option<&Point3> {
        self.centers.get((y, x))
    }
}

impl Default for Screen {
    fn default() -> Screen { Screen::new((0.25, 0.25), (64, 64)) }
}

impl Transformable for Camera {
    fn apply(&mut self, t: &Transform) -> &mut Self {
        // self.screen.apply(t);
        self.coord_sys.apply(t);
        self
    }
}

impl Transformable for Screen {
    fn apply(&mut self, t: &Transform) -> &mut Self {
        self.corners.0.apply(t);
        self.corners.1.apply(t);
        self.corners.2.apply(t);
        self.corners.3.apply(t);
        self
    }
}
