// use rand::Rng;
use codec::{Encode, Decode};
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, Clone, PartialEq, TypeInfo)]
pub enum Direction {
    None = 0,
    Left = 1,
    Forward = 2,
    Right = 3,
}

#[derive(Debug, Encode, Decode, Clone, PartialEq, TypeInfo)]
pub enum Weapon {
	None,
	Rock,
	Paper,
	Scissor,
	Trap,
	King,
}
impl Default for Weapon { fn default() -> Self { Self::None } }


pub struct Logic {
}

impl Logic {

    pub fn initialize() -> [[u8; 6]; 7] {
        let mut board = [[u8::MAX; 6]; 7];

        for y in 0..board[0].len() {
            for x in 0..board.len() {
                let pos: u8 = (y * 7 + x) as u8;
                if pos < 14 {
                    board[x][y] = pos;
                } else if pos > 27 {
                    board[x][y] = 15 + (42 - pos);
                }
            }
        }

        board
    }

    pub fn position(position: [u8;2]) -> bool {

        position[0] < 7 && position[1] < 6
    }

    pub fn destination(player: u8, position: &mut [u8; 2], direction: u8) -> bool {

        if ((player == 0 && direction ==  1) || (player == 1 && direction == 3)) && position[0] > 0 {
            position[0] = position[0] - 1;
            return true;
        } else if ((player == 0 && direction == 3) || (player == 1 && direction == 1)) && position[0] < 6 {
            position[0] = position[0] + 1;
            return true;
        } else if player == 0 && direction == 2 && position[1] < 5 {
            position[1] = position[1] + 1;
            return true;
        } else if player == 1 && direction == 2 && position[1] > 0 {
            position[1] = position[1] - 1;
            return true;
        } else {
            return false;
        }
    }

    pub fn combat(a: &Weapon, b: &Weapon) -> u8 {
        match a {
            Weapon::Rock => {
                match b {
                    Weapon::Rock => return u8::MAX,
                    Weapon::Paper => return 1u8,
                    Weapon::Scissor => return 0u8,
                    _ => u8::MAX,
                }
            },
            Weapon::Paper => {
                match b {
                    Weapon::Rock => return 0u8,
                    Weapon::Paper => return u8::MAX,
                    Weapon::Scissor => return 1u8,
                    _ => u8::MAX,
                }
            },
            Weapon::Scissor => {
                match b {
                    Weapon::Rock => return 1u8,
                    Weapon::Paper => return 0u8,
                    Weapon::Scissor => return u8::MAX,
                    _ => u8::MAX,
                }    
            },
            _ => u8::MAX,
        }
    }
}