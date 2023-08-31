#![no_std]
#![allow(non_upper_case_globals)]

use core::i16;

//Include the Arduboy Library
//Initialize the arduboy object
use arduboy_rust::prelude::*;
const arduboy: Arduboy2 = Arduboy2::new();

const CHAR_WIDTH: i16 = 6;
const CHAR_HEIGHT: i16 = 8;

enum ValType {
    Digit,
    Multiplier,
    Tolerance,
    TCR,
}

struct Band {
    value: c_char,
    show: bool,
    vtype: ValType,
    x: i16,
    y: i16,
}

impl Band {
    fn change_by(&mut self, increment: i8) {
        let (max_value, add) = match self.vtype {
            ValType::Digit => (10, 0),
            ValType::Multiplier => (13, 3),
            ValType::Tolerance => (10, 0),
            ValType::TCR => (9, 0),
        };
        self.value = (self.value + increment + add).rem_euclid(max_value) - add;
    }

    fn display(&self) {
        if !self.show {
            return;
        }
        arduboy.set_cursor(self.x, self.y);
        match self.vtype {
            ValType::Digit => arduboy.print(self.value as i16),
            ValType::Multiplier => {
                arduboy.print(f!(b"10^\0"));
                arduboy.print(self.value as i16);
            }
            ValType::Tolerance => {
                arduboy.print(TOLERANCES[self.value as usize]);
                arduboy.print("%\0");
            }
            ValType::TCR => {
                arduboy.print("+\0"); //±
                arduboy.print(TCRs[self.value as usize]);
            },
        }
    }
    fn display_rgb(&self) {
        match self.vtype {
            ValType::Digit => write_led(&VALUE_COLORS[self.value as usize]),
            ValType::Multiplier => write_led(&MULTIPLIER_COLORS[self.value as usize + 3]),
            ValType::Tolerance => write_led(&TOLERANCE_COLORS[self.value as usize]),
            ValType::TCR => write_led(&TCR_COLORS[self.value as usize]),
        }
    }

    fn get_value(&self) -> c_char {
        self.value
    }
}

struct Resistance {
    value1: Band,
    value10: Band,
    value100: Band,
    multiplier_pow: Band,
    tolerance_index: Band,
    tcr_index: Band,
    bands: u8,
}

impl Resistance {
    const fn new(bands: u8) -> Self {
        Resistance {
            value1: Band {
                value: 0,
                show: true,
                vtype: ValType::Digit,
                x: CHAR_WIDTH * 0,
                y: CHAR_HEIGHT,
            },
            value10: Band {
                value: 0,
                show: true,
                vtype: ValType::Digit,
                x: CHAR_WIDTH * 1,
                y: CHAR_HEIGHT,
            },
            value100: Band {
                value: 0,
                show: true,
                vtype: ValType::Digit,
                x: CHAR_WIDTH * 2,
                y: CHAR_HEIGHT,
            },
            multiplier_pow: Band {
                value: 0,
                show: true,
                vtype: ValType::Multiplier,
                x: CHAR_WIDTH * 4,
                y: CHAR_HEIGHT,
            },
            tolerance_index: Band {
                value: 0,
                show: true,
                vtype: ValType::Tolerance,
                x: CHAR_WIDTH * 9,
                y: CHAR_HEIGHT,
            },
            tcr_index: Band {
                value: 0,
                show: true,
                vtype: ValType::TCR,
                x: CHAR_WIDTH * 15,
                y: CHAR_HEIGHT,
            },
            bands,
        }
    }

    fn index(&self, i: u8) -> &Band {
        match self.bands {
            3 => match i {
                0 => return &self.value1,
                1 => return &self.value10,
                2 => return &self.multiplier_pow,
                _ => panic!("unknown field: {}", i),
            },
            4 => match i {
                0 => &self.value1,
                1 => &self.value10,
                2 => &self.multiplier_pow,
                3 => &self.tolerance_index,
                _ => panic!("unknown field: {}", i),
            },
            5 => match i {
                0 => &self.value1,
                1 => &self.value10,
                2 => &self.value100,
                3 => &self.multiplier_pow,
                4 => &self.tolerance_index,
                _ => panic!("unknown field: {}", i),
            },
            6 => match i {
                0 => &self.value1,
                1 => &self.value10,
                2 => &self.value100,
                3 => &self.multiplier_pow,
                4 => &self.tolerance_index,
                5 => &self.tcr_index,
                _ => panic!("unknown field: {}", i),
            },
            _ => panic!("bad band num: {}", i),
        }
    }

