use colors_transform::{Hsl, Rgb, Color};
use image::{ColorType};
use crate::backend::FrameCopy;


/**
 * Minimum lightness for a pixel.
 */
const LIGHTNESS_MIN: f32 = 0.15;

/**
 * Minimum saturation for a pixel.
 */
const SATURATION_MIN: f32 = 0.15;

/**
 * How many pixels to skip in a chunk, for performance.
 */
const SKIP_PIXEL: usize = 16;

pub fn determine_prominent_color(frame_copy: FrameCopy) -> Hsl {
    if ColorType::Rgba8 != frame_copy.frame_color_type {
        panic!("Cannot handle frame!")
    };
    // Find the modal colour from the frame.
    // Split r,g,b into a 3 dimensional array.
    let mut heatmap = vec![vec![vec![0u32; 21]; 21]; 37];

    let mut most_prominent= Hsl::from(0.0, 0.0, 0.0);
    let mut most_prominent_idx = 0;
    for chunk in frame_copy.frame_mmap.chunks_exact(4 + (SKIP_PIXEL*4)) {
        let hsl = Rgb::from(chunk[0] as f32, chunk[1] as f32, chunk[2] as f32).to_hsl();

        // Reject any really dark colours.
        if hsl.get_lightness() < LIGHTNESS_MIN {
            continue;
        }
        if hsl.get_saturation() < SATURATION_MIN {
            continue;
        }
        // Split into 36 blocks
        let h_index = (hsl.get_hue() as usize) / 10;
        let s_index = (hsl.get_saturation() as usize) / 5;
        let l_index = (hsl.get_saturation() as usize) / 5;
        let new_prominence = heatmap[h_index][s_index][l_index] + 1;
        // With what's left, primary focus on getting the most prominent colour in the frame.
        heatmap[h_index][s_index][l_index] = new_prominence;
        if new_prominence > most_prominent_idx {
            most_prominent = Hsl::from(
                (h_index * 10) as f32,
                (s_index * 5) as f32,
                (l_index * 5) as f32,
            );
            most_prominent_idx = new_prominence;
        }
    }
    most_prominent
}

