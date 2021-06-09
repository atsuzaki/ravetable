mod adsr;
mod core_ui;
mod events;
mod filter;
mod oscillator;

use log::info;
use tuix::Application;
use tuix::*;

use crate::gui::events::OscillatorControlEvent;
use crate::synths::Sample;
use crate::{
    gui::adsr::ADSRControls,
    gui::oscillator::Oscillator,
    keyboard::{keyboard_to_midi, MidiNote},
    mixer::MixerStatePacket,
    state::get_sample_clock,
    state::{get_midi_keyboard, set_midi_keyboard},
    EffectsEvent::IIRFreqChange,
    Message,
    Message::EffectsEvent,
};

static THEME: &'static str = include_str!("../bbytheme.css");

pub enum AudioWidget {
    Adsr,
    Lfo,
    IIRFilter,
}

pub struct Controller {
    command_sender: crossbeam_channel::Sender<Message>,
    command_receiver: crossbeam_channel::Receiver<Message>,

    mixer_state_packet: MixerStatePacket,

    oscillators: Vec<Entity>,
    currently_pressed_keys: Vec<Code>,
    time_since_last_octave_change: u64,

    available_samples: Vec<Sample>,
}

impl Controller {
    pub fn new(
        command_sender: crossbeam_channel::Sender<Message>,
        command_receiver: crossbeam_channel::Receiver<Message>,
        mixer_state_packet: MixerStatePacket,
        available_samples: Vec<Sample>,
    ) -> Self {
        Controller {
            command_sender,
            command_receiver,
            mixer_state_packet,
            oscillators: vec![],
            currently_pressed_keys: vec![],
            time_since_last_octave_change: 0,
            available_samples,
        }
    }
}

impl Widget for Controller {
    type Ret = Entity;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        state.focused = entity;

        let root = HBox::new().build(state, entity, |builder| {
            builder
                .set_margin(Units::Pixels(4.))
                .set_flex_direction(FlexDirection::Row)
        });

        for (i, oscillator) in self.mixer_state_packet.oscillators.iter().enumerate() {
            let osc = Oscillator::new(i, oscillator.clone(), self.available_samples.clone()).build(
                state,
                root,
                |builder| builder,
            );

            self.oscillators.push(osc);
        }
        entity
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(window_event) = event.message.downcast::<WindowEvent>() {
            match window_event {
                WindowEvent::KeyDown(code, _) => {
                    if !self.currently_pressed_keys.contains(code) {
                        if let Some(midi_note) = keyboard_to_midi(*code) {
                            info!("first time midi pressed: {:?}", midi_note);
                            let frq = get_midi_keyboard().get_frequency_from_key(&midi_note);
                            self.command_sender.send(Message::Note(1.0)).unwrap();
                            self.command_sender.send(Message::Frequency(frq)).unwrap();
                            self.currently_pressed_keys.push(*code);
                        }
                    }
                    event.consume();
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
                            let index = self
                                .currently_pressed_keys
                                .iter()
                                .position(|x| *x == *code)
                                .unwrap();
                            self.currently_pressed_keys.remove(index);

                            // If that keyup was the last key pressed, send a message for release
                            if self.currently_pressed_keys.len() == 0 {
                                self.command_sender.send(Message::Note(0.0)).unwrap();
                            }
                        }
                    }
                    event.consume();
                }
                _ => {}
            }
        }

        if let Some(ev) = event.message.downcast::<OscillatorControlEvent>() {
            match ev {
                OscillatorControlEvent::GainChange(id, val) => {
                    self.command_sender.send(Message::Gain(*id, *val)).unwrap();
                }
                OscillatorControlEvent::OscWavetableChange(id, sample_idx) => {
                    self.command_sender
                        .send(Message::OscWavetableChange(
                            *id,
                            self.available_samples[*sample_idx].clone(),
                        ))
                        .unwrap();
                }
                _ => {}
            }
        }
    }
}
