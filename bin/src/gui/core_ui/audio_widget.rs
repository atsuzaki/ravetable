use crate::gui::core_ui::hdivider::HDivider;
use tuix::*;

pub struct AudioWidgetContainer {
    label: String,
}

impl AudioWidgetContainer {
    pub fn new<T: Into<String>>(label: T) -> AudioWidgetContainer {
        AudioWidgetContainer {
            label: label.into(),
        }
    }
}

impl Widget for AudioWidgetContainer {
    type Ret = Entity;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity
            .set_flex_direction(state, FlexDirection::Column)
            .class(state, "widget_container");

        Label::new(&self.label).build(state, entity, |builder| {
            builder
                .set_text_align(Align::Center)
                .set_text_justify(Justify::Center)
                .set_flex_grow(1.)
                .set_min_width(Units::Pixels(50.))
                .set_height(Units::Pixels(40.))
                .set_max_height(Units::Pixels(40.))
        });

        HDivider::new()
            .with_height(20.)
            .build(state, entity, |builder| builder);

        entity
    }
}
