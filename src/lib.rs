use image::{Rgb, RgbImage, imageops::{flip_vertical, invert}};
use imageproc::{drawing::{draw_line_segment_mut}, };
use itertools::Itertools;

pub fn build_signature_image(signature: &[u8]) -> RgbImage {
    let mut img = RgbImage::new(640, 200);
    let pen_strokes = signature
        .split(|byte| byte == &b'p')
        .map(|byte_array| byte_array.into_iter());
    
    let segments = pen_strokes.map(|mut stroke| {
        // start_byte contains the 10th and 11th bits of the x and y starting coordinate values
        let start_byte = *stroke.next().unwrap_or(&0) as u16;
        let x_start = ((start_byte & 0b0000000000001100) << 7) as i16;
        let y_start = ((start_byte & 0b0000000000000011) << 9) as i16;
        let coordinate_byte_triplets = stroke.map(|byte| (byte - 0x20) as u16).tuples();

        let coordinate_offsets = coordinate_byte_triplets
            .map(|(byte_1, byte_2, byte_3)| {
                // x and y values are spread across the 3 bytes
                // Byte 1: 0-0-x8-x7-x6-x5-x4-x3
                // Byte 2: 0-0-y8-y7-y6-y5-y4-y3
                // Byte 3: 0-0-x2-x1-x0-y2-y1-y0
                let mut x =
                    ((byte_1 & 0b0000000000111111) << 3) | ((byte_3 & 0b0000000000111000) >> 3);
                let mut y = ((byte_2 & 0b0000000000111111) << 3) | (byte_3 & 0b0000000000000111);

                // if bit 9 === 1 set value to negative (2's complement).
                if x & 0b1111111100000000 > 0 {
                    x = x | 0b1111111000000000;
                }
                if y & 0b1111111100000000 > 0 {
                    y = y | 0b1111111000000000;
                }
                (x as i16, y as i16)
            });

        coordinate_offsets
            .scan((x_start, y_start), |(x, y), (x_offset, y_offset)| {
                *x = *x + x_offset;
                *y = *y + y_offset;
                Some((*x as f32, *y as f32))
            })
            .tuple_windows::<(_, _)>()
    });

    segments.for_each(|segment| {
        segment.for_each(|(point_a, point_b)| {
            draw_line_segment_mut(
                &mut img,
                point_a,
                point_b,             
                Rgb([255, 255, 255]),
            );
        });
    });

    invert(&mut img);
    flip_vertical(&img)
}
