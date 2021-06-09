use tuix::*;

use crate::gui::core_ui::audio_slider::AudioSlider;
use crate::gui::core_ui::audio_widget::AudioWidgetContainer;
use crate::gui::core_ui::hdivider::HDivider;
use crate::gui::events::OscillatorControlEvent;
use effects::adsr::ADSR;

pub struct ADSRControls {
    osc_id: usize,

    adsr: ADSR,

    // components
}

impl ADSRControls {
    pub fn new(osc_id: usize, adsr: ADSR) -> Self {
        let adsr = ADSR::convert_adsr_from_sample_clock(&adsr);
        ADSRControls { osc_id, adsr }
    }
}

impl Widget for ADSRControls {
    type Ret = Entity;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let id = self.osc_id;

        let container =
            AudioWidgetContainer::new("ADSR Envelope").build(state, entity, |builder| builder);

        let row = HBox::new().build(state, container, |builder| {
            builder
                .set_justify_content(JustifyContent::SpaceEvenly)
                .set_height(Units::Pixels(150.))
                .set_flex_direction(FlexDirection::Column)
        });

        let slider_max = 32.;
        AudioSlider::new("Attack", 0., slider_max, self.adsr.attack)
            .build(state, row, |builder| builder);
        AudioSlider::new("Decay", 0., slider_max, self.adsr.decay).build(state, row, |builder| builder);
        AudioSlider::new("Sustain", 0., slider_max, self.adsr.sustain)
            .build(state, row, |builder| builder);
        AudioSlider::new("Release", 0., slider_max, self.adsr.release)
            .build(state, row, |builder| builder);

        HDivider::new().build(state, row, |builder| builder);

        AudioSlider::new("Delay", 0., slider_max, self.adsr.delay).build(state, row, |builder| builder);

        entity
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {}
}
