#![no_std]
#![no_main]

use reflect_mirrors::{LineSegment, Sphere};
use reflect_numworks::{SimulationParams, SimulationRay};

#[used]
#[link_section = ".rodata.eadk_app_name"]
pub static EADK_APP_NAME: [u8; 10] = *b"HelloRust\0";

#[used]
#[link_section = ".rodata.eadk_api_level"]
pub static EADK_APP_API_LEVEL: u32 = 0;

#[used]
#[link_section = ".rodata.eadk_app_icon"]
pub static EADK_APP_ICON: [u8; 4250] = [255 ; 4250];

#[no_mangle]
pub fn main() {

    // coordinates of screen corners
    const TOP_LEFT: [f32 ; 2] = [0., 0.];
    const BOTTOM_RIGHT: [f32 ; 2] = [320., 111.];
    const BOTTOM_LEFT: [f32 ; 2] = [0., 111.];
    const TOP_RIGHT: [f32 ; 2] = [320., 0.];

    let mirrors = (
        Sphere::new([160f32, 111.], 50.),
        [
            LineSegment::new([TOP_LEFT, TOP_RIGHT]),
            LineSegment::new([TOP_LEFT, BOTTOM_LEFT]),
            LineSegment::new([BOTTOM_RIGHT, TOP_RIGHT]),
            LineSegment::new([BOTTOM_RIGHT, BOTTOM_LEFT]),
        ]
    );

    let rays = [SimulationRay::new([30., 30.], [2., 1.])];    

    reflect_numworks::run_simulation(&mirrors, rays, SimulationParams::default());
}