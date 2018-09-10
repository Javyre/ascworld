// #[macro_export]
// macro_rules! point3 {
//     [$a:expr, $b:expr, $c:expr] => {
//         $crate::graphics::Point3(array![$a, $b, $c, 1.])
//     };
// }
// 
// #[macro_export]
// macro_rules! vec3 {
//     [$a:expr, $b:expr, $c:expr] => {
//         $crate::graphics::Vec3(array![$a, $b, $c, 0.])
//     };
// }

#[macro_export]
macro_rules! val_v3 {
    ($v:expr) => {
        unsafe { if $v.0.uget(3) != &0. { panic!("invalid v3") } }
    }
}

#[macro_export]
macro_rules! val_p3 {
    ($v:expr) => {
        unsafe { if $v.0.uget(3) != &1. { panic!("invalid p3") } }
    }
}
