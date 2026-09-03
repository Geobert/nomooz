use crate::color_rgba::ColorRGBA;
use crate::text_renderer::TextRenderer;

pub fn draw_grid(
    x: u32,
    y: u32,
    width: u32,
    height: u32, // zone
    cells_count_x: u32,
    cells_count_y: u32,
    color: ColorRGBA,
    mut canvas: &mut [u8],
    canvas_width: usize,
) {
    let fact = (width / cells_count_x) as usize;
    for i in 1..cells_count_x as usize {
        for j in y as usize..(y as usize + height as usize) {
            for k in 0..3usize {
                color.print_on_canvas(
                    x as usize + i * fact + k - 1,
                    j,
                    canvas,
                    canvas_width,
                );
            }
        }
    }

    let fact = (height / cells_count_y) as usize;
    for i in 1..cells_count_y as usize {
        for j in x as usize..(x as usize+width as usize) {
            for k in 0..3usize {
                color.print_on_canvas(
                    j,
                    y as usize + i * fact + k - 1,
                    canvas,
                    canvas_width,
                );
            }
        }
    }
}

pub fn draw_grid_labels(
    x: u32,
    y: u32,
    width: u32,
    height: u32, // zone
    cells_count_x: u32,
    cells_count_y: u32,
    labels: &Vec<Vec<String>>,
    size: f32,
    color: ColorRGBA,
    circle_color: ColorRGBA,
    text_renderer: &TextRenderer,
    canvas: &mut [u8],
    canvas_width: usize,
) {
    let cell_width = width / cells_count_x;
    let cell_height = height / cells_count_y;

    for row in 0..cells_count_y as usize {
        for col in 0..cells_count_x as usize {
            let center_x = x + col as u32 * cell_width + cell_width / 2;
            let center_y = y + row as u32 * cell_height + cell_height / 2;

            text_renderer.print_text(
                &labels[row][col],
                center_x as f32,
                center_y as f32,
                size,
                color,
                circle_color,
                canvas,
                canvas_width,
            );
        }
    }
}
