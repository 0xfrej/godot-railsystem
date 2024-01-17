use crate::scene::rail_2d::rail::Rail2D;
use godot::engine::node::InternalMode;
use godot::engine::{Curve2D, Engine, Timer};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, base=Node2D)]
pub struct RailFollower2D {
    #[base]
    base: Base<Node2D>,

    #[export]
    #[var(get, set = set_rail)]
    rail: Option<Gd<Rail2D>>,

    #[allow(dead_code)]
    update_timer: Option<Gd<Timer>>,

    #[export(range = (-10000.,10000.,0.01,or_less,or_greater))]
    #[var(get, set = set_progress)]
    progress: real,

    // TODO: rework when I figure out how to have virtual props
    #[export(range = (0.,1.1,0.0001,or_less,or_greater))]
    #[var(get = get_progress_ratio, usage_flags = [PROPERTY_USAGE_EDITOR])]
    #[allow(dead_code)]
    progress_ratio: real,

    #[export]
    #[var(get, set = set_h_offset)]
    h_offset: real,

    #[export]
    #[var(get, set = set_v_offset)]
    v_offset: real,

    #[export]
    #[var]
    use_cubic_interpolation: bool,

    #[export]
    #[var(get, set = set_rotation_enabled)]
    rotation_enabled: bool,
}

#[godot_api]
impl RailFollower2D {
    // fn path_changed(&mut self) {
    //todo: implement this -> has to be called when path changes within currently occupied rail
    /*
        if (update_timer && !update_timer->is_stopped()) {
        update_timer->start();
    } else {
        _update_transform();
    }
         */
    // }

    fn update_transform(&mut self) {
        if let Some(rail) = self.rail.as_ref() {
            if let Some(curve) = rail.get_curve() {
                if !curve.is_instance_valid() {
                    return;
                }

                let path_len = curve.get_baked_length();
                if path_len == 0. {
                    return;
                }

                let offset = Vector2::new(self.h_offset, self.v_offset);
                if self.rotation_enabled {
                    let xform = curve
                        .sample_baked_with_rotation_ex()
                        .offset(self.progress)
                        .cubic(self.use_cubic_interpolation)
                        .done()
                        .translated_local(offset);
                    let mut base = self.base_mut();
                    base.set_rotation(xform.a.angle());
                    base.set_position(xform.origin)
                } else {
                    let mut pos = curve
                        .sample_baked_ex()
                        .offset(self.progress)
                        .cubic(self.use_cubic_interpolation)
                        .done();
                    pos.x += offset.x;
                    pos.y += offset.y;
                    self.base_mut().set_position(pos)
                }
            }
        }
    }

    fn get_next_rail_in_direction(&self, progress: real) -> Option<Gd<Rail2D>> {
        if let Some(rail) = self.rail.as_ref() {
            if let Some(curve) = rail.get_curve() {
                let path_len = curve.get_baked_length();

                if progress > path_len {
                    let nl = rail.callable("get_north_link").callv(varray![]);
                    if !nl.is_nil() {
                        let r: Gd<Rail2D> = Gd::from_variant(&nl);
                        return Some(r);
                    }
                } else {
                    let nl = rail.callable("get_south_link").callv(varray![]);
                    if !nl.is_nil() {
                        let r: Gd<Rail2D> = Gd::from_variant(&nl);
                        return Some(r);
                    }
                }
            }
        }
        None
    }

    fn hop_to_next_rail_in_direction(&mut self, progress: real, current_path_len: real) -> bool {
        if let Some(next) = self.get_next_rail_in_direction(progress) {
            self.rail = Some(next);

            if progress < 0. {
                if let Some(curve) = self.rail.as_ref().unwrap().get_curve() {
                    let next_path_len = curve.get_baked_length();
                    self.set_progress(progress + next_path_len);
                } else {
                    godot_warn!(
                        "RailFollower2D: next rail had no valid curve, setting progress to 0"
                    );
                }
            } else {
                self.set_progress(progress - current_path_len);
            }

            return true;
        }
        false
    }

