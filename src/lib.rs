#![no_std]
#![allow(non_upper_case_globals)]

use core::i16;

//Include the Arduboy Library
//Initialize the arduboy object
use arduboy_rust::prelude::*;
const arduboy: Arduboy2 = Arduboy2::new();

const CHAR_WIDTH: i16 = 6;
const CHAR_HEIGHT: i16 = 8;

struct Resistance {
    resistance: c_int,
}

impl Resistance {
    fn get_digit(&self, place: u8) -> i16 {
        let power = 10_i16.pow(place.into());
        self.resistance / power % 10
    }

    fn increase(&mut self, place: u8) {
        let place_digit = self.get_digit(place);
        self.resistance += if place_digit != 9 { 1 } else { -9 } * 10_i16.pow(place.into());
    }

    fn decrease(&mut self, place: u8) {
        let place_digit = self.get_digit(place);
        self.resistance -= if place_digit != 0 { 1 } else { -9 } * 10_i16.pow(place.into());
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

//Initialize variables used in this game
static mut bands: u8 = 4;

static mut pointer: u8 = 0;
static mut resistance: Resistance = Resistance { resistance: 1234 };

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

    // let messagestr = "Hais from Rust\0";

    // arduboy.set_cursor(
    //     (WIDTH as i16 - messagestr.len() as i16 * CHAR_WIDTH) / 2,
    //     (HEIGHT as i16 - CHAR_HEIGHT) / 2,
    // );
    // arduboy.print(messagestr);

    if LEFT.just_pressed() {
        if pointer > 0 {
            pointer -= 1;
        }
    }
    if RIGHT.just_pressed() {
        if pointer < bands - 1 {
            pointer += 1;
        }
    }
    if UP.just_pressed() {
        resistance.increase(get_place());
    }
    if DOWN.just_pressed() {
        resistance.decrease(get_place())
    }

    arduboy.set_cursor(0, 0);

    for place in (0..bands).rev() {
    arduboy.print(resistance.get_digit(place));
    }

    arduboy.draw_fast_hline(
        pointer as i16 * CHAR_WIDTH,
        CHAR_HEIGHT + 1,
        CHAR_WIDTH as u8,
        Color::White,
    );

    let i: usize = (resistance.get_digit(get_place())) as usize;

    write_led(&COLORS[i]);

    arduboy.set_cursor(0, (HEIGHT/2).into());
    arduboy.print(pointer as i16);

    arduboy.display();
}

unsafe fn get_place() -> u8 {
    bands - 1 - pointer
}