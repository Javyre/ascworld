
pub mod camera;
pub mod scene;

use ndarray::linalg::*;

use ndarray::{
    Array,
    Array1,
    Array2,
    ArrayBase,
};

use std::cell::RefCell;
use std::rc::Rc;


#[derive(Debug, Clone)]
pub struct Point3(Array1<f64>);

#[derive(Debug, Clone)]
pub struct Vec3(Array1<f64>);

#[derive(Debug, Clone)]
pub struct Transform(pub Array2<f64>);

#[derive(Debug, Clone)]
pub struct Ray3 {
    origin: Point3,
    vec: Vec3,
    inv_vec: (f64, f64, f64),
    inv_vec_sign: (u8, u8, u8),
}

pub enum Polygon {
    Triangle(Point3, Point3, Point3),
}

pub trait Transformable {
    fn apply(&mut self, t: &Transform) -> &mut Self;
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray3) -> Option<(Point3, f64)>;
}

#[derive(Debug)]
pub struct CoordSys(Transform);

#[derive(Debug)]
pub struct SharedCoordSys(Rc<RefCell<CoordSys>>);

#[derive(Debug)]
pub struct SelfRelative<T: Transformable>(T, CoordSys);

#[derive(Debug)]
pub struct Relative<T: Transformable>(T, SharedCoordSys);

impl Ray3 {
    pub fn new(origin: Point3, vec: Vec3) -> Self {
        let iv = vec.unpack_map(|x| 1./x);
        Self {
            inv_vec_sign: (
                (iv.0 < 0.) as u8,
                (iv.1 < 0.) as u8,
                (iv.2 < 0.) as u8,
            ),
            inv_vec: iv,
            origin,
            vec,
        }
    }

    fn map<F, R>(&self, f: F) -> (R, R, R)
        where F: Fn(u8, f64, f64, f64, u8) -> R
    {
        (
            f(
                0,
                self.origin.unpack().0,
                self.vec.unpack().0,
                self.inv_vec.0,
                self.inv_vec_sign.0,
            ),
            f(
                1,
                self.origin.unpack().1,
                self.vec.unpack().1,
                self.inv_vec.1,
                self.inv_vec_sign.1,
            ),
            f(
                2,
                self.origin.unpack().2,
                self.vec.unpack().2,
                self.inv_vec.2,
                self.inv_vec_sign.2,
            ),
        )
    }

    /// Test for collision with an axis-aligned box
    // TODO: test p.unpack().0 vs p.uget(0) for performance
    pub fn collides_box(&self, bounds: &[Point3; 2]) -> bool {
        let tmin = self.map(|i, o, v, iv, ivs|{
            ( unsafe { bounds[ivs as usize].uget(i) } - o) * iv
        });
        let tmax = self.map(|i, o, v, iv, ivs|{
            ( unsafe { bounds[(1-ivs) as usize].uget(i) } - o) * iv
        });

        if tmin.0 > tmax.1 || tmin.1 > tmax.0 {
            return false;
        }

        if tmin.0 > tmax.2 || tmin.2 > tmax.0 {
            return false;
        }

        let tmin = tmin.0.max(tmin.1).max(tmin.2);
        let tmax = tmax.0.min(tmax.1).min(tmax.2);

        //const t0: f64 = -1.;
        //const t1: f64 =  1.;

        return tmax >= tmin;
        //return  tmax > t0 && t1 > tmin;

    }
}

