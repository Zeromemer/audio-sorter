mod audio;

use std::env::args;

use eframe::{
    App, Frame,
    egui::{
        Align, CentralPanel, Context, InnerResponse, Layout, ScrollArea, TopBottomPanel, Ui,
        scroll_area::ScrollAreaOutput,
    },
    emath::OrderedFloat,
};
use rfd::FileDialog;

use crate::audio::{Audio, AudioExtractError};

fn main() {
    let native_options = eframe::NativeOptions::default();

    let files = args()
        .skip(1)
        .map(Audio::from_file)
        .collect::<Result<Vec<_>, AudioExtractError>>();
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
        Box::new(|cc| Ok(Box::new(AudioSortApp::new(cc, files)))),
    );

    if let Err(err) = result {
        eprintln!("{err}");
    }
}

struct AudioSortApp {
    audios: Vec<Audio>,
}

impl App for AudioSortApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Files");
            ui.separator();
            self.files(ui);
        });

        TopBottomPanel::bottom("bottom_buttons").show(ctx, |ui| {
            self.actions(ui);
        });
    }
}

impl AudioSortApp {
    const fn new(_cc: &eframe::CreationContext<'_>, files: Vec<Audio>) -> Self {
        Self { audios: files }
    }

    fn files(&mut self, ui: &mut Ui) -> ScrollAreaOutput<()> {
        ScrollArea::vertical().show(ui, |ui| {
            self.audios.retain(|file| {
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

    fn actions(&mut self, ui: &mut Ui) -> InnerResponse<()> {
        ui.horizontal(|ui| {
            if ui.button("add").clicked() {
                let files = select_files_to_add();
                match files {
                    Ok(mut files) => {
                        self.audios.append(&mut files);
                    }
                    Err(err) => {
                        println!("{err}");
                    }
                }
            }

            if ui.button("sort").clicked() {
                self.audios
                    .sort_by_cached_key(|a| OrderedFloat(a.mean_absolute()));
            }
        })
    }
}

fn select_files_to_add() -> Result<Vec<Audio>, AudioExtractError> {
    let handles = FileDialog::new().pick_files();

    handles.map_or_else(
        || Ok(Vec::new()),
        |handles| {
            handles
                .iter()
                .map(|path| Audio::from_file(path.to_string_lossy().into_owned()))
                .collect()
        },
    )
}
