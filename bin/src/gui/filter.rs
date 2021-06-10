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

    // components
    dropdown: Entity,
}

const FILTER_TYPES: [FilterType; 3] = [
    FilterType::LowPass,
    FilterType::HighPass,
    FilterType::BandPass,
];

impl ModulatedFilterControls {
    pub fn new(osc_id: usize, effect_id: usize, filter: ModulatedFilterStatePacket) -> Self {
        ModulatedFilterControls {
            osc_id,
            effect_id,
            filter,
            dropdown: Entity::null(),
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
                .set_align_items(AlignItems::Center)
                .set_max_height(Units::Pixels(120.))
                .set_flex_direction(FlexDirection::Column)
        });

        let (_, _, dropdown) =
            Dropdown::new(&format!("{}", self.filter.filter.filter_type)).build(state, row, |b| {
                b.set_height(Units::Pixels(30.0))
                    .set_width(Units::Pixels(175.))
                    .set_margin_bottom(Units::Pixels(8.))
            });
        let options = List::new().build(state, dropdown, |b| b);

        FILTER_TYPES.iter().for_each(|filter_type| {
            CheckButton::new(false)
                .on_checked(Event::new(SynthControlEvent::ModulatedFilter(
                    id,
                    effect_id,
                    ModulatedFilterParams::Filter(StateVarTPTFilterParams::FilterType(
                        *filter_type,
                    )),
                )))
                .build(state, options, |b| {
                    b.set_text(&format!("{}", filter_type))
                        .set_color(Color::blue())
                        .set_height(Pixels(30.0))
                        .set_width(Units::Pixels(175.))
                        .set_margin_left(Pixels(5.0))
                });
        });

        self.dropdown = dropdown;

        AudioSlider::new("Frq", 0., 15_000., self.filter.base_frequency)
            .set_to_round_label(true)
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
                // Same here with negative margin
                .set_margin_top(Units::Pixels(-16.))
        });

        let row3 = HBox::new().build(state, container, |builder| {
            builder
                .set_justify_content(JustifyContent::SpaceEvenly)
                .set_flex_direction(FlexDirection::Row)
                .set_max_height(Units::Pixels(0.))
                .set_height(Units::Pixels(0.))
                // Same here with negative margin
                .set_margin_top(Units::Pixels(-4.))
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

    fn on_event(&mut self, state: &mut State, _entity: Entity, event: &mut Event) {
        if let Some(SynthControlEvent::ModulatedFilter(osc_id, effect_id, param)) =
            event.message.downcast::<SynthControlEvent>()
        {
            if self.osc_id == *osc_id && self.effect_id == *effect_id {
                match param {
                    ModulatedFilterParams::Filter(StateVarTPTFilterParams::FilterType(
                        filter_type,
                    )) => {
                        let label = format!("{}", filter_type);

                        state.insert_event(
                            Event::new(DropdownEvent::SetText(label.clone()))
                                .target(self.dropdown)
                                .propagate(Propagation::Up),
                        )
                    }
                    _ => {}
                }
            }
        }
    }
}