    fn index_mut(&mut self, i: u8) -> &mut Band {
        match self.bands {
            3 => match i {
                0 => &mut self.value1,
                1 => &mut self.value10,
                2 => &mut self.multiplier_pow,
                _ => panic!("unknown field: {}", i),
            },
            4 => match i {
                0 => &mut self.value1,
                1 => &mut self.value10,
                2 => &mut self.multiplier_pow,
                3 => &mut self.tolerance_index,
                _ => panic!("unknown field: {}", i),
            },
            5 => match i {
                0 => &mut self.value1,
                1 => &mut self.value10,
                2 => &mut self.value100,
                3 => &mut self.multiplier_pow,
                4 => &mut self.tolerance_index,
                _ => panic!("unknown field: {}", i),
            },
            6 => match i {
                0 => &mut self.value1,
                1 => &mut self.value10,
                2 => &mut self.value100,
                3 => &mut self.multiplier_pow,
                4 => &mut self.tolerance_index,
                5 => &mut self.tcr_index,
                _ => panic!("unknown field: {}", i),
            },
            _ => panic!("bad band num: {}", i),
        }
    }
}

struct RGB(u8, u8, u8);

fn write_led(color: &RGB) {
    arduboy.set_rgb_led(color.0, color.1, color.2)
}

// Colours & orders

const PINK: RGB = RGB(255, 32, 128);
const SILVER: RGB = RGB(40, 40, 40);
const BLACK: RGB = RGB(0, 0, 0);
const GOLD: RGB = RGB(192, 64, 0);
const BROWN: RGB = RGB(192, 32, 8);
const RED: RGB = RGB(255, 0, 0);
const ORANGE: RGB = RGB(255, 40, 0);
const YELLOW: RGB = RGB(255, 128, 0);
const GREEN: RGB = RGB(0, 255, 0);
const BLUE: RGB = RGB(0, 0, 192);
const VIOLET: RGB = RGB(112, 0, 224);
const GRAY: RGB = RGB(24, 24, 24);
const WHITE: RGB = RGB(255, 255, 255);

const VALUE_COLORS: [RGB; 10] = [
    BLACK, BROWN, RED, ORANGE, YELLOW, GREEN, BLUE, VIOLET, GRAY, WHITE,
];

const MULTIPLIER_COLORS: [RGB; 13] = [
    PINK, SILVER, GOLD, BLACK, BROWN, RED, ORANGE, YELLOW, GREEN, BLUE, VIOLET, GRAY, WHITE,
];

const TOLERANCE_COLORS: [RGB; 10] = [
    GRAY, YELLOW, ORANGE, VIOLET, BLUE, GREEN, BROWN, RED, GOLD, SILVER,
];

const TCR_COLORS: [RGB; 9] = [GRAY, VIOLET, BLUE, ORANGE, GREEN, YELLOW, RED, BROWN, BLACK];

const TOLERANCES: [&str; 10] = [
    "0.01\0", "0.02\0", "0.05\0", "0.1\0", "0.25\0", "0.5\0", "1.0\0", "2.0\0", "5.0\0", "10.0\0",
];
const TCRs: [u16; 9] = [1, 5, 10, 15, 20, 25, 50, 100, 250];

// Sprites
#[link_section = ".progmem.data"]
static Ohm: [u8; 7] = [
    5, 7, // width, height,
    0x5e, 0x71, 0x01, 0x71, 0x5e,
];

//Initialize variables used in this game
static mut pointer: u8 = 0;
static mut resistance: Resistance = Resistance::new(6);

//The setup() function runs once when you turn your Arduboy on
#[no_mangle]
pub unsafe extern "C" fn setup() {
    // put your setup code here, to run once:
    arduboy.begin();
    arduboy.clear();
    arduboy.set_frame_rate(60);
}
//The loop() function repeats forever after setup() is done
#[no_mangle]
#[export_name = "loop"]
pub unsafe extern "C" fn loop_() {
    // put your main code here, to run repeatedly:
    //Skip cycles not in the framerate
    if !arduboy.next_frame() {
        return;
    }

    arduboy.clear();
    arduboy.poll_buttons();

    if A.just_pressed() {
        resistance = Resistance::new(resistance.bands + 1)
    }
    if B.just_pressed() {
        resistance = Resistance::new(resistance.bands - 1)
    }

    if LEFT.just_pressed() {
        if pointer > 0 {
            pointer -= 1;
        }
    }
    if RIGHT.just_pressed() {
        if pointer < resistance.bands - 1 {
            pointer += 1;
        }
    }
    if UP.just_pressed() {
        resistance.index_mut(pointer).change_by(1);
    }
    if DOWN.just_pressed() {
        resistance.index_mut(pointer).change_by(-1);
    }

    arduboy.set_cursor(0, 0);

    for place in 0..resistance.bands {
        resistance.index(place).display();
    }

    sprites::draw_override(CHAR_WIDTH * 19, CHAR_HEIGHT, get_sprite_addr!(Ohm), 0);

    arduboy.draw_fast_hline(
        resistance.index(pointer).x,
        CHAR_HEIGHT * 2 + 1,
        CHAR_WIDTH as u8,
        Color::White,
    );

    resistance.index(pointer).display_rgb();

    arduboy.display();
}
