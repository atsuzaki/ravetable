use tuix::Code;

#[derive(Copy, Clone, Debug)]
pub enum MidiNote {
    C = 0,
    Csharp,
    D,
    Dsharp,
    E,
    F,
    Fsharp,
    G,
    Gsharp,
    A,
    Asharp,
    B,
}

#[derive(Debug, Copy, Clone)]
pub struct MidiKeyboard {
    octave: i16,
}

impl MidiKeyboard {
    pub fn new() -> Self {
        MidiKeyboard { octave: 3 }
    }

    pub fn increase_octave(mut self) -> MidiKeyboard {
        if self.octave < 9 {
            self.octave += 1;
        }
        self
    }

    pub fn decrease_octave(mut self) -> MidiKeyboard {
        if self.octave > -1 {
            self.octave -= 1;
        }
        self
    }

    pub fn get_frequency_from_key(&self, key_pressed: &MidiNote) -> f32 {
        let mut keypress = *key_pressed as i16;
        keypress += self.octave * 12;

        let freq = 440. * 2_f32.powf((keypress - 69) as f32 / 12.);
        freq
    }
}

pub fn keyboard_to_midi(keycode: Code) -> Option<MidiNote> {
    match keycode {
        Code::KeyA => Some(MidiNote::C),
        Code::KeyW => Some(MidiNote::Csharp),
        Code::KeyS => Some(MidiNote::D),
        Code::KeyE => Some(MidiNote::Dsharp),
        Code::KeyD => Some(MidiNote::E),
        Code::KeyR => Some(MidiNote::F),
        Code::KeyF => Some(MidiNote::Fsharp),
        Code::KeyH => Some(MidiNote::G),
        Code::KeyU => Some(MidiNote::Gsharp),
        Code::KeyJ => Some(MidiNote::A),
        Code::KeyI => Some(MidiNote::Asharp),
        Code::KeyK => Some(MidiNote::B),
        _ => None,
    }
}
