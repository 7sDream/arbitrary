use eframe::egui::{Id, Ui, Window};

pub trait WindowState: Default + Clone + PartialEq + Send + Sync + 'static {
    fn open_mut(&mut self) -> &mut bool;

    fn get(ui: &mut Ui, id: Id) -> Self {
        ui.memory_mut(|mem| mem.data.get_temp_mut_or_default::<Self>(id).clone())
    }
}

pub trait FloatWindow {
    type Data;
    type State: WindowState;

    fn new(ui: &mut Ui, id: Id) -> Self;
    fn id(&self) -> Id;
    fn state_mut(&mut self) -> &mut Self::State;
    fn into_state(self) -> Self::State;

    fn title(&self) -> &'static str;

    fn window_controls(&mut self, ui: &mut Ui, data: &mut Self::Data);

    fn open_mut(&mut self) -> &mut bool {
        self.state_mut().open_mut()
    }

    fn open(ui: &mut Ui, id: Id)
    where
        Self: Sized,
    {
        Self::new(ui, id).update(ui, |_, s| *s.open_mut() = true);
    }

    fn close(ui: &mut Ui, id: Id)
    where
        Self: Sized,
    {
        Self::new(ui, id).update(ui, |_, s| *s.open_mut() = false);
    }

    fn update<F, T>(mut self, ui: &mut Ui, f: F) -> T
    where
        F: FnOnce(&mut Ui, &mut Self) -> T,
        Self: Sized,
    {
        let origin = self.state_mut().clone();
        let result = f(ui, &mut self);
        if self.state_mut() != &origin {
            ui.memory_mut(|mem| mem.data.insert_temp(self.id(), self.into_state()))
        }
        result
    }

    fn show(mut self, ui: &mut Ui, data: &mut Self::Data)
    where
        Self: Sized,
    {
        if !*self.state_mut().open_mut() {
            return;
        }

        let window_id = self.id().with("window");
        let title = self.title();

        self.update(ui, |ui, myself| {
            let mut opened = true;

            Window::new(title)
                .id(window_id)
                .open(&mut opened)
                .default_open(true)
                .default_width(160.0)
                .scroll2([false, true])
                .max_height(f32::MAX)
                .show(ui.ctx(), |ui| {
                    myself.window_controls(ui, data);
                    ui.allocate_space((0.0, ui.available_height()).into());
                });

            if !opened {
                *myself.state_mut().open_mut() = false;
            }
        });
    }
}

macro_rules! impl_window {
    (
        $window:ident<$data:ty> as $title:literal : $state:ident { $($field:ident : $field_type:ty = $field_default:expr ,)*}
    ) => {
        #[derive(Clone, PartialEq)]
        pub struct $state {
            opened: bool,
            $($field : $field_type ,)*
        }

        impl Default for $state {
            fn default() -> Self {
                Self {
                    opened: false,
                    $($field: $field_default ,)*
                }
            }
        }

        impl $crate::window::WindowState for $state {
            fn open_mut(&mut self) -> &mut bool {
                &mut self.opened
            }
        }

        pub struct $window {
            id: ::eframe::egui::Id,
            state: $state,
        }

        impl $crate::window::FloatWindow for $window {
            type Data  = $data;
            type State = $state;

            fn new(ui: &mut ::eframe::egui::Ui, id: ::eframe::egui::Id) -> Self {
                Self {
                    id,
                    state: <$state as $crate::window::WindowState>::get(ui, id),
                }
            }

            fn id(&self) -> ::eframe::egui::Id {
                self.id
            }

            fn state_mut(&mut self) -> &mut Self::State {
                &mut self.state
            }

            fn into_state(self) -> Self::State {
                self.state
            }

            fn title(&self) -> &'static str {
                $title
            }

            fn window_controls(&mut self, ui: &mut ::eframe::egui::Ui, data: &mut Self::Data) {
                self.controls(ui, data);
            }
        }
    };
}

mod configure;
mod shape;

pub use self::{configure::ConfigureWindow, shape::ShapeDataWindow};
