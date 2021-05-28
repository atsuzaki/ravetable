use crate::Message;
use tuix::Application;
use tuix::*;

static THEME: &'static str = include_str!("bbytheme.css");

struct Controller {
    command_sender: crossbeam_channel::Sender<Message>,
    // oscillators: [OscillatorControl; 3],
    amplitude_knob: Entity,
    frequency_knob: Entity,
    active_toggle: Entity,
}

impl Controller {
    pub fn new(command_sender: crossbeam_channel::Sender<Message>) -> Self {
        // let mut oscillators = [OscillatorControl::default(); 3];

        Controller {
            command_sender,
            amplitude_knob: Entity::null(),
            frequency_knob: Entity::null(),
            active_toggle: Entity::null(),
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
                    if *code == Code::KeyZ || *code == Code::Digit5 {
                        println!("Z pressed");
                        self.command_sender.send(Message::Note(1.0)).unwrap();
                    }
                }
                WindowEvent::KeyUp(code, _) => {
                    if *code == Code::KeyZ || *code == Code::Digit5 {
                        println!("Z up");
                        self.command_sender.send(Message::Note(0.0)).unwrap();
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
                        self.command_sender.send(Message::Frequency(*val)).unwrap();
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
