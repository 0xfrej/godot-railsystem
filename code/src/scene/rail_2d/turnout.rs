use crate::scene::rail_2d::rail::Rail2D;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, base=Node2D)]
pub struct Turnout2D {
    #[base]
    base: Base<Node2D>,

    #[export]
    #[var]
    current_point: u8,

    #[export]
    #[var(get, set = set_points)]
    points: Array<Gd<Rail2D>>,
}

#[godot_api]
impl Turnout2D {
    #[func]
    fn set_points(&mut self, points: Array<Gd<Rail2D>>) {
        self.points = points;

        if self.current_point > self.points.len() as u8 {
            godot_warn!(
                "Turnout2D: current point is outside of range of new point list, setting to 0"
            );
            self.current_point = 0;
        }
    }

    #[func]
    fn add_point(&mut self, rail: Gd<Rail2D>) -> u8 {
        let new_index = self.points.len();
        self.points.insert(new_index, rail);
        new_index as u8
    }

    #[func]
    fn remove_point(&mut self, index: u8) {
        self.points.remove(index as usize);

        if self.current_point <= index {
            godot_warn!(
                "Turnout2D: current point index was probably moved to a new spot, setting to 0"
            );
            self.current_point = 0;
        }
    }
}

#[godot_api]
impl INode2D for Turnout2D {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            current_point: 0,
            points: Array::new(),
        }
    }
}
