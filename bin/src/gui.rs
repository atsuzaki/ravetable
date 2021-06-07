use crate::keyboard::{keyboard_to_midi, MidiNote};
use crate::state::{get_midi_keyboard, set_midi_keyboard};
use crate::EffectsEvent::IIRFreqChange;
use crate::Message;
use crate::Message::EffectsEvent;
use tuix::Application;
use tuix::*;

static THEME: &'static str = include_str!("bbytheme.css");

pub struct Controller {
    command_sender: crossbeam_channel::Sender<Message>,
    amplitude_knob: Entity,
    frequency_knob: Entity,
    active_toggle: Entity,

    currently_pressed_keys: Vec<Code>,
}

impl Controller {
    pub fn new(command_sender: crossbeam_channel::Sender<Message>) -> Self {
        // let mut oscillators = [OscillatorControl::default(); 3];

        Controller {
            command_sender,
            amplitude_knob: Entity::null(),
            frequency_knob: Entity::null(),
            active_toggle: Entity::null(),
            currently_pressed_keys: vec![],
        }
    }
}

//TODO osc freq defaults for 2 and 3 with 440 at 1 is 523.25 and 659.25

impl Widget for Controller {
    type Ret = Entity;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let root = HBox::new().build(state, entity, |builder| {
            builder
                .set_flex_direction(FlexDirection::Column)
                .set_padding(Units::Pixels(2.))
                .set_margin(Units::Pixels(4.))
                .set_border_width(Units::Pixels(2.))
                .set_border_color(Color::black())
        });
        let row = HBox::new().build(state, root, |builder| {
            builder
                .set_justify_content(JustifyContent::SpaceEvenly)
                .set_margin_bottom(Units::Pixels(5.0))
        });
        let row2 = HBox::new().build(state, root, |builder| {
            builder
                .set_justify_content(JustifyContent::SpaceEvenly)
                .set_margin_bottom(Units::Pixels(5.0))
        });

        self.amplitude_knob =
            ValueKnob::new("Amplitude", 1.0, 0.0, 1.0)
                .build(state, row, |builder| builder.set_width(Units::Pixels(50.0)));

        self.frequency_knob =
            ValueKnob::new("Frequency", 440.0, 0.0, 6000.0)
                .build(state, row, |builder| builder.set_width(Units::Pixels(50.0)));

        println!("Set focused in on_build to {}", state.focused);
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

        if let Some(slider_event) = event.message.downcast::<SliderEvent>() {
            match slider_event {
                SliderEvent::ValueChanged(val) => {
                    if event.target == self.amplitude_knob {
                        self.command_sender.send(Message::Amplitude(*val)).unwrap();
                    }

                    if event.target == self.frequency_knob {
                        self.command_sender
                            .send(Message::EffectsEvent(0, IIRFreqChange(*val)))
                            .unwrap(); // TODO: currently hardcoded
                    }
                }

                _ => {}
            }
        }
    }
}

struct OscillatorControl {
    amplitude_knob: Entity,
    frequency_knob: Entity,
    active_toggle: Entity,
}

/*// impl Default for OscillatorControl {
//     fn default() -> Self {
//         OscillatorControl {
//             amplitude_knob: Entity::null(),
//             frequency_knob: Entity::null(),
//             active_toggle: Entity::null(),
//         }
//     }
// }
//
// impl Widget for OscillatorControl {
//     type Ret = Entity;
//
//     fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
//         let root= HBox::new().build(state, entity, |builder| {
//             builder.set_flex_direction(FlexDirection::Column).set_padding(Units::Pixels(2.)).set_margin(Units::Pixels(4.))
//                 .set_border_width(Units::Pixels(2.)).set_border_color(Color::black())
//         });
//         let row = HBox::new().build(state, root, |builder| {
//             builder.set_justify_content(JustifyContent::SpaceEvenly).set_margin_bottom(Units::Pixels(5.0))
//         });
//         let row2 = HBox::new().build(state, root, |builder| {
//             builder.set_justify_content(JustifyContent::SpaceEvenly).set_margin_bottom(Units::Pixels(5.0))
//         });
//     }
// }*/
