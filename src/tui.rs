use crossbeam_channel::{Receiver, Sender};
use cursive::theme::*;
use cursive::traits::*;
use cursive::views::*;
use cursive::{Cursive, CursiveExt};

pub enum EditEvent {
    Name(String),
    About(String),
}

pub struct Tui {
    cursive: Cursive,
    edit_receiver: Receiver<EditEvent>,
    identity: Identity,
}

impl Tui {
    pub fn new() -> Self {
        let (edit_sender, edit_receiver) = crossbeam_channel::unbounded();
        let mut cursive = Cursive::new();
        cursive.set_user_data(edit_sender);

        cursive.update_theme(|theme| {
            theme.shadow = false;
            theme.borders = BorderStyle::Simple;

            let palette = &mut theme.palette;
            palette[PaletteColor::Background] = Color::TerminalDefault;
            palette[PaletteColor::View] = Color::TerminalDefault;
            palette[PaletteColor::Primary] = Color::TerminalDefault;
        });

        Self {
            cursive,
            edit_receiver,
            identity: Identity {
                name: "<display name>".into(),
                about: "<short about text>".into(),
            },
        }
    }

    pub fn run(&mut self) {
        self.edit_identity();

        while let Ok(edit) = self.edit_receiver.try_recv() {
            match edit {
                EditEvent::Name(name) => self.identity.name = name,
                EditEvent::About(about) => self.identity.about = about,
            }
        }
    }

    pub fn edit_identity(&mut self) {
        self.identity.edit(&mut self.cursive);
        self.cursive.run();
    }
}

pub struct Identity {
    pub name: String,
    pub about: String,
}

impl Identity {
    pub fn edit(&mut self, siv: &mut Cursive) {
        let labels = LinearLayout::vertical()
            .child(TextView::new("Name"))
            .child(TextView::new("About"))
            .fixed_width(10);

        let values = LinearLayout::vertical()
            .child(TextView::new(&self.name).with_name("name_text"))
            .child(TextView::new(&self.about).with_name("about_text"))
            .fixed_width(45);

        let buttons = LinearLayout::vertical()
            .child(Button::new("Edit", |siv| {
                siv.add_layer(
                    Dialog::around(
                        EditView::new()
                            .on_submit(|s, text| {
                                s.call_on_name("name_text", |view: &mut TextView| {
                                    view.set_content(text);
                                })
                                .unwrap();
                                s.with_user_data(|sender: &mut Sender<EditEvent>| {
                                    sender.send(EditEvent::Name(text.to_string())).unwrap();
                                });
                                s.pop_layer();
                            })
                            .with_name("name_edit")
                            .fixed_width(32),
                    )
                    .title("Edit Name")
                    .button("Cancel", |s| {
                        s.pop_layer();
                    }),
                );
            }))
            .child(Button::new("Edit", |siv| {
                siv.add_layer(
                    Dialog::around(
                        EditView::new()
                            .on_submit(|s, text| {
                                s.call_on_name("about_text", |view: &mut TextView| {
                                    view.set_content(text);
                                })
                                .unwrap();
                                s.with_user_data(|sender: &mut Sender<EditEvent>| {
                                    sender.send(EditEvent::About(text.to_string())).unwrap();
                                });
                                s.pop_layer();
                            })
                            .with_name("about_edit")
                            .fixed_width(32),
                    )
                    .title("Edit About")
                    .button("Cancel", |s| {
                        s.pop_layer();
                    }),
                );
            }));

        let mut dialog = Dialog::around(
            LinearLayout::horizontal()
                .child(labels)
                .child(values)
                .child(buttons),
        )
        .title("Edit Identity");
        dialog.add_button("Ok", |siv| {
            siv.pop_layer();
        });
        dialog.add_button("Quit", |siv| siv.quit());

        siv.add_layer(dialog);
    }
}