impl Point3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Point3(array![x, y, z, 1.])
    }

    pub unsafe fn uget(&self, i: u8) -> f64 {
        unsafe { *self.0.uget(i as usize) }
    }

    pub unsafe fn unpack_ptr_mut(&mut self) ->
        (*mut f64, *mut f64, *mut f64) {
        (
            self.0.uget_mut(0),
            self.0.uget_mut(1),
            self.0.uget_mut(2),
        )
    }

    pub fn unpack(&self) -> (f64, f64, f64) {
        unsafe {
            (
                *self.0.uget(0),
                *self.0.uget(1),
                *self.0.uget(2),
            )
        }
    }

    pub fn unpack_map<F>(&self, f: F)
        -> (f64, f64, f64) where F: Fn(f64) -> f64 {
        unsafe {
            (
                f(*self.0.uget(0)),
                f(*self.0.uget(1)),
                f(*self.0.uget(2)),
            )
        }
    }

    pub fn cross_product(&self, mut b: Self) -> Self {
        // let mut r = self.clone();

        // assert!(a.len() == 3 || a.len() == 4);

        unsafe {
            let ax = *self.0.uget(0);
            let ay = *self.0.uget(1);
            let az = *self.0.uget(2);

            let bx = *b.0.uget(0);
            let by = *b.0.uget(1);
            let bz = *b.0.uget(2);

            *b.0.uget_mut(0) = ay*bz - az*by;
            *b.0.uget_mut(1) = az*bx - ax*bz;
            *b.0.uget_mut(2) = ax*by - ay*bx;
        }

        b
    }

    #[inline]
    pub fn dot(&self, b: &Self) -> f64 {
        self.0.dot(&b.0)
    }

    #[inline]
    pub fn lower_bound(&self, mut b: Self) -> Self {
        unsafe {
            let a = self.unpack();
            let b = b.unpack_ptr_mut();

            *b.0 = a.0.min(*b.0);
            *b.1 = a.1.min(*b.1);
            *b.2 = a.2.min(*b.2);
        }
        b
    }

    #[inline]
    pub fn upper_bound(&self, mut b: Self) -> Self {
        unsafe {
            let a = self.unpack();
            let b = b.unpack_ptr_mut();

            *b.0 = a.0.max(*b.0);
            *b.1 = a.1.max(*b.1);
            *b.2 = a.2.max(*b.2);
        }
        b
    }
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3(array![x, y, z, 0.])
    }

    pub unsafe fn uget(&self, i: u8) -> f64 {
        unsafe { *self.0.uget(i as usize) }
    }

    pub unsafe fn unpack_ptr_mut(&mut self) ->
        (*mut f64, *mut f64, *mut f64) {
        (
            self.0.uget_mut(0),
            self.0.uget_mut(1),
            self.0.uget_mut(2),
        )
    }

    pub fn unpack(&self) -> (f64, f64, f64) {
        unsafe {
            (
                *self.0.uget(0),
                *self.0.uget(1),
                *self.0.uget(2),
            )
        }
    }

    pub fn unpack_map<F>(&self, f: F)
        -> (f64, f64, f64) where F: Fn(f64) -> f64 {
        unsafe {
            (
                f(*self.0.uget(0)),
                f(*self.0.uget(1)),
                f(*self.0.uget(2)),
            )
        }
    }

    pub fn cross_product(&self, mut b: Self) -> Self {
        // let mut r = self.clone();

        // assert!(a.len() == 3 || a.len() == 4);

        unsafe {
            let ax = *self.0.uget(0);
            let ay = *self.0.uget(1);
            let az = *self.0.uget(2);

            let bx = *b.0.uget(0);
            let by = *b.0.uget(1);
            let bz = *b.0.uget(2);

            *b.0.uget_mut(0) = ay*bz - az*by;
            *b.0.uget_mut(1) = az*bx - ax*bz;
            *b.0.uget_mut(2) = ax*by - ay*bx;
        }

        b
    }

    pub fn dot(&self, b: &Self) -> f64 {
        self.0.dot(&b.0)
    }

    pub fn norm(&self) -> f64 {
        unsafe {
            f64::sqrt(self.0.uget(0).powf(2.) + self.0.uget(1).powf(2.) + self.0.uget(2).powf(2.))
        }
    }
}

impl From<Point3> for Vec3 {
    fn from(a: Point3) -> Vec3 {
        let mut r = Vec3(a.0);
        unsafe {
            *r.0.uget_mut(3) = 0.;
        }
        r
    }
}

