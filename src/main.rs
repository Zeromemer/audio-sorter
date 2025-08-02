mod audio;

use std::env::args;

use eframe::{
    App, Frame,
    egui::{
        Align, Button, CentralPanel, Context, InnerResponse, Layout, ScrollArea, TopBottomPanel,
        Ui, scroll_area::ScrollAreaOutput,
    },
};

use crate::audio::{Audio, AudioExtractError};

fn main() {
    let native_options = eframe::NativeOptions::default();

    let files = args().skip(1).map(Audio::from_file).collect::<Result<Vec<_>, AudioExtractError>>();
    let files = match files {
        Ok(files) => files,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    let result = eframe::run_native(
        "Audio sorter",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(AudioSortApp::new(cc, files)))
        }),
    );

    if let Err(err) = result {
        eprintln!("{err}");
    }
}

struct AudioSortApp {
    files: Vec<Audio>,
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
    const fn new(_cc: &eframe::CreationContext<'_>, files: Vec<Audio>) -> Self {
        Self { files }
    }

    fn files(&mut self, ui: &mut Ui) -> ScrollAreaOutput<()> {
        ScrollArea::vertical().show(ui, |ui| {
            self.files.retain(|file| {
                let mut retain = true;

                ui.horizontal(|ui| {
                    if ui.label(file.path()).double_clicked() {
                        println!("{:?}", file.pcm());
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Delete").clicked() {
                            retain = false;
                        }
                    });
                });

                retain
            });
        })
    }

    fn actions(ui: &mut Ui) -> InnerResponse<()> {
        ui.horizontal(|ui| if ui.add(Button::new("add")).clicked() {})
    }
}
