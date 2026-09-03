use crate::color_rgba::ColorRGBA;

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