impl From<Vec3> for Point3 {
    fn from(a: Vec3) -> Point3 {
        let mut r = Point3(a.0);
        unsafe {
            *r.0.uget_mut(3) = 1.;
        }
        r
    }
}

impl From<Array1<f64>> for Vec3 {
    fn from(a: Array1<f64>) -> Vec3 {
        let mut r = Vec3(a);
        unsafe {
            *r.0.uget_mut(3) = 0.;
        }
        r
    }
}

macro_rules! impl_op {
    ($tr:ident, $f:ident, $ty:ty) => {

impl ::std::ops::$tr<$ty> for $ty {
    type Output = $ty;
    fn $f(mut self, rhs: $ty) -> $ty {
        unsafe {
            *self.0.uget_mut(0) = ::std::ops::$tr::$f(*self.0.uget(0), *rhs.0.uget(0));
            *self.0.uget_mut(1) = ::std::ops::$tr::$f(*self.0.uget(1), *rhs.0.uget(1));
            *self.0.uget_mut(2) = ::std::ops::$tr::$f(*self.0.uget(2), *rhs.0.uget(2));
        }
        self
    }
}

impl<'a> ::std::ops::$tr<&'a $ty> for $ty {
    type Output = $ty;
    fn $f(mut self, rhs: &'a $ty) -> $ty {
        unsafe {
            *self.0.uget_mut(0) = ::std::ops::$tr::$f(*self.0.uget(0), *rhs.0.uget(0));
            *self.0.uget_mut(1) = ::std::ops::$tr::$f(*self.0.uget(1), *rhs.0.uget(1));
            *self.0.uget_mut(2) = ::std::ops::$tr::$f(*self.0.uget(2), *rhs.0.uget(2));
        }
        self
    }
}

impl<'a> ::std::ops::$tr<$ty> for &'a $ty {
    type Output = $ty;
    fn $f(self, rhs: $ty) -> $ty {
        let mut r = self.clone();
        unsafe {
            *r.0.uget_mut(0) = ::std::ops::$tr::$f(*r.0.uget(0), *rhs.0.uget(0));
            *r.0.uget_mut(1) = ::std::ops::$tr::$f(*r.0.uget(1), *rhs.0.uget(1));
            *r.0.uget_mut(2) = ::std::ops::$tr::$f(*r.0.uget(2), *rhs.0.uget(2));
        }
        r
    }
}

impl ::std::ops::$tr<f64> for $ty {
    type Output = $ty;
    fn $f(mut self, rhs: f64) -> $ty {
        unsafe {
            *self.0.uget_mut(0) = ::std::ops::$tr::$f(*self.0.uget(0), rhs);
            *self.0.uget_mut(1) = ::std::ops::$tr::$f(*self.0.uget(1), rhs);
            *self.0.uget_mut(2) = ::std::ops::$tr::$f(*self.0.uget(2), rhs);
        }
        self
    }
}

    }
}

impl_op!(Add, add, Point3);
impl_op!(Sub, sub, Point3);
impl_op!(Mul, mul, Point3);
impl_op!(Div, div, Point3);

impl_op!(Add, add, Vec3);
impl_op!(Sub, sub, Vec3);
impl_op!(Mul, mul, Vec3);
impl_op!(Div, div, Vec3);

// origin-point distance func
#[inline]
pub fn dist(r: &Ray3, p: &Point3) -> f64 {
    Vec3::from(p.0.clone() - r.origin.0.clone()).norm()
}

impl Transform {
    pub fn id() -> Self {
        Transform(array![
            [ 1., 0., 0., 0. ],
            [ 0., 1., 0., 0. ],
            [ 0., 0., 1., 0. ],
            [ 0., 0., 0., 1. ],
        ])
    }

    pub fn pivot(t: f64, axis: Vec3, center: Point3) -> Self {
        let center: Vec3 = center.into();
        let mut r = Self::translate(center.clone()*-1.);
        r
            .apply(&Self::rotate(t, axis))
            .apply(&Self::translate(center));
        r
    }

