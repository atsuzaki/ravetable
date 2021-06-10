use tuix::*;

use crate::gui::filter::ModulatedFilterControls;
use crate::messages::OscParams;
use crate::messages::OscParams::Gain;
use crate::{
    gui::adsr::ADSRControls,
    gui::events::SynthControlEvent,
    synths::{OscStatePacket, Sample},
};
use effects::EffectStatePacket;

pub struct Oscillator {
    id: usize,
    osc_state: OscStatePacket,
    available_samples: Vec<Sample>,
}

impl Oscillator {
    pub fn new(id: usize, osc_state: OscStatePacket, available_samples: Vec<Sample>) -> Self {
        Oscillator {
            id,
            osc_state,
            available_samples,
        }
    }
}

impl Widget for Oscillator {
    type Ret = Entity;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let id = self.id;

        let container = HBox::new().build(state, entity, |builder| {
            builder
                .set_flex_direction(FlexDirection::Row)
                .class("oscillator")
        });

        // TODO: these available_samples clonings are severely bothering me, borrow instead later
        OscillatorControls::new(
            id,
            &self.osc_state.name,
            self.osc_state.gain,
            self.available_samples.clone(),
        )
        .build(state, container, |builder| builder);

        let widget_rack = HBox::new().build(state, container, |builder| {
            builder.set_flex_direction(FlexDirection::Row)
            // .class("oscillator")
        });

        ADSRControls::new(id, self.osc_state.adsr).build(state, widget_rack, |builder| builder);

        // Skip 1 bc we're not showing the internal low pass filter (yet. TODO)
        for (effect_id, effect) in self.osc_state.effects.iter().skip(1).enumerate() {
            match effect {
                EffectStatePacket::ModulatedFilter(e) => {
                    let effect_id = effect_id + 1; // Plus one since we skipped the first el
                    ModulatedFilterControls::new(id, effect_id, e.clone()).build(
                        state,
                        widget_rack,
                        |builder| builder,
                    );
                }
                EffectStatePacket::IIRFilter(_) => {}
                EffectStatePacket::StateVariablePTPFilter(_) => {}
            }
        }

        entity
    }
}

pub struct OscillatorControls {
    id: usize,

    // data
    sample_label: String,
    available_samples: Vec<Sample>,
    gain: f32,

    // components
    pub dropdown: Entity,
    pub label: Entity,
    pub gain_knob: Entity,
    pub frequency_knob: Entity,
    pub active_toggle: Entity,
}

impl OscillatorControls {
    pub fn new<T: Into<String>>(
        id: usize,
        label: T,
        gain: f32,
        available_samples: Vec<Sample>,
    ) -> Self {
        OscillatorControls {
            id,
            sample_label: label.into(),
            available_samples,
            gain,
            label: Entity::null(),
            dropdown: Entity::null(),
            gain_knob: Entity::null(),
            frequency_knob: Entity::null(),
            active_toggle: Entity::null(),
        }
    }
}

impl Widget for OscillatorControls {
    type Ret = Entity;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let id = self.id;

        let container = HBox::new().build(state, entity, |builder| {
            builder
                .set_flex_direction(FlexDirection::Column)
                .class("widget_container")
                .set_height(Units::Pixels(300.))
        });

        let row = HBox::new().build(state, container, |builder| {
            builder
                .set_justify_content(JustifyContent::SpaceEvenly)
                .set_height(Units::Pixels(50.))
                .set_width(Units::Pixels(200.))
                .set_margin_bottom(Units::Pixels(5.0))
        });
        let row1 = HBox::new().build(state, container, |builder| {
            builder
                .set_justify_content(JustifyContent::SpaceEvenly)
                .set_height(Units::Pixels(100.))
                .set_width(Units::Pixels(200.))
        });
        let row2 = HBox::new().build(state, container, |builder| {
            builder
                .set_justify_content(JustifyContent::SpaceEvenly)
                .set_flex_grow(1.)
                .set_border_width(Units::Pixels(2.))
                .set_border_color(Color::black())
        });

        let (_, _, dropdown) = Dropdown::new(&self.sample_label).build(state, row, |b| {
            b.set_height(Units::Pixels(30.0))
                .set_width(Units::Pixels(175.))
        });
        let options = List::new().build(state, dropdown, |b| b);
        // let options = RadioList::new().build(state, dropdown, |b| b);
        self.available_samples
            .iter()
            .enumerate()
            .for_each(|(idx, sample)| {
                CheckButton::new(false)
                    .on_checked(Event::new(SynthControlEvent::OscillatorControl(
                        id,
                        OscParams::SampleChange(self.available_samples[idx].clone()),
                    )))
                    .build(state, options, |b| {
                        b.set_text(&sample.name)
                            .set_color(Color::blue()) // TODO: these needs color? or dropdown needs to be a darker color really
                            .set_height(Pixels(30.0))
                            .set_width(Units::Pixels(175.))
                            .set_margin_left(Pixels(5.0))
                    });
            });

        self.dropdown = dropdown;

        self.label = Label::new(&self.sample_label).build(state, row1, |builder| {
            builder
                .set_text_justify(Justify::Center)
                .set_width(Units::Pixels(50.0))
        });

        self.gain_knob = ValueKnob::new("Gain", self.gain, 0.0, 1.0)
            .on_change(move |val| Event::new(SynthControlEvent::OscillatorControl(id, Gain(val))))
            .build(state, row2, |builder| {
                builder.set_width(Units::Pixels(50.0))
            });

        self.frequency_knob = ValueKnob::new("Frequency", 440.0, 0.0, 6000.0) // TODO: supply with actual value osc is initialized with
            // .on_change(move |val| {
            // })
            .build(state, row2, |builder| {
                builder.set_width(Units::Pixels(50.0))
            });

        entity
    }

    fn on_event(&mut self, state: &mut State, _entity: Entity, event: &mut Event) {
        if let Some(SynthControlEvent::OscillatorControl(idx, param)) =
            event.message.downcast::<SynthControlEvent>()
        {
            if self.id == *idx {
                match param {
                    OscParams::SampleChange(sample) => {
                        let label = &sample.name;

                        self.sample_label = label.to_string();
                        self.label.set_text(state, label);
                        state.insert_event(
                            Event::new(DropdownEvent::SetText(label.clone()))
                                .target(self.dropdown)
                                .propagate(Propagation::Up),
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}
