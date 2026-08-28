use crate::direction::Direction;
use crate::geometry::{Coordinate, Size};

#[derive(Debug, Clone, Copy)]
pub struct Zone {
    pub position: Coordinate,
    pub size: Size,
}

impl Zone {
    /// Split this zone in half.
    pub fn halved(&self, direction: Direction) -> Zone {
        let half_width = self.size.width / 2;
        let half_height = self.size.height / 2;

        if direction == Direction::Left {
            Zone {
                position: self.position,
                size: Size {
                    width: half_width,
                    height: self.size.height,
                },
            }
        } else if direction == Direction::Right {
            Zone {
                position: Coordinate {
                    x: self.position.x + half_width,
                    y: self.position.y,
                },
                size: Size {
                    width: half_width,
                    height: self.size.height,
                },
            }
        } else if direction == Direction::Up {
            Zone {
                position: self.position,
                size: Size {
                    width: self.size.width,
                    height: half_height,
                },
            }
        } else {
            // Direction::Down
            Zone {
                position: Coordinate {
                    x: self.position.x,
                    y: self.position.y + half_height,
                },
                size: Size {
                    width: self.size.width,
                    height: half_height,
                },
            }
        }
    }
}
