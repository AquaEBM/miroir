#![no_std]
#![no_main]

use core::panic::PanicInfo;

use miroir_numworks::{
    eadk::{ion::*, kandinsky::*},
    run_simulation, SimulationParams, SimulationRay,
};
use miroir_shapes::{LineSegment, Sphere};

#[used]
#[link_section = ".rodata.eadk_app_name"]
static APP_NAME: [u8; 15] = *b"Trapped Circle\0";

#[used]
#[link_section = ".rodata.eadk_api_level"]
static API_LEVEL: u32 = 0;

#[used]
#[link_section = ".rodata.eadk_app_icon"]
static ICON: [u8; 4250] = *include_bytes!("icon.nwi");

#[no_mangle]
fn main() {
    fill_rect(
        Rect {
            point: Point { x: 0, y: 18 },
            w: 320,
            h: 222,
        },
        Color::from_rgb([78, 78, 78]),
    );

    const NUMWORKS_COL: Color = Color::from_rgb([248, 180, 48]);

    fill_rect(
        Rect {
            point: Point { x: 40, y: 0 },
            w: 240,
            h: 18,
        },
        NUMWORKS_COL,
    );

    unsafe {
        draw_string_unchecked(
            APP_NAME.as_ptr(),
            Point {
                x: 160 - APP_NAME.len() as i16 * 3,
                y: 3,
            },
            false,
            Color::from_rgb([255, 255, 255]),
            NUMWORKS_COL,
        )
    };

    // coordinates of screen corners
    const TOP_LEFT: [f32; 2] = [0., 18.];
    const BOTTOM_RIGHT: [f32; 2] = [319., 239.];
    const BOTTOM_LEFT: [f32; 2] = [0., 239.];
    const TOP_RIGHT: [f32; 2] = [319., 18.];
    const CENTER: [f32; 2] = [160., 129.];

    let mirrors = (
        Sphere::new(CENTER, 50.),
        [
            LineSegment::new([TOP_LEFT, TOP_RIGHT]),
            LineSegment::new([TOP_LEFT, BOTTOM_LEFT]),
            LineSegment::new([BOTTOM_RIGHT, TOP_RIGHT]),
            LineSegment::new([BOTTOM_RIGHT, BOTTOM_LEFT]),
        ],
    );

    let rays = [SimulationRay::new([10., 50.], [2., 1.]).with_reflection_cap(200)];

    run_simulation(
        &mirrors,
        rays,
        SimulationParams {
            // high epsilon because, with f32 precision, the ray ends up piercing through walls
            epsilon: f32::EPSILON * 1024.0,
            // a little pause between each reflection to see the ray's movements
            step_time_ms: 100,
            ..Default::default()
        },
    );

    while !{
        let scan = KeyboardState::scan();
        scan.key_down(Key::Back) | scan.key_down(Key::Power) | scan.key_down(Key::Home)
    } {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
