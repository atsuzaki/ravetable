use tuix::*;

pub struct HDivider {
    height: Units
}

impl HDivider {
    pub fn new() -> HDivider {
        HDivider {
            height: Units::Pixels(50.),
        }
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Units::Pixels(height);
        self
    }
}

impl Widget for HDivider {
    type Ret = Entity;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        Element::new().build(state, entity, |builder| {
            builder
                .set_height(self.height)
        });

        entity
    }
}
