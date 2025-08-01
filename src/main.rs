use std::{env::args, path::PathBuf};

use eframe::{
    App, Frame,
    egui::{
        Button, CentralPanel, Context, InnerResponse, ScrollArea, TopBottomPanel, Ui,
        scroll_area::ScrollAreaOutput,
    },
};

fn main() {
    let native_options = eframe::NativeOptions::default();
    let result = eframe::run_native(
        "Audio sorter",
        native_options,
        Box::new(|cc| {
            let files = args().skip(1).map(PathBuf::from).collect::<Vec<_>>();

            Ok(Box::new(AudioSortApp::new(cc, files)))
        }),
    );

    if let Err(err) = result {
        eprintln!("{err}");
    }
}

struct AudioSortApp {
    files: Vec<PathBuf>,
}

impl App for AudioSortApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Files");
            ui.separator();
            self.files(ui);
        });

        TopBottomPanel::bottom("bottom_buttons").show(ctx, |ui| {
            Self::actions(ui);
        });
    }
}

impl AudioSortApp {
    const fn new(_cc: &eframe::CreationContext<'_>, files: Vec<PathBuf>) -> Self {
        Self { files }
    }

    fn files(&self, ui: &mut Ui) -> ScrollAreaOutput<()> {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for file in &self.files {
                    ui.label(file.to_string_lossy());
                }
            })
    }

    fn actions(ui: &mut Ui) -> InnerResponse<()> {
        ui.horizontal(|ui| if ui.add(Button::new("add")).clicked() {})
    }
}
