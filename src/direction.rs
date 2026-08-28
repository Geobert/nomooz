use smithay_client_toolkit::output::OutputInfo;
use wayland_client::protocol::wl_output;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Find the screen in one direction.
pub fn find_output_in_direction(
    current: &OutputInfo,
    outputs: &[(wl_output::WlOutput, OutputInfo)],
    direction: Direction,
) -> Option<wl_output::WlOutput> {
    if current.logical_position.is_none() || current.logical_size.is_none() {
        return None;
    }
    let (cur_x, cur_y) = current.logical_position.unwrap();
    let (cur_width, cur_height) = current.logical_size.unwrap();

    let mut closest_output: Option<wl_output::WlOutput> = None;
    let mut closest_distance = 0;

    for (output, info) in outputs {
        if info.logical_position.is_none() || info.logical_size.is_none() {
            continue;
        }
        let (x, y) = info.logical_position.unwrap();
        let (width, height) = info.logical_size.unwrap();

        let mut is_candidate = false;
        let mut distance = 0;

        if direction == Direction::Left && x < cur_x {
            is_candidate = true;
            distance = cur_x - x;
        }
        if direction == Direction::Right && x > cur_x {
            is_candidate = true;
            distance = x - cur_x;
        }
        if direction == Direction::Up && y < cur_y {
            is_candidate = true;
            distance = cur_y - y;
        }
        if direction == Direction::Down && y > cur_y {
            is_candidate = true;
            distance = y - cur_y;
        }

        if !is_candidate {
            continue;
        }

        // Must be aligned on the other axis too.
        let mut same_level = false;
        if direction == Direction::Left || direction == Direction::Right {
            same_level = y < cur_y + cur_height && cur_y < y + height;
        }
        if direction == Direction::Up || direction == Direction::Down {
            same_level = x < cur_x + cur_width && cur_x < x + width;
        }

        if !same_level {
            continue;
        }

        if closest_output.is_none() || distance < closest_distance {
            closest_output = Some(output.clone());
            closest_distance = distance;
        }
    }

    closest_output
}
