# Arduboy Resistance Calculator
This is a small application for the Arduboy that allows you to calculate the resistance of a resistor from its colour values and vice versa.

It was created as a test for writing for the device in Rust using the [Rust for Arduboy](https://github.com/ZennDev1337/Rust-for-Arduboy) library.

The program makes heavy use of the Arduboy's RGB LED, so running it on a device with one is advised, but the program is still useable without.

## Controls

**L + R** - Select band

**U + D** - Change value by one place

**A** - Open colour selection menu

**B** - Cycle number of bands on resistor

**A (in menu)** - Confirm colour choice & close menu

**B (in menu)** - Close menu

**Hold L and R, Press B** - Save current number of bands in EEPROM to be loaded on startup