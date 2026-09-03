use crate::{
    draw_geometry::draw_filled_rectangle, labels::Labels, main_layer::MainLayer, zone::Zone,
};

use std::convert::TryInto;

use smithay_client_toolkit::{
    compositor::FrameCallbackData,
    shell::WaylandSurface,
    shm::slot::{Buffer, SlotPool},
};
use wayland_client::{QueueHandle, protocol::wl_shm};

use crate::draw_grid::{draw_grid, draw_grid_labels};

impl MainLayer {
    pub fn create_canvas<'a>(
        pool: &'a mut SlotPool,
        buffer: &'a mut Option<Buffer>,
        width: u32,
        height: u32,
    ) -> &'a mut [u8] {
        let stride = width as i32 * 4;

        let (new_buffer, canvas) = pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("create buffer");
        *buffer = Some(new_buffer);
        canvas
    }

    pub fn hide(&mut self) {
        self.layer.wl_surface().attach(None, 0, 0);
        self.layer.wl_surface().commit();
    }

    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;

        if self.need_redraw {
            // Try to use the same canvas again
            let mut canvas = if self.buffer.is_none() {
                Self::create_canvas(&mut self.pool, &mut self.buffer, width, height)
            } else {
                let c = self.pool.canvas(self.buffer.as_ref().unwrap());
                if let Some(cv) = c {
                    cv
                } else {
                    Self::create_canvas(&mut self.pool, &mut self.buffer, width, height)
                }
            };

            // Build the labels before the first real keymap event
            if self.labels.is_none() {
                self.labels = Some(Labels::rebuild(&self.xkb_parser));
            }
            let labels = self.labels.as_ref().unwrap();

            let sel = &self.selection[self.current_selection_index];

            // Quickly darken all background
            let dark_background: [u8; 4] = self.theme.background_color.into();
            {
                canvas.chunks_exact_mut(4).for_each(|chunk| {
                    let array: &mut [u8; 4] = chunk.try_into().unwrap();
                    *array = dark_background;
                });
            }

            let empty: Vec<Vec<String>> = vec![vec![String::new()]];

            // Draw the grid: full screen, one column, or the zone.
            let (
                zone_x,
                zone_y,
                zone_width,
                zone_height,
                cells_count_x,
                cells_count_y,
                cell_labels,
                font_size,
            ) = match (
                sel.selected_column,
                sel.selected_line,
                sel.selected_division,
            ) {
                // No selection : 10x30 grid
                (None, _, _) => (0, 0, width, height, 10, 30, &labels.all, 32f32),

                // A column is selected : trace only one column
                (Some(column), None, _) => (
                    column * (width / 10),
                    0,
                    width / 10,
                    height,
                    1,
                    30,
                    &labels.column,
                    32f32,
                ),

                (Some(column), Some(line), None) => {
                    let cell_width = self.width / 10;
                    let cell_height = self.height / 30;
                    (
                        column * cell_width,
                        line * cell_height,
                        cell_width,
                        cell_height,
                        10,
                        3,
                        &labels.divisions,
                        16f32,
                    )
                }

                // Only the zone: no grid, no labels.
                (Some(_column), Some(_line), Some(_division)) => {
                    let base_zone = Zone::from_selection(sel, width, height);
                    let active_zone = if let Some(zone) = sel.zones.last() {
                        *zone
                    } else {
                        base_zone
                    };

                    (
                        active_zone.position.x,
                        active_zone.position.y,
                        active_zone.size.width,
                        active_zone.size.height,
                        1,
                        1,
                        &empty,
                        0.0, // No label here
                    )
                }
            };
            draw_filled_rectangle(
                zone_x,
                zone_y,
                zone_width,
                zone_height,
                self.theme.grid_cells_color,
                &mut canvas,
                width as usize,
            );

            draw_grid(
                zone_x,
                zone_y,
                zone_width,
                zone_height,
                cells_count_x,
                cells_count_y,
                self.theme.grid_lines_color,
                &mut canvas,
                width as usize,
            );

            draw_grid_labels(
                zone_x,
                zone_y,
                zone_width,
                zone_height,
                cells_count_x,
                cells_count_y,
                cell_labels,
                font_size,
                self.theme.text_color,
                self.theme.text_background_color,
                &self.text_renderer,
                &mut canvas,
                width as usize,
            );


            // Draw the center of a selection
            let selection = &self.selection[self.current_selection_index];
            if selection.selected_column.is_some() &&
                selection.selected_line.is_some() &&
                    selection.selected_division.is_none() {
                let column = selection.selected_column.unwrap();
                let line = selection.selected_line.unwrap();
                let cell_width = self.width / 10;
                let cell_height = self.height / 30;
                let center_x = column * cell_width + cell_width / 2;
                let center_y = line * cell_height + cell_height / 2;

                draw_filled_rectangle(
                    center_x - 3,
                    center_y - 3,
                    5,
                    5,
                    self.theme.text_color,
                    &mut canvas,
                    width as usize,
                );
            }


            self.buffer
                .as_ref()
                .unwrap()
                .attach_to(self.layer.wl_surface())
                .expect("buffer attach");

            self.need_redraw = false;
        }

        // Send damage every tick
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        // Keep the frame loop alive, then commit
        self.layer
            .wl_surface()
            .frame(qh, FrameCallbackData(self.layer.wl_surface().clone()));
        self.layer.commit();
    }
}
