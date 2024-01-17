use godot::engine::{IPath2D, Path2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, base=Path2D)]
pub struct Rail2D {
    #[base]
    base: Base<Path2D>,

    #[var]
    #[export]
    north_link: Option<Gd<Rail2D>>,

    #[var]
    #[export]
    south_link: Option<Gd<Rail2D>>,
}

#[godot_api]
impl Rail2D {
    fn is_curve_valid(&self) -> bool {
        self.base()
            .get_curve()
            .map_or(false, |curve| curve.get_point_count() > 1)
    }

    fn check_for_self_in_neighbor_link(&self, neighbor: &Gd<Rail2D>) -> bool {
        let self_gd = &self.to_gd().to_variant();
        let nl = neighbor.callable("get_north_link").callv(varray![]);
        let sl = neighbor.callable("get_south_link").callv(varray![]);

        let mut result = false;
        if !nl.is_nil() && nl.eq(self_gd) {
            result = true;
        }

        if !sl.is_nil() && sl.eq(self_gd) {
            result = true;
        }
        result
    }

    fn is_mutually_connected(&self) -> bool {
        let mut result = true;

        if let Some(nl) = &self.north_link {
            result = self.check_for_self_in_neighbor_link(nl);
        }

        if let Some(sl) = &self.south_link {
            result = self.check_for_self_in_neighbor_link(sl);
        }

        result
    }
}

#[godot_api]
impl IPath2D for Rail2D {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            north_link: None,
            south_link: None,
        }
    }

    #[allow(clippy::needless_return)]
    fn get_configuration_warnings(&self) -> PackedStringArray {
        let mut errs = PackedStringArray::new();

        if !self.is_curve_valid() {
            errs.push(GString::from(
                "A curve needs to be set with at least 2 points to be useful",
            ));
        }

        if !self.is_mutually_connected() {
            errs.push(GString::from(
                "Linked neighbours also need to link back to this node",
            ));
        }

        return errs;
    }
}
