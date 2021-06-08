mod oscillator;
mod events;
mod filter;
mod adsr;
mod core_ui;

use tuix::Application;
use tuix::*;

use crate::{
    EffectsEvent::IIRFreqChange,
    state::{get_midi_keyboard, set_midi_keyboard},
    keyboard::{keyboard_to_midi, MidiNote},
    Message,
    Message::EffectsEvent,
    gui::oscillator::Oscillator,
    gui::adsr::ADSRControls,
};

static THEME: &'static str = include_str!("../bbytheme.css");

pub enum AudioWidget {
    Adsr,
    Lfo,
    IIRFilter,
}

pub struct Controller {
    command_sender: crossbeam_channel::Sender<Message>,

    oscillators: Vec<Entity>,
    currently_pressed_keys: Vec<Code>,
}

impl Controller {
    pub fn new(command_sender: crossbeam_channel::Sender<Message>) -> Self {
        Controller {
            command_sender,
            oscillators: vec![],
            currently_pressed_keys: vec![],
        }
    }
}

impl Widget for Controller {
    type Ret = Entity;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let root = HBox::new().build(state, entity, |builder| {
            builder
                .set_margin(Units::Pixels(4.))
        });

        let osc = Oscillator::new(0, "sample label").build(state, root, |builder| builder);
        self.oscillators.push(osc);

        // println!("Set focused in on_build to {}", state.focused);
        entity
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(window_event) = event.message.downcast::<WindowEvent>() {
            match window_event {
                WindowEvent::KeyDown(code, _) => {
                     if !self.currently_pressed_keys.contains(code) {
                        if let Some(midi_note) = keyboard_to_midi(*code) {
                            println!("first time midi pressed: {:?}", midi_note);
                            let frq = get_midi_keyboard().get_frequency_from_key(&midi_note);
                            self.command_sender.send(Message::Note(1.0)).unwrap();
                            self.command_sender.send(Message::Frequency(frq)).unwrap();
                            self.currently_pressed_keys.push(*code);
                        }
                    }
                }
                WindowEvent::KeyUp(code, _) => {
                    if *code == Code::KeyZ {
                        println!("Z up");
                        let new_midi_keyboard = get_midi_keyboard().decrease_octave();
                        set_midi_keyboard(new_midi_keyboard);
                    } else if *code == Code::KeyX {
                        println!("X up");
                        let new_midi_keyboard = get_midi_keyboard().increase_octave();
                        set_midi_keyboard(new_midi_keyboard);
                    } else if self.currently_pressed_keys.contains(code) {
                        if let Some(_) = keyboard_to_midi(*code) {
                            self.command_sender.send(Message::Note(0.0)).unwrap();

                            let index = self
                                .currently_pressed_keys
                                .iter()
                                .position(|x| *x == *code)
                                .unwrap();
                            self.currently_pressed_keys.remove(index);
                        }
                    }
                }
                _ => {}
            }
        }

        // if let Some(slider_event) = event.message.downcast::<SliderEvent>() {
        //     match slider_event {
        //         SliderEvent::ValueChanged(val) => {
        //             if event.target == self.oscillators[0].gain_knob {
        //                 self.command_sender.send(Message::Amplitude(*val)).unwrap();
        //             }
        //
        //             if event.target == self.oscillators[0].frequency_knob {
        //                 self.command_sender
        //                     .send(Message::EffectsEvent(0, IIRFreqChange(*val)))
        //                     .unwrap(); // TODO: currently hardcoded
        //             }
        //         }
        //
        //         _ => {}
        //     }
        // }
    }
}
