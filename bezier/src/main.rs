use eframe::{
    egui::{CentralPanel, Context},
    App, Frame,
};
use egui_plot::PlotPoint;

mod bezier;

struct Application {
    curve: bezier::Cubic,
}

impl App for Application {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Cubic Curve");
            let plot_id = ui.id().with("curve");

            let transform = egui_plot::Plot::new(plot_id)
                .data_aspect(1.0)
                .include_x(-40)
                .include_x(40)
                .include_y(-60)
                .include_y(20)
                .y_axis_width(3)
                .show(ui, |plot| {
                    for point in self.curve.points() {
                        plot.points(point);
                    }
                })
                .transform;

            self.curve.ui(plot_id, ui, transform);
        });
    }
}

impl Application {
    fn create(_ctx: &eframe::CreationContext<'_>) -> Box<dyn App + 'static> {
        Box::new(Self {
            curve: bezier::Cubic::new(
                PlotPoint { x: -40.0, y: 0.0 },
                PlotPoint { x: 40.0, y: 0.0 },
                PlotPoint { x: -20.0, y: -20.0 },
                PlotPoint { x: 20.0, y: -20.0 },
            ),
        })
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Bezier",
        eframe::NativeOptions::default(),
        Box::new(Application::create),
    )
}
