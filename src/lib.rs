#![no_std]
#![allow(non_upper_case_globals)]
#![feature(const_fn_floating_point_arithmetic)]

use core::i16;

//Include the Arduboy Library
//Initialize the arduboy object
use arduboy_rust::prelude::*;
const arduboy: Arduboy2 = Arduboy2::new();

const CHAR_WIDTH: i16 = 6;
const CHAR_HEIGHT: i16 = 8;

const MAX_BANDS: u8 = 6;
const MIN_BANDS: u8 = 3;
const DEFAULT_BANDS: u8 = 4;

// Rounds down to nearest multiple
const fn round_down_to(unrounded: i16, multiple: i16) -> i16 {
    if unrounded >= 0 {
        (unrounded / multiple) * multiple
    } else {
        ((unrounded - multiple + 1) / multiple) * multiple
    }
}

// What the resistor bands can represent
enum ValType {
    Digit,
    Multiplier,
    Tolerance,
    TCR,
}

// A single band of the resistor
#[repr(C)]
struct Band {
    value: c_char,
    show: bool,
    vtype: ValType,
    x: i16,
    y: i16,
    width: u8,
    bandx: i16,
}

impl Band {
    const fn new(show: bool, vtype: ValType, x: i16, bandx: i16) -> Self {
        let width = Band::get_width(&vtype);

        Band {
            value: 0,
            show,
            vtype,
            x,
            y: TEXT_Y,
            width,
            bandx,
        }
    }

    const fn get_width(vtype: &ValType) -> u8 {
        match vtype {
            ValType::Digit => 1,
            ValType::Multiplier => 4,
            ValType::Tolerance => 5,
            ValType::TCR => 4,
        }
    }

    fn change_by(&mut self, increment: i8) {
        let new = self.value + increment;

        let (min, max) = match self.vtype {
            ValType::Digit => {
                self.value = new.rem_euclid(10);
                return;
            }
            ValType::Multiplier => (-3, 9),
            ValType::Tolerance => (0, 9),
            ValType::TCR => (0, 8),
        };

        if new <= max && new >= min {
            self.value = new
        }
    }

    fn change_to(&mut self, new: i8) {
        if let ValType::Multiplier = self.vtype {
            self.value = new - 3
        } else {
            self.value = new
        }
    }

    fn get_pointer(&self) -> i8 {
        if let ValType::Multiplier = self.vtype {
            self.value + 3
        } else {
            self.value
        }
    }

    fn display(&self) {
        if !self.show {
            return;
        }

        // Display number
        arduboy.set_cursor(self.x, self.y);
        match self.vtype {
            ValType::Digit => arduboy.print(self.value as i16),
            ValType::Multiplier => {
                let repeat: i16 = (self.value.rem_euclid(3)).into();
                match repeat {
                    1 => arduboy.print(f!(b"0 \0")),
                    2 => arduboy.print(f!(b"00\0")),
                    _ => arduboy.print(f!(b"  \0")),
                }

                arduboy.print(PREFIXES[(round_down_to(self.value.into(), 3) / 3 + 1) as usize]);
                sprites::draw_override(
                    self.x + CHAR_WIDTH * (self.width as i16 - 1),
                    self.y,
                    get_sprite_addr!(Ohm),
                    0,
                );
            }
            ValType::Tolerance => {
                arduboy.print(TOLERANCES[self.value as usize]);
                arduboy.print(f!(b"%\0"));
            }
            ValType::TCR => {
                let tcr = TCRs[self.value as usize];
                sprites::draw_override(
                    self.x + CHAR_WIDTH * (4 - tcr.len() as i16),
                    self.y,
                    get_sprite_addr!(Plus_Minus),
                    0,
                );
                arduboy.set_cursor(self.x + CHAR_WIDTH * (4 - tcr.len() as i16 + 1), self.y);
                arduboy.print(tcr);
                arduboy.print(f!(b"TCR\0"));
            }
        }

        // Display band
        sprites::draw_override(self.bandx, BAND_Y, get_sprite_addr!(Band), self.get_rgb().3);
        // Display abbreviation
        sprites::draw_self_masked(
            self.bandx + (BAND_WIDTH - ABBR_WIDTH as i16) / 2,
            ABBR_Y as i16,
            get_sprite_addr!(Abbreviations),
            self.get_rgb().4,
        )
    }