    pub fn rotate(t: f64, axis: Vec3) -> Self {
        let (l, m, n) = axis.unpack();
        let ct = t.cos();
        let st = t.sin();
        let cm = 1. - ct;

        Transform(array![
            [l*l * cm +     ct, m*l * cm - n * st, n*l * cm + m * st, 0.],
            [l*m * cm + n * st, m*m * cm +     ct, n*m * cm - l * st, 0.],
            [l*n * cm - m * st, m*n * cm + l * st, n*n * cm +     ct, 0.],
            [0., 0., 0., 1.],
        ])

    }

    pub fn translate(ofs: Vec3) -> Self {
        let (x, y, z) = ofs.unpack();
        Transform(array![
            [ 1., 0., 0., x  ],
            [ 0., 1., 0., y  ],
            [ 0., 0., 1., z  ],
            [ 0., 0., 0., 1. ],
        ])
    }
}



impl Transformable for Transform {
    fn apply(&mut self, t: &Transform) -> &mut Self {
        general_mat_mul(1., &t.0, &(self.0.clone()),
                        0., &mut self.0);
        self
    }
}


impl Transformable for Point3 {
    fn apply(&mut self, t: &Transform) -> &mut Self {
        general_mat_vec_mul(1., &t.0, &(self.0.clone()),
                            0., &mut self.0);
        self
    }
}

// pub fn norm(v: &Array1<f64>) -> f64 {
//     unsafe {
//         f64::sqrt(v.uget(0).powf(2.) + v.uget(1).powf(2.) + v.uget(2).powf(2.))
//     }
// }

// fn cross_product(a: &Array1<f64>,  b: &Array1<f64>) -> Array1<f64> {
//     let mut r = a.clone();
// 
//     assert!(a.len() == 3 || a.len() == 4);
// 
//     unsafe {
//         let ax = a.uget(0);
//         let ay = a.uget(1);
//         let az = a.uget(2);
// 
//         let bx = b.uget(0);
//         let by = b.uget(1);
//         let bz = b.uget(2);
// 
//         *r.uget_mut(0) = ay*bz - az*by;
//         *r.uget_mut(1) = az*bx - ax*bz;
//         *r.uget_mut(2) = ax*by - ay*bx;
//     }
// 
//     r
// }

impl Polygon {
    #[inline]
    pub fn bounds(&self) -> [Point3; 2] {
        match self {
            Polygon::Triangle(p0, p1, p2) => {
                [
                    p0.lower_bound(p1.lower_bound(p2.clone())),
                    p0.upper_bound(p1.upper_bound(p2.clone())),
                ]
            }
        }
    }
}

impl Transformable for  Polygon {
    fn apply(&mut self, t: &Transform) -> &mut Self {
        match self {
            Polygon::Triangle(p0, p1, p2) => {
                p0.apply(t);
                p1.apply(t);
                p2.apply(t);
            }
        }
        self
    }
}

impl Intersectable for Polygon {
    // TODO: ray should be passed by immut reference
    fn intersect(&self, ray: &Ray3) -> Option<(Point3, f64)> {
        match self {
            Polygon::Triangle(p0, p1, p2) => {
                let Ray3 { origin: rayO, vec: rayV, .. } = ray;
                let rayV:  Point3 = rayV.clone().into();
                let (p0, p1, p2) = (p0.clone(), p1.clone(), p2.clone());

                const EPSILON: f64 = 0.0000001;

                let edge1 = p1 - &p0;
                let edge2 = p2 - &p0;

                let h = edge2.cross_product(rayV.clone());
                let a = edge1.dot(&h);

                if a > -EPSILON && a < EPSILON {
                    return None;
                }

                let f = 1./a;
                let s = rayO.clone() - &p0;
                let u = f * (s.dot(&h));

                if u < 0.0 || u > 1.0 {
                    return None;
                }

                let q = edge1.cross_product(s); // s could be mut movd here
                let v = f * rayV.dot(&q);

                if v < 0.0 || u + v > 1.0 {
                    return None;
                }

                let t = f * edge2.dot(&q);

                if t > EPSILON {
                    let p = rayO + rayV * t;
                    return Some((p.clone(), Vec3::from(rayO - p).norm()));
                } else {
                    return None;
                }
            },
        }
    }

}

