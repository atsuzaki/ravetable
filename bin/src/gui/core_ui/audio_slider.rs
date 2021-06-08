use tuix::*;

pub struct AudioSlider {
    label: String,

    value: f32,
    min: f32,
    max: f32,

    // Components
    slider: Entity,
    textbox: Entity,
}

impl AudioSlider {
    pub fn new<T: Into<String>>(label: T, min: f32, max: f32, starting_value: f32) -> Self {
        AudioSlider {
            label: label.into(),
            value: starting_value,
            min,
            max,
            slider: Entity::null(),
            textbox: Entity::null(),
        }
    }
}

impl Widget for AudioSlider {
    type Ret = Entity;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        // TODO: everything is slightly misaligned since passing align-items to
        //       audio_slider will cause the label and textbox to disappear.
        //       Suspecting that's bc element height was not explicitly specified?
        //       Find out what the heck happened there and report to geom
        let container = HBox::new().build(state, entity, |builder| {
            builder.class("audio_slider")
        });

        Label::new(&self.label).build(state, container, |builder| {
            builder
                .set_text_justify(Justify::Center)
                .set_width(Units::Pixels(50.0))
        });

        let slider_container = HBox::new().build(state, container, |builder| {
            builder.class("audio_slider_container")
        });
        self.slider = Slider::new()
            .with_min(self.min)
            .with_max(self.max)
            .with_initial_value(self.value / self.max) // Initial value expects a percentage
            .build(state, slider_container, |builder| builder);

        // TODO: Textbox doesn't update slider right now, I can't even set it to not focusable to
        //       disable it since it inteferes with the slider for some reason.
        //       Trying to handle the events for textbox runs into fun weird mess,
        //       I think it's being worked on, might be working in experiment branch?
        self.textbox = Textbox::new(&self.value.to_string()).build(state, container, |builder| {
            builder
                .set_width(Units::Pixels(50.0))
                .set_margin_left(Units::Pixels(8.))
                .set_margin_right(Units::Pixels(2.5))
        });

        entity
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(ev) = event.message.downcast::<SliderEvent>() {
            match ev {
                SliderEvent::ValueChanged(v) => {
                    if event.target == self.slider {
                        self.value = *v;
                        state.insert_event(
                            Event::new(TextboxEvent::SetValue(v.round().to_string()))
                                .target(self.textbox)
                                .propagate(Propagation::Direct),
                        );
                    }
                },
                _ => {},
            }
        }
    }
}