    fn display_rgb(&self) {
        write_led(self.get_rgb());
    }

    fn get_rgb(&self) -> &RGB {
        let arr = Band::rgb_arr_from_valtype(&self.vtype);
        if let ValType::Multiplier = self.vtype {
            &arr[self.value as usize + 3]
        } else {
            &arr[self.value as usize]
        }
    }

    fn rgb_arr_from_valtype(vtype: &ValType) -> &[RGB] {
        match vtype {
            ValType::Digit => &VALUE_COLORS,
            ValType::Multiplier => &MULTIPLIER_COLORS,
            ValType::Tolerance => &TOLERANCE_COLORS,
            ValType::TCR => &TCR_COLORS,
        }
    }
}

// All bands of the resistor
#[repr(C)]
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
        // Procedural Spacing
        let tot_width = (if bands >= 5 {
            TEXT_WIDTHS[0]
        } else {
            TEXT_WIDTHS[1]
        } + if bands >= 4 { TEXT_WIDTHS[2] } else { 0 }
            + if bands >= 6 { TEXT_WIDTHS[3] } else { 0 });
        let spacing: f32 =
            (WIDTH as i16 - tot_width) as f32 / (bands - if bands >= 5 { 2 } else { 1 }) as f32;

        // Start with left spacing
        let mut x: f32 = spacing + 1.0;

        let value1 = Band::new(true, ValType::Digit, x as i16, BAND_Xs[0]);
        x += CHAR_WIDTH as f32;
        let value10 = Band::new(true, ValType::Digit, x as i16, BAND_Xs[1]);
        if bands >= 5 {
            x += CHAR_WIDTH as f32;
        }
        let value100 = Band::new(bands >= 5, ValType::Digit, x as i16, BAND_Xs[2]);
        x += CHAR_WIDTH as f32;
        let multiplier_pow = Band::new(true, ValType::Multiplier, x as i16, BAND_Xs[3]);
        if bands >= 4 {
            x += (CHAR_WIDTH * Band::get_width(&ValType::Multiplier) as i16) as f32 + spacing;
        }
        let tolerance_index = Band::new(bands >= 4, ValType::Tolerance, x as i16, BAND_Xs[4]);

        if bands >= 6 {
            x += (CHAR_WIDTH * Band::get_width(&ValType::Tolerance) as i16) as f32 + spacing;
        }
        let tcr_index = Band::new(bands >= 6, ValType::TCR, x as i16, BAND_Xs[5]);

        Resistance {
            value1,
            value10,
            value100,
            multiplier_pow,
            tolerance_index,
            tcr_index,

            bands,
        }
    }

    // An easier way to index through the resistor correctly
    fn index(&self, i: u8) -> &Band {
        match self.bands {
            3 => match i {
                0 => return &self.value1,
                1 => return &self.value10,
                2 => return &self.multiplier_pow,
                _ => panic!(),
            },
            4 => match i {
                0 => &self.value1,
                1 => &self.value10,
                2 => &self.multiplier_pow,
                3 => &self.tolerance_index,
                _ => panic!(),
            },
            5 => match i {
                0 => &self.value1,
                1 => &self.value10,
                2 => &self.value100,
                3 => &self.multiplier_pow,
                4 => &self.tolerance_index,
                _ => panic!(),
            },
            6 => match i {
                0 => &self.value1,
                1 => &self.value10,
                2 => &self.value100,
                3 => &self.multiplier_pow,
                4 => &self.tolerance_index,
                5 => &self.tcr_index,
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    fn index_mut(&mut self, i: u8) -> &mut Band {
        match self.bands {
            3 => match i {
                0 => &mut self.value1,
                1 => &mut self.value10,
                2 => &mut self.multiplier_pow,
                _ => panic!(),
            },
            4 => match i {
                0 => &mut self.value1,
                1 => &mut self.value10,
                2 => &mut self.multiplier_pow,
                3 => &mut self.tolerance_index,
                _ => panic!(),
            },
            5 => match i {
                0 => &mut self.value1,
                1 => &mut self.value10,
                2 => &mut self.value100,
                3 => &mut self.multiplier_pow,
                4 => &mut self.tolerance_index,
                _ => panic!(),
            },
            6 => match i {
                0 => &mut self.value1,
                1 => &mut self.value10,
                2 => &mut self.value100,
                3 => &mut self.multiplier_pow,
                4 => &mut self.tolerance_index,
                5 => &mut self.tcr_index,
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    fn display(&self) {
        for place in 0..self.bands {
            self.index(place).display();
        }
    }
}

struct RGB(u8, u8, u8, u8, u8);

fn write_led(color: &RGB) {
    arduboy.set_rgb_led(color.0, color.1, color.2)
}

// For colour selection menu
fn draw_menu(band_type: &ValType, menu_index: u8) {
    arduboy.draw_rect(
        ((WIDTH - MENU_SIZE - 2) / 2).into(),
        ((HEIGHT - MENU_SIZE - 2) / 2).into(),
        MENU_SIZE + 2,
        MENU_SIZE + 2,
        Color::White,
    );
    arduboy.fill_rect(
        ((WIDTH - MENU_SIZE) / 2).into(),
        ((HEIGHT - MENU_SIZE) / 2).into(),
        MENU_SIZE,
        MENU_SIZE,
        Color::Black,
    );

    let arr_len = Band::rgb_arr_from_valtype(&band_type).len();

    let mut count: i16 = 0;
    let x = (WIDTH - ABBR_WIDTH) as i16 / 2 - MENU_GAP - ABBR_WIDTH as i16;
    let y =
        (HEIGHT as i16 - (arr_len as i16 + 2) / 3 * (ABBR_HEIGHT as i16 + MENU_GAP) + MENU_GAP) / 2;

    for rgb in Band::rgb_arr_from_valtype(&band_type) {
        let center_final = if count + 1 == arr_len as i16 && arr_len % 3 != 0 {
            1
        } else {
            0
        };

        sprites::draw_override(
            x + ((count % 3) + center_final) * (ABBR_WIDTH as i16 + MENU_GAP),
            y + (count / 3) * (ABBR_HEIGHT as i16 + MENU_GAP),
            get_sprite_addr!(Abbreviations),
            rgb.4,
        );

        count += 1;
    }

    // If the total isn't divisible by 3, centre the last value
    let centre_final = if menu_index + 1 == arr_len as u8 && arr_len % 3 != 0 {
        1
    } else {
        0
    };

    sprites::draw_override(
        x + ((menu_index as i16 % 3) + centre_final) * (ABBR_WIDTH as i16 + MENU_GAP) - 4,
        y + (menu_index as i16 / 3) * (ABBR_HEIGHT as i16 + MENU_GAP),
        get_sprite_addr!(Arrow),
        0,
    )
}

// EEPROM
fn init_eeprom(eep: &EEPROMBYTECHECKLESS) -> u8 {
    eep.init();
    let saved_data = eep.read();
    if let MIN_BANDS..=MAX_BANDS = saved_data {
        saved_data
    } else {
        DEFAULT_BANDS
    }
}

fn save_eeprom(eep: &EEPROMBYTECHECKLESS, bands: u8) {
    eep.update(bands);
}

// Colours & orders

const PINK: RGB = RGB(255, 32, 128, Patterns::Vibrant as u8, 0);
const SILVER: RGB = RGB(40, 40, 40, Patterns::ShinyRev as u8, 1);
const GOLD: RGB = RGB(192, 64, 0, Patterns::Shiny as u8, 2);
const BLACK: RGB = RGB(0, 0, 0, Patterns::Black as u8, 3);
const BROWN: RGB = RGB(192, 32, 8, Patterns::Dull as u8, 4);
const RED: RGB = RGB(255, 0, 0, Patterns::Squared as u8, 5);
const ORANGE: RGB = RGB(255, 40, 0, Patterns::Striped as u8, 6);
const YELLOW: RGB = RGB(255, 128, 0, Patterns::Strips as u8, 7);
const GREEN: RGB = RGB(0, 255, 0, Patterns::Orbs as u8, 8);
const BLUE: RGB = RGB(0, 0, 192, Patterns::Snow as u8, 9);
const VIOLET: RGB = RGB(112, 0, 224, Patterns::Wavy as u8, 10);
const GRAY: RGB = RGB(24, 24, 24, Patterns::Gray as u8, 11);
const WHITE: RGB = RGB(255, 255, 255, Patterns::White as u8, 12);

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
    "0.01\0", "0.02\0", "0.05\0", "0.10\0", "0.25\0", "0.50\0", "1.00\0", "2.00\0", "5.00\0",
    "10.0\0",
];
const TCRs: [&str; 9] = [
    "1\0", "5\0", "10\0", "15\0", "20\0", "25\0", "50\0", "100\0", "250\0",
];

const PREFIXES: [&str; 5] = ["m\0", " \0", "k\0", "M\0", "G\0"];

const VALUES3_WIDTH: i16 = 7 * CHAR_WIDTH;
const VALUES2_WIDTH: i16 = 6 * CHAR_WIDTH;
const TOLERANCE_WIDTH: i16 = 5 * CHAR_WIDTH;
const TCR_WIDTH: i16 = 7 * CHAR_WIDTH;

const TEXT_WIDTHS: [i16; 4] = [VALUES3_WIDTH, VALUES2_WIDTH, TOLERANCE_WIDTH, TCR_WIDTH];

const TEXT_Y: i16 = (RES_Y - CHAR_HEIGHT) / 2;

const RES_Y: i16 = ((HEIGHT - RES_HEIGHT) / 2) as i16;
const RES_HEIGHT: u8 = 32;

const MENU_SIZE: u8 = 56;
const MENU_GAP: i16 = 6;

const ABBR_WIDTH: u8 = 7;
const ABBR_HEIGHT: u8 = 5;
const ABBR_Y: i16 = (HEIGHT as i16 + RES_Y + RES_HEIGHT as i16) / 2 - ABBR_HEIGHT as i16;

const BAND_Y: i16 = RES_Y;
const BAND_Xs: [i16; 6] = [32, 44, 56, 69, 82, 94];
const BAND_WIDTH: i16 = 6;

const EEPROM_ADDR: i16 = 416;

// Sprites
progmem!(
    static Ohm: [u8; 7] = [
        5, 7, // width, height,
        0x5e, 0x71, 0x01, 0x71, 0x5e,
    ];

    #[link_section = ".progmem.data"]
    static Plus_Minus: [u8; 7] = [
        5, 7, // width, height,
        0x44, 0x44, 0x5f, 0x44, 0x44,
    ];

    static Res: [u8; 514] = [
        128, 32, // width, height,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x08,
        0x04, 0x04, 0x02, 0x02, 0x02, 0x02, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x02,
        0x02, 0x02, 0x04, 0x04, 0x04, 0x08, 0x08, 0x08, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
        0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x08,
        0x08, 0x08, 0x08, 0x04, 0x04, 0x04, 0x02, 0x02, 0x02, 0x02, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x02, 0x02, 0x02, 0x02, 0x04, 0x04, 0x08, 0x08, 0x10, 0x20, 0x40, 0x80, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0,
        0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0,
        0x30, 0x0e, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x0e, 0x30, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0,
        0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0,
        0xc0, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03,
        0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x0c, 0x70, 0x80, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
        0x70, 0x0c, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03,
        0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x01, 0x02, 0x04, 0x08, 0x10, 0x10, 0x20, 0x20, 0x40, 0x40, 0x40, 0x40,
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x40, 0x40, 0x40, 0x40, 0x20, 0x20, 0x20, 0x10,
        0x10, 0x10, 0x10, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
        0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x10, 0x10, 0x10, 0x10, 0x20, 0x20, 0x20,
        0x40, 0x40, 0x40, 0x40, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x40, 0x40, 0x40, 0x40,
        0x20, 0x20, 0x10, 0x10, 0x08, 0x04, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];

    static ResMask: [u8; 512] = [
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f, 0x3f, 0x1f, 0x0f, 0x0f,
        0x07, 0x07, 0x03, 0x03, 0x03, 0x03, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x03, 0x03,
        0x03, 0x03, 0x07, 0x07, 0x07, 0x0f, 0x0f, 0x0f, 0x0f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f,
        0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x0f,
        0x0f, 0x0f, 0x0f, 0x07, 0x07, 0x07, 0x03, 0x03, 0x03, 0x03, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x03, 0x03, 0x03, 0x03, 0x07, 0x07, 0x0f, 0x0f, 0x1f, 0x3f, 0x7f, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0x3f, 0x0f, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x0f, 0x3f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfc, 0xf0, 0x80, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
        0xf0, 0xfc, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xfe, 0xfc, 0xf8, 0xf0, 0xf0, 0xe0, 0xe0, 0xc0, 0xc0, 0xc0, 0xc0,
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xc0, 0xc0, 0xc0, 0xc0, 0xe0, 0xe0, 0xe0, 0xf0,
        0xf0, 0xf0, 0xf0, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8,
        0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf0, 0xf0, 0xf0, 0xf0, 0xe0, 0xe0, 0xe0,
        0xc0, 0xc0, 0xc0, 0xc0, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xc0, 0xc0, 0xc0, 0xc0,
        0xe0, 0xe0, 0xf0, 0xf0, 0xf8, 0xfc, 0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff,
    ];

    static Abbreviations: [u8; 93] = [
        7, 5, // width, height,
        // PINK
        0x1f, 0x05, 0x07, 0x00, 0x1f, 0x04, 0x1b, // SILVER
        0x17, 0x15, 0x1d, 0x00, 0x1f, 0x05, 0x1a, // GOLD
        0x1f, 0x11, 0x19, 0x00, 0x1f, 0x11, 0x1e, // BLACK
        0x1f, 0x15, 0x1a, 0x00, 0x1f, 0x04, 0x1b, // BROWN
        0x1f, 0x15, 0x1a, 0x00, 0x1f, 0x01, 0x1f, // RED
        0x1f, 0x05, 0x1a, 0x00, 0x1f, 0x11, 0x1e, // ORANGE
        0x1f, 0x11, 0x1f, 0x00, 0x1f, 0x11, 0x19, // YELLOW
        0x07, 0x1c, 0x07, 0x00, 0x1f, 0x15, 0x15, // GREEN
        0x1f, 0x11, 0x19, 0x00, 0x1f, 0x01, 0x1f, // BLUE
        0x1f, 0x15, 0x1a, 0x00, 0x1f, 0x10, 0x1f, // VIOLET
        0x0f, 0x10, 0x0f, 0x00, 0x01, 0x1f, 0x01, // GRAY
        0x1f, 0x11, 0x19, 0x00, 0x07, 0x1c, 0x07, // WHITE
        0x1f, 0x08, 0x1f, 0x00, 0x1f, 0x04, 0x1f,
    ];

    static Arrow: [u8; 5] = [
        3, 5, // width, height,
        0x1f, 0x0e, 0x04,
    ];

    static Band: [u8; 314] = [
        6, 32, // width, height,
        // Black
        0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00,
        0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, // White
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //Gray
        0xff, 0x55, 0xaa, 0x55, 0xaa, 0xff, 0xff, 0x55, 0xaa, 0x55, 0xaa, 0xff, 0xff, 0x55, 0xaa,
        0x55, 0xaa, 0xff, 0xff, 0x55, 0xaa, 0x55, 0xaa, 0xff, // Shiny
        0xff, 0x24, 0x92, 0x92, 0x49, 0xff, 0xff, 0x49, 0x24, 0x24, 0x92, 0xff, 0xff, 0x92, 0x49,
        0x49, 0x24, 0xff, 0xff, 0x24, 0x92, 0x92, 0x49, 0xff, // Shiny Reversed
        0xff, 0x49, 0x92, 0x92, 0x24, 0xff, 0xff, 0x92, 0x24, 0x24, 0x49, 0xff, 0xff, 0x24, 0x49,
        0x49, 0x92, 0xff, 0xff, 0x49, 0x92, 0x92, 0x24, 0xff, // Vibrant
        0xff, 0x02, 0x47, 0xe2, 0x40, 0xff, 0xff, 0x08, 0x1c, 0x88, 0x00, 0xff, 0xff, 0x20, 0x71,
        0x23, 0x01, 0xff, 0xff, 0x80, 0xc4, 0x8e, 0x04, 0xff, // Dull
        0xff, 0x00, 0x11, 0x44, 0x00, 0xff, 0xff, 0x00, 0x11, 0x44, 0x00, 0xff, 0xff, 0x00, 0x11,
        0x44, 0x00, 0xff, 0xff, 0x00, 0x11, 0x44, 0x00, 0xff, // Orbs
        0xff, 0x18, 0x3c, 0x3c, 0x18, 0xff, 0xff, 0x86, 0xcf, 0xcf, 0x86, 0xff, 0xff, 0x61, 0xf3,
        0xf3, 0x61, 0xff, 0xff, 0x18, 0x3c, 0x3c, 0x18, 0xff, // Strips
        0xff, 0x66, 0x66, 0x66, 0x66, 0xff, 0xff, 0x66, 0x66, 0x66, 0x66, 0xff, 0xff, 0x66, 0x66,
        0x66, 0x66, 0xff, 0xff, 0x66, 0x66, 0x66, 0x66, 0xff, // Snow
        0xff, 0x05, 0xa2, 0x45, 0xa0, 0xff, 0xff, 0x14, 0x88, 0x14, 0x80, 0xff, 0xff, 0x50, 0x22,
        0x51, 0x02, 0xff, 0xff, 0x40, 0x8a, 0x44, 0x0a, 0xff, // Squared
        0xff, 0x03, 0x1b, 0xd8, 0xc0, 0xff, 0xff, 0x30, 0xb6, 0x86, 0x00, 0xff, 0xff, 0x00, 0x61,
        0x6d, 0x0c, 0xff, 0xff, 0x03, 0x1b, 0xd8, 0xc0, 0xff, // Wavy
        0xff, 0x92, 0x49, 0x92, 0x49, 0xff, 0xff, 0x24, 0x92, 0x24, 0x92, 0xff, 0xff, 0x49, 0x24,
        0x49, 0x24, 0xff, 0xff, 0x92, 0x49, 0x92, 0x49, 0xff, // Striped
        0xff, 0x00, 0xff, 0xff, 0x00, 0xff, 0xff, 0x00, 0xff, 0xff, 0x00, 0xff, 0xff, 0x00, 0xff,
        0xff, 0x00, 0xff, 0xff, 0x00, 0xff, 0xff, 0x00, 0xff,
    ];
);

// Options for the patterns that appear on the resistor bands
enum Patterns {
    Black,
    White,
    Gray,
    Shiny,
    ShinyRev,
    Vibrant,
    Dull,
    Orbs,
    Strips,
    Snow,
    Squared,
    Wavy,
    Striped,
}

//Initialize variables used in this game
static mut pointer: u8 = 0;
static mut menu_pointer: u8 = 0;
static mut resistance: Resistance = Resistance::new(DEFAULT_BANDS);
static mut show_menu: bool = false;

const EEPROM_CONFIRM_TIME: u16 = 30;
static mut eeprom_confirm_timer: u16 = 0;

// Setup eeprom memory
// EEPROMBYTECHECKLESS is a clone of the EEPROMBYTE struct without check digits
static mut eeprom: EEPROMBYTECHECKLESS = EEPROMBYTECHECKLESS::new(EEPROM_ADDR - 16);

//The setup() function runs once when you turn your Arduboy on
#[no_mangle]
pub unsafe extern "C" fn setup() {
    // put your setup code here, to run once:
    arduboy.begin();
    resistance = Resistance::new(init_eeprom(&eeprom));
    arduboy.clear();
    arduboy.set_frame_rate(30);
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

    // CONTROLS

    arduboy.poll_buttons();

    let current_rgb = Band::rgb_arr_from_valtype(&resistance.index(pointer).vtype);

    if !show_menu {
        if A.just_pressed() {
            menu_pointer = resistance.index_mut(pointer).get_pointer() as u8;
            show_menu = true;
        }
        if B.just_pressed() {
            if LEFT.pressed() && RIGHT.pressed() {
                // Save default bands button combo
                save_eeprom(&eeprom, resistance.bands);
                eeprom_confirm_timer = EEPROM_CONFIRM_TIME;
            } else {
                // Stick the pointer to currently selected band
                if resistance.bands == 4 && pointer > 1 {
                    pointer += 1;
                }

                // Increment, looping at 6 back to 3
                if resistance.bands < MAX_BANDS {
                    resistance = Resistance::new(resistance.bands + 1);
                } else {
                    resistance = Resistance::new(MIN_BANDS);
                }

                // Prevent invalid index call
                if pointer > resistance.bands - 1 {
                    pointer = resistance.bands - 1;
                }
            }
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

        resistance.index(pointer).display_rgb();
    } else {
        // Select colour choice
        if A.just_pressed() {
            resistance.index_mut(pointer).change_to(menu_pointer as i8);
            show_menu = false;
        }
        // Cancel menu
        if B.just_pressed() {
            show_menu = false;
        }

        // Move menu cursor left if possible
        if LEFT.just_pressed() && menu_pointer % 3 != 0 {
            if menu_pointer > 0 {
                menu_pointer -= 1;
            }
        }
        // Move menu cursor right if possible
        if RIGHT.just_pressed() {
            if menu_pointer < (current_rgb.len() - 1) as u8 && menu_pointer % 3 != 2 {
                menu_pointer += 1;
            }
        }
        if UP.just_pressed() {
            if menu_pointer > 2 {
                // If pointing to central bottom place, cursor will go directly up
                if menu_pointer == (current_rgb.len() - 1) as u8 && current_rgb.len() % 3 != 0 {
                    menu_pointer -= 2;
                } else {
                    menu_pointer -= 3;
                }
            }
        }
        if DOWN.just_pressed() {
            if menu_pointer < (current_rgb.len() - 3) as u8 {
                menu_pointer += 3;
            } else if current_rgb.len() % 3 != 0 {
                menu_pointer = (current_rgb.len() - 1) as u8
            }
        }

        write_led(&current_rgb[menu_pointer as usize])
    }

    // LED flashes to confirm EEPROM write
    if eeprom_confirm_timer > 0 {
        eeprom_confirm_timer -= 1;
        arduboy.set_rgb_led(96, 255, 16)
    }

    // DISPLAY

    // Increase width of selected band
    arduboy.draw_fast_vline(
        resistance.index(pointer).bandx - 1,
        RES_Y,
        RES_HEIGHT,
        Color::White,
    );
    arduboy.draw_fast_vline(
        resistance.index(pointer).bandx + BAND_WIDTH,
        RES_Y,
        RES_HEIGHT,
        Color::White,
    );
    // Display all bands
    resistance.display();

    // Draw resistor over bands
    sprites::draw_external_mask(
        0,
        RES_Y,
        get_sprite_addr!(Res),
        get_sprite_addr!(ResMask),
        0,
        0,
    );

    // Underline selected band text
    arduboy.draw_fast_hline(
        resistance.index(pointer).x - 1,
        TEXT_Y + CHAR_HEIGHT,
        resistance.index(pointer).width * CHAR_WIDTH as u8 + 1,
        Color::White,
    );

    // Draw menu
    if show_menu {
        draw_menu(&resistance.index(pointer).vtype, menu_pointer);
    }

// Draw border
    arduboy.draw_rect(0, 0, WIDTH, HEIGHT, Color::White);

    arduboy.display();
}
