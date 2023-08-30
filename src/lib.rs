#![no_std]
#![allow(non_upper_case_globals)]

use core::i16;
use core::ops::{Index, IndexMut};

//Include the Arduboy Library
//Initialize the arduboy object
use arduboy_rust::prelude::*;
const arduboy: Arduboy2 = Arduboy2::new();

const CHAR_WIDTH: i16 = 6;
const CHAR_HEIGHT: i16 = 8;

const TOLERANCES: [f32; 10] = [0.01, 0.1, 0.25, 0.5, 0.02, 0.05, 1.0, 2.0, 5.0, 10.0];
const TCRs: [u16; 9] = [1, 5, 10, 20, 25, 15, 50, 100, 250];

enum ValType {
    digit,
    multiplier,
    tolerance,
    tcr,
}

struct Value {
    value: c_char,
    show: bool,
    vtype: ValType,
}

impl Value {
    fn change_by(&mut self, increment: i8) {
        let (max_value, add) = match self.vtype {
            ValType::digit => (10, 0),
            ValType::multiplier => (13, 3),
            ValType::tolerance => (10, 0),
            ValType::tcr => (9, 0),
        };
        self.value = (self.value + increment + add).rem_euclid(max_value) - add;
    }
}

struct Resistance {
    value1: Value,
    value10: Value,
    value100: Value,
    multiplier_pow: Value,
    tolerance_index: Value,
    tcr_index: Value,
    bands: u8,
}

impl Resistance {
    const fn new(bands: u8) -> Self {
        Resistance {
            value1: Value {
                value: 0,
                show: true,
                vtype: ValType::digit,
            },
            value10: Value {
                value: 0,
                show: true,
                vtype: ValType::digit,
            },
            value100: Value {
                value: 0,
                show: true,
                vtype: ValType::digit,
            },
            multiplier_pow: Value {
                value: 0,
                show: true,
                vtype: ValType::multiplier,
            },
            tolerance_index: Value {
                value: 0,
                show: true,
                vtype: ValType::tolerance,
            },
            tcr_index: Value {
                value: 0,
                show: true,
                vtype: ValType::tcr,
            },
            bands,
        }
    }
}

impl Index<u8> for Resistance {
    type Output = Value;
    fn index(&self, i: u8) -> &Value {
        match self.bands {
            3 => match i {
                0 => &self.value1,
                1 => &self.value10,
                2 => &self.multiplier_pow,
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
}

impl IndexMut<u8> for Resistance {
    fn index_mut(&mut self, i: u8) -> &mut Value {
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

const BLACK: RGB = RGB(0, 0, 0);
const BROWN: RGB = RGB(192, 32, 8);
const RED: RGB = RGB(255, 0, 0);
const ORANGE: RGB = RGB(255, 40, 0);
const YELLOW: RGB = RGB(255, 128, 0);
const GREEN: RGB = RGB(0, 255, 0);
const BLUE: RGB = RGB(0, 0, 192);
const VIOLET: RGB = RGB(112, 0, 224);
const GRAY: RGB = RGB(24, 24, 24);
const WHITE: RGB = RGB(255, 255, 255);

const COLORS: [RGB; 10] = [
    BLACK, BROWN, RED, ORANGE, YELLOW, GREEN, BLUE, VIOLET, GRAY, WHITE,
];

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
        resistance[pointer].change_by(1);
    }
    if DOWN.just_pressed() {
        resistance[pointer].change_by(-1);
    }

    arduboy.set_cursor(0, 0);

    for place in (0..resistance.bands) {
        // arduboy.print(resistance.get_digit(place) as i16);
        arduboy.print(resistance[place].value as i16);
    }

    sprites::draw_override(
        CHAR_WIDTH * resistance.bands as i16,
        0,
        get_sprite_addr!(Ohm),
        0,
    );

    arduboy.draw_fast_hline(
        pointer as i16 * CHAR_WIDTH,
        CHAR_HEIGHT + 1,
        CHAR_WIDTH as u8,
        Color::White,
    );

    if let ValType::digit = resistance[pointer].vtype {
        let i = resistance[pointer].value as usize;
        write_led(&COLORS[i]);
    }

    arduboy.display();
}
