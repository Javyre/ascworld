#[macro_use]
extern crate ndarray;

extern crate termion;

#[macro_use]
mod macros;

mod graphics;

use termion::color::*;
use termion::{
    cursor,
    clear,
    event,
    raw::IntoRawMode,
    input::TermRead,
};

use graphics::*;
use graphics::camera::*;
use graphics::scene::*;

use std::io::{Write, Read};

use std::time::{
    Instant,
    Duration,
};

use std::thread;
use std::sync::mpsc::channel;
use std::sync::{
    Arc,
    RwLock,
};

fn main() {
    let mut scene = Scene::new();
    let mut rendered_s = scene.empty_render();

    scene.objects.push(
        Object::new(
            Rgb(55, 155, 255),
            vec![
                Polygon::Triangle(
                    Point3::new(10., 10., -10.),
                    Point3::new(10., 20., -10.),
                    Point3::new(20., 20., -10.),
                ),
                Polygon::Triangle(
                    Point3::new(10., 10., -10.),
                    Point3::new(20., 10., -10.),
                    Point3::new(20., 20., -10.),
                ),
            ],
        )
    );
   scene.objects.push(
       Object::new(
           Rgb(155, 55, 155),
           vec![
               Polygon::Triangle(
                   Point3::new(10., 10., -10.),
                   Point3::new(10., 20., -10.),
                   Point3::new(10., 20., -20.),
               ),
               Polygon::Triangle(
                   Point3::new(10., 10., -10.),
                   Point3::new(10., 10., -20.),
                   Point3::new(10., 20., -20.),
               ),
           ],
       )
   );
   scene.objects.push(
       Object::new(
           Rgb(155, 255, 55),
           vec![
               Polygon::Triangle(
                   Point3::new(10., 10., -20.),
                   Point3::new(20., 10., -20.),
                   Point3::new(20., 20., -10.),
               ),
           ],
       )
   );

    let mut s = Instant::now();
    let mut e = s.elapsed();
    let mut stdout = ::std::io::stdout().into_raw_mode().unwrap();
    let mut running = Arc::new(RwLock::new(true));
    let events = ::std::io::stdin().events();

    let (te, re) = channel();

    let running_ = running.clone();
    let event_loop = thread::spawn(move || {
        for event in events {
            te.send(event).unwrap();
            if !*running_.read().unwrap() {
                break
            }
        }
    });

    while *running.read().unwrap() {
        write!(stdout, "{}{}{}{}\n\r",
              cursor::Goto(1,1),
              Fg(Rgb(200,200,55)),
              (1000./(e.as_secs() as f64 *1000.
                    + e.subsec_millis() as f64)).to_string() + "fps",
              clear::UntilNewline,
        );

        s = Instant::now();

        scene.render(&mut rendered_s);
        rendered_s.display(&mut stdout);

        scene.objects.iter_mut().for_each(|o| {
            o
            .apply(
                &Transform::pivot(
                    3.1415926*2./(60.*4.),
                    Vec3::new(0., 1., 0.),
                    Point3::new(15.,15.,-10.)
                )
            )
//            .apply(
//                &Transform::pivot(
//                    3.1415926*2./(60.*4.),
//                    Vec3::new(0., 0., 1.),
//                    Point3::new(10.,10.,-10.)
//                )
//            )
            ;
        });

        for event in re.try_iter() {
            match event.unwrap() {
                event::Event::Key(event::Key::Char('q')) => {
                    *running.write().unwrap() = false;
                },

                event::Event::Key(k) => {
                    match k {
                          event::Key::Char(c@'w')
                        | event::Key::Char(c@'a')
                        | event::Key::Char(c@'s')
                        | event::Key::Char(c@'d')=> {
                            scene.camera.apply_rel(
                                Transform::translate(
                                    match c {
                                        'w' => Vec3::new(0., 0., -2.5),
                                        'a' => Vec3::new(-2.5, 0., 0.),
                                        's' => Vec3::new(0., 0., 2.5),
                                        'd' => Vec3::new(2.5, 0., 0.),
                                        _ => unreachable!(),
                                    }
                                )
                            );
                        },

                          event::Key::Char(c@'W')
                        | event::Key::Char(c@'A')
                        | event::Key::Char(c@'S')
                        | event::Key::Char(c@'D') => {
                            let pivot = scene.camera.get_pivot();
                            scene.camera.apply(
                                &Transform::pivot(
                                    3.1415926*2./(60.*3.),
                                    match c {
                                        'W' => Vec3::new(-1., 0., 0.),
                                        'A' => Vec3::new(0., 1., 0.),
                                        'S' => Vec3::new(1., 0., 0.),
                                        'D' => Vec3::new(0., -1., 0.),
                                        _ => unreachable!(),
                                    },
                                    pivot
                                )
                            );
                        },

                        event::Key::Char('f') => {
                            scene.camera.eye.as_rel_mut().apply(
                                &Transform::translate(Vec3::new(0., 0., 5.))
                            );
                        },
                        event::Key::Char('F') => {
                            scene.camera.eye.as_rel_mut().apply(
                                &Transform::translate(Vec3::new(0., 0., -5.))
                            );
                        },
                        _ => {},
                    }
                }
                _ => {},
            }
        }

        // scene.camera.apply(
        //     &Transform::pivot(
        //         3.1415926*2./(60.*4.),
        //         Vec3::new(1., 0., 0.),
        //         Point3::new(10.,10.,-10.)
        //     )
        // );


        thread::sleep(Duration::from_millis(1000/60).checked_sub(s.elapsed()).unwrap_or(Duration::new(0,0)));

        e = s.elapsed();
        // break;
    }

    event_loop.join();

    return;

    let cell_dims = (0.25, 0.5);
    // let scrn_dims = (90, 35);
    let scrn_dims = (2, 2);
    let mut cam = Camera::new(
        cell_dims,
        scrn_dims,
        Point3::new(
            cell_dims.0 * scrn_dims.0 as f64 / 2.,
            cell_dims.1 * scrn_dims.1 as f64 / 2.,
            45.
        ),
    );

    println!("Hello, world!");

    // println!("corners: {:?}", cam.screen.corners);
    println!("center(1,1): {:?}", cam.get_center((1,1)));
    println!("ray(1,1): {:?}", cam.get_ray((1,1)));
    println!("------");

    cam.apply(&Transform::id());
    cam.apply(&Transform(array![
                        [1., 0., 0., 0.],
                        [0., 1., 0., 0.],
                        [0., 0., 1., 10.],
                        [0., 0., 0., 1.],
                        ]));

    // println!("corners: {:?}", cam.screen.corners);
    println!("center(1,1): {:?}", cam.get_center((1,1)));
    println!("ray(1,1): {:?}", cam.get_ray((1,1)));
    // println!("curr_t:  {:?}", cam.current_transform);

}
