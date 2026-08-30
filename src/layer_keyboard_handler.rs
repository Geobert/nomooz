use smithay_client_toolkit::{
    seat::keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
    shell::WaylandSurface,
};
use wayland_client::{
    Connection, QueueHandle,
    protocol::{wl_keyboard, wl_surface},
};

use crate::{
    cache::cache_data,
    direction::{Direction, find_output_in_direction},
    geometry::{Coordinate, Size},
    labels::Labels,
    main_layer::{DOUBLE_CLICK_WINDOW_MS, MainLayer},
    selection::Selection,
    virtual_pointer::ClickButton,
    zone::Zone,
};

impl KeyboardHandler for MainLayer {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _keysyms: &[Keysym],
    ) {
        if self.layer.wl_surface() == surface {
            self.keyboard_focus = true;
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if self.layer.wl_surface() == surface {
            self.keyboard_focus = false;
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        let mut need_draw = false;

        log::trace!("Key press: {event:?} {}", event.raw_code);
        // println!("Key press: {event:?} {}", event.raw_code);

        if !self.modifiers.shift {
            // press 'esc' to exit
            if event.keysym == Keysym::Escape {
                self.exit = true;
            }

            if event.keysym == Keysym::space {
                log::debug!("Final left click");
                self.click_button = Some(ClickButton::Left);
                self.double_click = false;
                self.exit = true;
            }

            // 50/51/52 = left/middle/right click, double click included.
            let explicit_button = if event.raw_code == 50 {
                Some(ClickButton::Left)
            } else if event.raw_code == 51 {
                Some(ClickButton::Middle)
            } else if event.raw_code == 52 {
                Some(ClickButton::Right)
            } else {
                None
            };

            if let Some(button) = explicit_button {
                if self.selection.len() == 1 {
                    let is_repeat_click =
                        if let Some((pending_button, pending_time)) = self.pending_click {
                            pending_button == button
                                && event.time.saturating_sub(pending_time) <= DOUBLE_CLICK_WINDOW_MS
                        } else {
                            false
                        };

                    if is_repeat_click {
                        log::debug!("Double click");
                        self.click_button = Some(button);
                        self.double_click = true;
                        self.pending_click = None;
                        self.exit = true;
                    } else {
                        log::debug!("Click pending (waiting for a possible double click)");
                        self.pending_click = Some((button, event.time));
                    }
                } else {
                    log::debug!("Final click (multi-selection, immediate)");
                    self.click_button = Some(button);
                    self.double_click = false;
                    self.exit = true;
                }
            }

            if self.current_selection_index == 0
                && !self.selection[self.current_selection_index]
                    .selected_line
                    .is_none()
            {
                if event.keysym == Keysym::Return {
                    log::debug!("New selection");

                    self.selection.push(Selection {
                        selected_column: None,
                        selected_line: None,
                        zones: Vec::new(),
                        output: self.current_output.clone(),
                    });
                    self.current_selection_index += 1;
                    need_draw = true;
                }
            }

            if event.keysym == Keysym::BackSpace {
                log::debug!("Undo");
                self.pending_click = None;
                if !self.selection[self.current_selection_index]
                    .zones
                    .is_empty()
                {
                    self.selection[self.current_selection_index].zones.pop();
                    need_draw = true;
                } else if !self.selection[self.current_selection_index]
                    .selected_line
                    .is_none()
                {
                    self.selection[self.current_selection_index].selected_line = None;
                    need_draw = true;
                } else if !self.selection[self.current_selection_index]
                    .selected_column
                    .is_none()
                {
                    self.selection[self.current_selection_index].selected_column = None;
                    need_draw = true;
                }
            }

            if self.selection[self.current_selection_index]
                .selected_column
                .is_none()
            {
                for i in 0..10 {
                    if event.raw_code == 30 + i {
                        log::debug!("Column selected: {i}");
                        self.selection[self.current_selection_index].selected_column = Some(i);
                        need_draw = true;
                    }
                }
            } else if self.selection[self.current_selection_index]
                .selected_line
                .is_none()
            {
                for (key_line, first_key) in vec![16, 30, 44].iter().enumerate() {
                    for i in 0..10 {
                        if event.raw_code == first_key + i {
                            let choice = key_line as u32 * 10 + i;
                            log::debug!("Line selected: {choice}");
                            self.selection[self.current_selection_index].selected_line =
                                Some(choice);
                            need_draw = true;
                        }
                    }
                }
            } else {
                if event.raw_code == 23 {
                    let current_zone = if let Some(zone) =
                        self.selection[self.current_selection_index].zones.last()
                    {
                        *zone
                    } else {
                        Zone::from_selection(
                            &self.selection[self.current_selection_index],
                            self.width,
                            self.height,
                        )
                    };

                    if let Some(new_zone) = current_zone.halved() {
                        self.selection[self.current_selection_index]
                            .zones
                            .push(new_zone);

                        need_draw = true;
                    }
                }

                // Try to move the current zone (and create it if it doen’t exist
                let direction = if event.raw_code == 35 || event.keysym == Keysym::Left {
                    Some(Direction::Left)
                } else if event.raw_code == 38 || event.keysym == Keysym::Right {
                    Some(Direction::Right)
                } else if event.raw_code == 36 || event.keysym == Keysym::Down {
                    Some(Direction::Down)
                } else if event.raw_code == 37 || event.keysym == Keysym::Up {
                    Some(Direction::Up)
                } else {
                    None
                };

                if let Some(direction) = direction {
                    let current_zone = if let Some(zone) =
                        self.selection[self.current_selection_index].zones.last()
                    {
                        *zone
                    } else {
                        Zone::from_selection(
                            &self.selection[self.current_selection_index],
                            self.width,
                            self.height,
                        )
                    };

                    if let Some(new_zone) = current_zone.moved(direction, self.width, self.height) {
                        self.selection[self.current_selection_index]
                            .zones
                            .push(new_zone);
                    } else {
                        if self.selection[self.current_selection_index]
                            .zones
                            .is_empty()
                        {
                            self.selection[self.current_selection_index]
                                .zones
                                .push(current_zone);
                        }
                    }

                    need_draw = true;
                }
            }
        } else {
            // Shift pressed !
            // TODO : Remove hardcoded values
            let direction = if event.raw_code == 35 || event.keysym == Keysym::Left {
                Some(Direction::Left)
            } else if event.raw_code == 38 || event.keysym == Keysym::Right {
                Some(Direction::Right)
            } else if event.raw_code == 36 || event.keysym == Keysym::Down {
                Some(Direction::Down)
            } else if event.raw_code == 37 || event.keysym == Keysym::Up {
                Some(Direction::Up)
            } else {
                None
            };

            log::debug!("Direction requested: {direction:?}");
            if let Some(direction) = direction {
                if let Some(current_output) = self.current_output.clone() {
                    if let Some(current_info) = self.output_info(&current_output) {
                        log::debug!("Current output: {current_info:?}");
                        let outputs = self.outputs();
                        log::debug!("Known outputs: {outputs:?}");
                        if let Some(output) =
                            find_output_in_direction(&current_info, &outputs, direction)
                        {
                            log::debug!("Target output found: {output:?}");
                            self.switch_output(qh, &output);
                        } else {
                            log::warn!("No target output found in this direction");
                        }
                    }
                } else {
                    log::warn!("current_output is None: surface_enter not received yet");
                }
            }
        }

        if need_draw {
            self.need_redraw = true;
        }
    }

    // Required by the trait, unused here.
    fn repeat_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        _event: KeyEvent,
    ) {
    }

    // Required by the trait, unused here.
    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _event: KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        modifiers: Modifiers,
        _raw_modifiers: RawModifiers,
        layout: u32,
    ) {
        self.modifiers = modifiers;

        // Set the layout for the current loaded keymap
        if let Err(error) = cache_data("layout.dat", layout.to_string()) {
            log::error!("Can’t access cache directory: {}", error);
        }
        self.xkb_parser.set_layout(layout as usize);
        self.labels = Some(Labels::rebuild(&self.xkb_parser));
        self.need_redraw = true;
    }

    fn update_keymap(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        keymap: smithay_client_toolkit::seat::keyboard::Keymap<'_>,
    ) {
        // Load the new provided xkb keymap
        if let Err(error) = cache_data("keymap.xkb", keymap.as_string()) {
            log::error!("Can’t access cache directory: {}", error);
        }
        if let Err(error) = self.xkb_parser.parse_from_string(keymap.as_string()) {
            log::error!("Can’t parse xkb-keymap: {error}");
            return;
        }
        self.labels = Some(Labels::rebuild(&self.xkb_parser));
        self.need_redraw = true;
    }
}
