#![windows_subsystem = "windows"]

mod audio;

use anyhow::Result;
use std::env::args;

use eframe::{
    App, Frame,
    egui::{
        Align, CentralPanel, Context, InnerResponse, Layout, RichText, ScrollArea, TopBottomPanel,
        Ui, Window, scroll_area::ScrollAreaOutput,
    },
    emath::OrderedFloat,
};
use rfd::FileDialog;

use crate::audio::Audio;

fn main() {
    let native_options = eframe::NativeOptions::default();

    let audios = args()
        .skip(1)
        .map(Audio::from_file)
        .collect::<Result<Vec<_>, _>>();

    let result = eframe::run_native(
        "Audio sorter",
        native_options,
        Box::new(|cc| Ok(Box::new(AudioSortApp::new(cc, audios)))),
    );

    if let Err(err) = result {
        eprintln!("{err}");
    }
}

struct AudioSortApp {
    audios: Vec<Audio>,
    error_message: Option<String>,
}

impl App for AudioSortApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.error_message(ctx);

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Audios");
            ui.separator();
            self.audios(ui);
        });

        TopBottomPanel::bottom("bottom_buttons").show(ctx, |ui| {
            self.actions(ui);
        });
    }
}

impl AudioSortApp {
    fn new(_cc: &eframe::CreationContext<'_>, audios: Result<Vec<Audio>>) -> Self {
        match audios {
            Ok(audios) => Self {
                audios,
                error_message: None,
            },
            Err(err) => Self {
                audios: vec![],
                error_message: Some(format!("Error while processing files from arguments {err}")),
            },
        }
    }

    fn error_message(&mut self, ctx: &Context) {
        self.error_message.take_if(|msg| {
            let response = Window::new("Error").resizable(true).show(ctx, |ui| {
                ui.label(RichText::new(msg.as_str()).size(24.0).strong());
                ui.button("Ok").clicked()
            });

            response.is_some_and(|inner| inner.inner.is_some_and(|b| b))
        });
    }

    fn audios(&mut self, ui: &mut Ui) -> ScrollAreaOutput<()> {
        ScrollArea::vertical().show(ui, |ui| {
            self.audios.retain(|file| {
                let mut retain = true;

                ui.horizontal(|ui| {
                    if ui.label(file.path().to_string_lossy()).secondary_clicked() {
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
                let new_audios = select_audio_to_add();
                match new_audios {
                    Ok(mut new_audios) => {
                        self.audios.append(&mut new_audios);
                    }
                    Err(err) => {
                        self.error_message = Some(format!("Error while processing files: {err}"));
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

fn select_audio_to_add() -> Result<Vec<Audio>> {
    let handles = FileDialog::new().pick_files();

    handles.map_or_else(
        || Ok(Vec::new()),
        |handles| handles.iter().map(Audio::from_file).collect(),
    )
}