    #[func]
    fn set_rail(&mut self, rail: Option<Gd<Rail2D>>) {
        self.rail = rail;
        self.update_transform();
    }

    #[func]
    fn get_followed_curve(&self) -> Option<Gd<Curve2D>> {
        self.rail.as_ref().and_then(|rail| rail.get_curve())
    }

    #[func]
    fn advance_progress(&mut self, amount: real) {
        let current = self.get_progress();
        self.set_progress(current + amount);
    }

    #[func]
    fn can_progress(&self, progress: real) -> bool {
        self.get_followed_curve().map_or(false, |curve| {
            (progress >= 0.0 && progress <= curve.get_baked_length())
                || self.get_next_rail_in_direction(progress).is_some()
        })
    }

    #[func]
    fn can_advance_progress(&self, amount: real) -> bool {
        self.can_progress(self.progress + amount)
    }

    #[func]
    fn set_progress(&mut self, progress: real) {
        assert!(progress.is_finite(), "progress value is infinite");
        self.progress = progress;

        if let Some(curve) = self.get_followed_curve() {
            let path_len = curve.get_baked_length();

            // check if we are overshooting and try hop to another rail
            // clamp on failure
            if (self.progress < 0.0 || self.progress > path_len)
                && !self.hop_to_next_rail_in_direction(self.progress, path_len)
            {
                self.progress = self.progress.clamp(real!(0.0), path_len);
            }
            self.update_transform();
        }
    }

    #[func]
    fn get_progress_ratio(&self) -> real {
        let default_value = real!(0.);
        self.get_followed_curve().map_or(default_value, |c| {
            if c.is_instance_valid() && c.get_baked_length() > 0. {
                self.get_progress() / c.get_baked_length()
            } else {
                default_value
            }
        })
    }

    // #[func]
    // fn set_progress_ratio(&mut self, ratio: real) {
    // if let Some(length) = self
    //     .rail
    //     .as_ref()
    //     .and_then(|rail| rail.get_curve())
    //     .filter(|curve| curve.is_instance_valid() && curve.get_baked_length() > 0.)
    //     .map(|curve| curve.get_baked_length())
    // {
    //     self.set_progress(ratio * length);
    // }
    // }

    #[func]
    fn set_h_offset(&mut self, offset: real) {
        self.h_offset = offset;
        self.update_transform()
    }

    #[func]
    fn set_v_offset(&mut self, offset: real) {
        self.v_offset = offset;
        self.update_transform();
    }

    #[func]
    fn set_rotation_enabled(&mut self, enabled: bool) {
        self.rotation_enabled = enabled;
        self.update_transform();
    }
}

#[godot_api]
impl INode2D for RailFollower2D {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            rail: None,
            update_timer: None,
            progress: real!(0.),
            progress_ratio: real!(0.),
            h_offset: real!(0.),
            v_offset: real!(0.),
            use_cubic_interpolation: true,
            rotation_enabled: true,
        }
    }

    fn enter_tree(&mut self) {
        self.update_transform();
    }

    fn ready(&mut self) {
        if Engine::singleton().is_editor_hint() {
            let mut timer = Timer::new_alloc();
            timer.set_wait_time(0.2);
            timer.set_one_shot(true);
            timer.connect(
                StringName::from("timeout"),
                Callable::from_object_method(&self.to_gd(), "update_transform"),
            );
            self.base_mut()
                .add_child_ex(timer.upcast())
                .force_readable_name(false)
                .internal(InternalMode::INTERNAL_MODE_BACK)
                .done();
        }
    }

    #[allow(clippy::needless_return)]
    fn get_configuration_warnings(&self) -> PackedStringArray {
        let mut errs = PackedStringArray::new();

        if !self.get_rail().is_some_and(|rail| rail.is_instance_valid()) {
            errs.push(GString::from(
                "A RailFollower2D needs to point at a valid Rai2D instance",
            ));
        }

        return errs;
    }
}
