use cpal::SampleRate;
use crate::keyboard::MidiKeyboard;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use effects::set_effects_sample_rate;
use std::borrow::{BorrowMut, Borrow};

#[derive(Debug)]
pub struct State {
    pub sample_rate: SampleRate,
    pub sample_clock: u64,
    pub midi_keyboard: MidiKeyboard
}

impl State {
    pub fn new() -> State {
        State {
            sample_rate: SampleRate(48000),
            sample_clock: 0,
            midi_keyboard: MidiKeyboard::new(),
        }
    }
}

pub static STATE: Lazy<Mutex<State>> = Lazy::new(|| Mutex::new(State::new()));

pub fn get_midi_keyboard() -> MidiKeyboard {
    STATE.lock().unwrap().midi_keyboard
}

pub fn set_midi_keyboard(new_midi_keyboard: MidiKeyboard) {
    STATE.lock().unwrap().midi_keyboard = new_midi_keyboard;
}

pub fn get_sample_rate() -> f32 {
    STATE.lock().unwrap().sample_rate.0 as f32
}

pub fn advance_sample_clock(sample_count: u64) {
    STATE.lock().unwrap().sample_clock = sample_count;
}

pub fn get_sample_clock() -> u64 {
    STATE.lock().unwrap().sample_clock
}

pub fn set_sample_rate(new_rate: SampleRate) {
    STATE.lock().unwrap().sample_rate = new_rate;
    set_effects_sample_rate(new_rate);
}
