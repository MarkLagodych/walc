#![no_std]
#![no_main]
mod walc;
use walc::*;

const FRAC_BITS: i32 = 16;
const ONE: i32 = 1 << FRAC_BITS;

const WIDTH: i32 = 160;
const HEIGHT: i32 = 50;

// This must be a power of 2 so that the division can be done with a shift
const MAX_ITER: i32 = 32;

// The length is a power of two so that the mapping can be done with a shift
const PALETTE: &[u8] = b" .:+xXM@";

// Mandelbrot region
const XMIN: i32 = -2 * ONE;
const XMAX: i32 = 1 * ONE;
const YMIN: i32 = -ONE;
const YMAX: i32 = 1 * ONE;

// Precomputed steps (no division at runtime)
const XSTEP: i32 = (XMAX - XMIN) / WIDTH;
const YSTEP: i32 = (YMAX - YMIN) / HEIGHT;

fn main() {
    let mut cy = YMIN;

    for _y in 0..HEIGHT {
        let mut cx = XMIN;

        for _x in 0..WIDTH {
            let mut zx: i32 = 0;
            let mut zy: i32 = 0;
            let mut iter: i32 = 0;

            while iter < MAX_ITER {
                let zx2 = ((zx as i64 * zx as i64) >> FRAC_BITS) as i32;
                let zy2 = ((zy as i64 * zy as i64) >> FRAC_BITS) as i32;

                if zx2 + zy2 > (4 * ONE) {
                    break;
                }

                let new_zx = zx2 - zy2 + cx;
                let new_zy = ((zx as i64 * zy as i64 * 2) >> FRAC_BITS) as i32 + cy;

                zx = new_zx;
                zy = new_zy;

                iter += 1;
            }

            // Map iteration to palette
            let idx = (iter * ((PALETTE.len() as i32) - 1) / MAX_ITER) as usize;
            print_byte(PALETTE[idx]);

            cx += XSTEP;
        }

        print_byte(b'\n');
        cy += YSTEP;
    }
}
