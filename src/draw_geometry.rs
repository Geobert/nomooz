use crate::color_rgba::ColorRGBA;

/// Draw a filled circle, centered on (cx, cy), with radius r.
/// Taken from : https://en.wikipedia.org/wiki/Midpoint_circle_algorithm
pub fn draw_filled_circle(
    cx: i32,
    cy: i32,
    r: i32,
    color: ColorRGBA,
    canvas: &mut [u8],
    canvas_width: usize,
) {
    let mut x = r;
    let mut y = 0;
    let mut p = 1 - r;

    while x >= y {
        for px in (cx - x)..=(cx + x) {
            // Skip negative positions, they would crash as usize.
            if px >= 0 {
                let py_top = cy + y;
                if py_top >= 0 {
                    color.print_on_canvas(px as usize, py_top as usize, canvas, canvas_width);
                }
                let py_bottom = cy - y;
                if py_bottom >= 0 {
                    color.print_on_canvas(px as usize, py_bottom as usize, canvas, canvas_width);
                }
            }
        }

        for px in (cx - y)..=(cx + y) {
            if px >= 0 {
                let py_top = cy + x;
                if py_top >= 0 {
                    color.print_on_canvas(px as usize, py_top as usize, canvas, canvas_width);
                }
                let py_bottom = cy - x;
                if py_bottom >= 0 {
                    color.print_on_canvas(px as usize, py_bottom as usize, canvas, canvas_width);
                }
            }
        }

        y += 1;
        if p > 0 {
            x -= 1;
            p += 1 - 2 * x + 2 * y;
        } else {
            p += 1 + 2 * y;
        }
    }
}

// Quickly draw a filled rectangle.
// It completely override pixels and do not manage transparency.
pub fn draw_filled_rectangle(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: ColorRGBA,
    canvas: &mut [u8],
    canvas_width: usize,
) {
    let (x, y, width, height) = (x as usize, y as usize, width as usize, height as usize);
    for j in y..y + height {
        // Compute start and end of each line
        let start = (canvas_width * j + x) * 4;
        let end = start+(width * 4);

        // Fill the line
        canvas[start..end]
            .chunks_exact_mut(4)
            .for_each(|chunk| {
                    let array: &mut [u8; 4] = chunk.try_into().unwrap();
                    *array = color.into();
            })
    }
}
