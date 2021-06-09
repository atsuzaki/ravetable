use crate::gui::adsr::ADSRControls;
use crate::gui::events::OscillatorControlEvent;
use crate::gui::AudioWidget;
use crate::synths::{OscStatePacket, Sample};
use effects::adsr::ADSR;
use itertools::Itertools;
use tuix::*;

const DUMMY_WIDGETS_LIST: [AudioWidget; 1] = [
    AudioWidget::Adsr,
    // AudioWidget::Lfo,
];

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

        // TODO: all these data to be replaced with real data
        let adsr = ADSR::default();

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
            builder.set_flex_direction(FlexDirection::Column)
            // .class("oscillator")
        });

        for widget in std::array::IntoIter::new(DUMMY_WIDGETS_LIST) {
            match widget {
                AudioWidget::Adsr => {
                    ADSRControls::new(id, self.osc_state.adsr).build(
                        state,
                        widget_rack,
                        |builder| builder,
                    );
                }
                AudioWidget::Lfo => {}
                AudioWidget::IIRFilter => {}
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

        // TODO: need a component or whatever
        let (_, _, dropdown) = Dropdown::new(&self.sample_label).build(state, row, |b| {
            b.set_height(Units::Pixels(30.0))
                .set_width(Units::Pixels(175.))
        });
        let options = List::new().build(state, dropdown, |b| b);
        // let options = RadioList::new().build(state, dropdown, |b| b);
        self.available_samples.iter().enumerate().for_each(|(idx, sample)| {
            CheckButton::new(false)
                .on_checked(
                    Event::new(OscillatorControlEvent::OscWavetableChange(id, idx))
                )
                .build(state, options, |b| {
                    b.set_text(&sample.name)
                        .set_color(Color::blue()) // TODO: these needs color? or dropdown needs to be a darker color really
                        .set_height(Pixels(30.0))
                        .set_width(Units::Pixels(175.))
                        .set_margin_left(Pixels(5.0))
                });
        });

        self.label = Label::new(&self.sample_label).build(state, row1, |builder| {
            builder
                .set_text_justify(Justify::Center)
                .set_width(Units::Pixels(50.0))
        });

        self.gain_knob = ValueKnob::new("Gain", self.gain, 0.0, 1.0)
            .on_change(move |val| Event::new(OscillatorControlEvent::GainChange(id, val)))
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

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(ev) = event.message.downcast::<OscillatorControlEvent>() {
            match ev {
                OscillatorControlEvent::OscWavetableChange(idx, sample_idx) => {
                    if self.id == *idx {
                        let label = &self.available_samples[*sample_idx].name;

                        self.sample_label = label.to_string();
                        self.label.set_text(state, label);
                    }
                },
                _ => {},
            }
        }
    }
}
