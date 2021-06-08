use crate::gui::adsr::ADSRControls;
use crate::gui::events::OscillatorControlEvent;
use crate::gui::AudioWidget;
use effects::adsr::ADSR;
use tuix::*;
use itertools::Itertools;

const DUMMY_WIDGETS_LIST: [AudioWidget; 1] = [
    AudioWidget::Adsr,
    // AudioWidget::Lfo,
];

pub struct Oscillator {
    sample_label: String,

    id: usize,
}

impl Oscillator {
    pub fn new<T: Into<String>>(id: usize, label: T) -> Self {
        Oscillator {
            id,
            sample_label: label.into(),
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

        OscillatorControls::new(id, &self.sample_label).build(state, container, |builder| builder);

        let widget_rack = HBox::new().build(state, container, |builder| {
            builder.set_flex_direction(FlexDirection::Column)
            // .class("oscillator")
        });

        // Would normally flex-wrap here instead of doing this weird calculated row thing
        //  but it is not implemented yet in tuix.
        // let widget_rack_rows_count = DUMMY_WIDGETS_LIST.len() / 2;
        // let widget_rack_rows = [0..widget_rack_rows_count].iter().map(|_| {
        //     HBox::new().build(state, widget_rack, |builder| {
        //         // TODO: ideally make a css class for this
        //         builder.set_flex_direction(FlexDirection::Column)
        //                 .set_justify_content(JustifyContent::SpaceEvenly)
        //                 .set_min_width(Units::Pixels(200.)) // TODO
        //     })
        // }).collect_vec();

        for widget in std::array::IntoIter::new(DUMMY_WIDGETS_LIST) {
            match widget {
                AudioWidget::Adsr => {
                    ADSRControls::new(id, adsr).build(state, widget_rack, |builder| builder);
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

    // components
    pub gain_knob: Entity,
    pub frequency_knob: Entity,
    pub active_toggle: Entity,
}

impl OscillatorControls {
    pub fn new<T: Into<String>>(id: usize, label: T) -> Self {
        OscillatorControls {
            id,
            sample_label: label.into(),
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
                .set_height(Units::Pixels(150.))
                .set_width(Units::Pixels(200.))
                .set_margin_bottom(Units::Pixels(5.0))
        });
        let row2 = HBox::new().build(state, container, |builder| {
            builder
                .set_justify_content(JustifyContent::SpaceEvenly)
                .set_flex_grow(1.)
                .set_border_width(Units::Pixels(2.))
                .set_border_color(Color::black())
        });

        Label::new(&self.sample_label).build(state, row, |builder| {
            builder
                .set_text_justify(Justify::Center)
                .set_width(Units::Pixels(50.0))
        });

        self.gain_knob = ValueKnob::new("Gain", 1.0, 0.0, 1.0)
            .on_change(move |val| Event::new(OscillatorControlEvent::GainChange(id, val)))
            .build(state, row2, |builder| {
                builder.set_width(Units::Pixels(50.0))
            });

        self.frequency_knob = ValueKnob::new("Frequency", 440.0, 0.0, 6000.0) // TODO: supply with actual value osc is initialized with
            .on_change(move |val| {
                Event::new(OscillatorControlEvent::FreqChange(id, val)).direct(entity)
            }) // TODO: We can set propagation mode too, dont know yet if I wanna do it
            .build(state, row2, |builder| {
                builder.set_width(Units::Pixels(50.0))
            });

        entity
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {}
}
