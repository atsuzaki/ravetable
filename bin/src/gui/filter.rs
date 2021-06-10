use tuix::*;

use effects::filters::ModulatedFilterStatePacket;

use crate::{
    gui::{
        core_ui::audio_slider::AudioSlider, core_ui::audio_widget::AudioWidgetContainer,
        events::SynthControlEvent,
    },
    messages::ModulatedFilterParams,
    messages::StateVarTPTFilterParams,
};
use effects::filters::FilterType;

pub struct ModulatedFilterControls {
    osc_id: usize,
    effect_id: usize,

    filter: ModulatedFilterStatePacket,
}

impl ModulatedFilterControls {
    pub fn new(osc_id: usize, effect_id: usize, filter: ModulatedFilterStatePacket) -> Self {
        ModulatedFilterControls {
            osc_id,
            effect_id,
            filter,
        }
    }
}
// TODO: Pretty much hardcoded for statevartpt filter right now. IIRFilter can't be used for this
//       as it's very sensitive to modulation, making it sound extremely crunchy.
//       Generalize this in the future
impl Widget for ModulatedFilterControls {
    type Ret = Entity;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let id = self.osc_id;
        let effect_id = self.effect_id;

        let container = AudioWidgetContainer::new("Filter").build(state, entity, |builder| builder);

        let row = HBox::new().build(state, container, |builder| {
            builder
                // Styling somehow got super cursed and added a huge gap between label and first row,
                //   using good ol' negative margin to try to line it up
                .set_margin_top(Units::Pixels(-28.))
                .set_justify_content(JustifyContent::SpaceBetween)
                .set_max_height(Units::Pixels(120.))
                .set_flex_direction(FlexDirection::Column)
        });

        // TODO: should be dropdown
        AudioSlider::new("Type", 0., 15_000., self.filter.base_frequency)
            .on_change(move |val| {
                Event::new(SynthControlEvent::ModulatedFilter(
                    id,
                    effect_id,
                    ModulatedFilterParams::Filter(StateVarTPTFilterParams::FilterType(
                        FilterType::HighPass,
                    )),
                ))
            })
            .build(state, row, |builder| builder);

        AudioSlider::new("Frq", 0., 15_000., self.filter.base_frequency)
            .on_change(move |val| {
                use crate::messages::StateVarTPTFilterParams::*;
                Event::new(SynthControlEvent::ModulatedFilter(
                    id,
                    effect_id,
                    ModulatedFilterParams::Filter(Frequency(val)),
                ))
            })
            .build(state, row, |builder| builder);

        AudioSlider::new("Reso", 0., 1., self.filter.filter.resonance)
            .on_change(move |val| {
                use crate::messages::StateVarTPTFilterParams::*;
                Event::new(SynthControlEvent::ModulatedFilter(
                    id,
                    effect_id,
                    ModulatedFilterParams::Filter(Resonance(val)),
                ))
            })
            .build(state, row, |builder| builder);

        let row2 = HBox::new().build(state, container, |builder| {
            builder.set_height(Units::Pixels(40.))
        });

        Label::new("Frequency Modulation").build(state, row2, |builder| {
            builder
                .set_text_align(Align::Center)
                .set_text_justify(Justify::Center)
                .set_flex_grow(1.)
                .set_min_width(Units::Pixels(50.))
        });

        let row3 = HBox::new().build(state, container, |builder| {
            builder
                .set_justify_content(JustifyContent::SpaceEvenly)
                .set_flex_direction(FlexDirection::Row)
                .set_max_height(Units::Pixels(0.))
                .set_height(Units::Pixels(0.))
        });

        ValueKnob::new("Frq", self.filter.lfo.frequency, 0.0, 10.0)
            .on_change(move |val| {
                use crate::messages::LfoParams::*;
                Event::new(SynthControlEvent::ModulatedFilter(
                    id,
                    effect_id,
                    ModulatedFilterParams::Lfo(Frequency(val)),
                ))
            })
            .build(state, row3, |builder| {
                builder.set_width(Units::Pixels(50.0))
            });

        ValueKnob::new("Phase", self.filter.lfo.phase, 0.0, 10.0)
            .on_change(move |val| {
                use crate::messages::LfoParams::*;
                Event::new(SynthControlEvent::ModulatedFilter(
                    id,
                    effect_id,
                    ModulatedFilterParams::Lfo(Phase(val)),
                ))
            })
            .build(state, row3, |builder| {
                builder.set_width(Units::Pixels(50.0))
            });

        entity
    }
}