impl CoordSys {
    pub fn new() -> Self {
        CoordSys(Transform::id())
    }

    pub fn apply_to<T: Transformable>(&self, mut x: T) -> T {
        x.apply(&self.0);
        x
    }

    pub fn apply_rel(&mut self, mut t: Transform) -> &mut Self {
        ::std::mem::swap(&mut self.0, &mut t);
        self.apply(&t)
    }
}

impl Transformable for CoordSys {
    fn apply(&mut self, t: &Transform) -> &mut Self {
        self.0.apply(t);
        self
    }
}

impl SharedCoordSys {
    pub fn new() -> Self {
        SharedCoordSys(Rc::new(RefCell::new(CoordSys::new())))
    }

    pub fn apply_to<T: Transformable>(&self, mut x: T) -> T {
        self.0.borrow().apply_to(x)
    }

    pub fn apply_rel(&mut self, t: Transform) -> &mut Self {
        self.0.borrow_mut().apply_rel(t);
        self
    }
}

impl Transformable for SharedCoordSys {
    fn apply(&mut self, t: &Transform) -> &mut Self {
        self.0.borrow_mut().apply(t);
        self
    }
}

impl Clone for SharedCoordSys {
    fn clone(&self) -> Self {
        SharedCoordSys(Rc::clone(&self.0))
    }
}

impl<T: Transformable> SelfRelative<T> {
    pub fn new(x: T) -> Self {
        SelfRelative(x, CoordSys::new())
    }

    pub fn into_abs(self) -> T { self.1.apply_to(self.0) }

    pub fn as_rel(&self) -> &T { &self.0 }

    pub fn as_rel_mut(&mut self) -> &mut T { &mut self.0 }

    pub fn abs_field<F: FnOnce(&T) -> R, R: Transformable>(&self, f: F) -> R {
        self.1.apply_to(f(self.as_rel()))
    }
}

impl<T: Transformable + Clone> SelfRelative<T> {
    pub fn get_abs(&self) -> T {
        self.1.apply_to(self.0.clone())
    }
}

impl<T: Transformable> Transformable for SelfRelative<T> {
    fn apply(&mut self, t: &Transform) -> &mut Self {
        self.1.apply(t);
        self
    }
}

impl<T: Transformable + Intersectable + Clone> Intersectable for SelfRelative<T> {
    fn intersect(&self, ray: &Ray3) -> Option<(Point3, f64)> {
        self.get_abs().intersect(ray)
    }
}

impl<T: Transformable> Relative<T> {
    pub fn new(sys: &SharedCoordSys, x: T) -> Self {
        Relative(x, sys.clone())
    }

    pub fn into_abs(self) -> T { self.1.apply_to(self.0) }

    pub fn as_rel(&self) -> &T { &self.0 }

    pub fn as_rel_mut(&mut self) -> &mut T { &mut self.0 }

    pub fn abs_field<F: FnOnce(&T) -> R, R: Transformable>(&self, f: F) -> R {
        self.1.apply_to(f(self.as_rel()))
    }

    pub fn abs_map_ref<F: FnOnce(&T, &SharedCoordSys) -> R, R>(&self, f: F) -> R {
        f(self.as_rel(), &self.1)
    }
}

impl<T: Transformable + Clone> Relative<T> {
    pub fn get_abs(&self) -> T {
        self.1.apply_to(self.0.clone())
    }
}

impl<T: Transformable> Transformable for Relative<T> {
    fn apply(&mut self, t: &Transform) -> &mut Self {
        self.1.apply(t);
        self
    }
}

impl<T: Transformable + Intersectable + Clone> Intersectable for Relative<T> {
    fn intersect(&self, ray: &Ray3) -> Option<(Point3, f64)> {
        self.get_abs().intersect(ray)
    }
}
