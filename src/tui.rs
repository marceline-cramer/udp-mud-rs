use crate::Pronouns;
use crossbeam_channel::{Receiver, Sender};
use cursive::align::*;
use cursive::theme::*;
use cursive::traits::*;
use cursive::views::*;
use cursive::{Cursive, CursiveExt};

pub enum EditEvent {
    Name(String),
    About(String),
    Pronouns(Option<Pronouns>),
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
                pronouns: None,
            },
        }
    }

    pub fn run(&mut self) {
        Self::edit_identity(&mut self.cursive);
        self.cursive.run();

        while let Ok(edit) = self.edit_receiver.try_recv() {
            match edit {
                EditEvent::Name(name) => self.identity.name = name,
                EditEvent::About(about) => self.identity.about = about,
                EditEvent::Pronouns(pronouns) => self.identity.pronouns = pronouns,
            }
        }

        Self::show_main(&mut self.cursive);
        self.cursive.run();
    }

    pub fn show_main(siv: &mut Cursive) {
        let list = ListView::new()
            .child("message_0000", TextView::new("Hello, world!"))
            .with_name("messages_list")
            .scrollable()
            .full_height();

        let message_edit = EditView::new()
            .on_submit(|siv, text| {
                siv.call_on_name("message_edit", |message: &mut EditView| {
                    message.set_content("");
                });
                siv.call_on_name("messages_list", |list: &mut ListView| {
                    list.add_child("new_message", TextView::new(text));
                });
            })
            .with_name("message_edit")
            .full_width();

        let chat = LinearLayout::vertical().child(list).child(message_edit);

        let mut rooms = SelectView::<&'static str>::new().on_submit(|siv, room: &str| {
            let dialog = Dialog::info(format!("Selected room: {}", room));
            siv.add_layer(dialog);
        });

        rooms.add_item("Room 1", "room_id_1");
        rooms.add_item("Room 2", "room_id_2");
        rooms.add_item("Room 3", "room_id_3");

        let rooms = Dialog::around(rooms)
            .title("Rooms")
            .title_position(HAlign::Left);

        let mut connections = SelectView::new();
        connections.add_item("Connection 1", "connection_1");
        let connections = Dialog::around(connections)
            .title("Connections")
            .title_position(HAlign::Left);

        let sidebar = LinearLayout::vertical().child(rooms).child(connections);

        let layout = LinearLayout::horizontal().child(sidebar).child(chat);

        siv.add_fullscreen_layer(layout);
    }

    pub fn edit_identity(siv: &mut Cursive) {
        let labels = make_vertical_labels(&["Name:", "About:", "Pronouns:"]).fixed_width(10);

        let values = LinearLayout::vertical()
            .child(EditView::new().with_name("name_edit"))
            .child(EditView::new().with_name("about_edit"))
            .child(TextView::new("<none>").with_name("pronouns_text"))
            .fixed_width(45);

        let columns = LinearLayout::horizontal().child(labels).child(values);
        let mut dialog = Dialog::around(columns);
        dialog.set_title("Edit Identity");
        dialog.add_button("Select Pronouns...", |siv| Self::select_pronouns(siv));
        dialog.add_button("Ok", |siv| {
            let name = get_edit_contents(siv, "name_edit");
            let about = get_edit_contents(siv, "about_edit");
            siv.with_user_data(|sender: &mut Sender<EditEvent>| {
                sender.send(EditEvent::Name(name)).unwrap();
                sender.send(EditEvent::About(about)).unwrap();
            });
            siv.quit();
        });

        siv.add_layer(dialog);
    }

    pub fn select_pronouns(siv: &mut Cursive) {
        let mut dialog = Dialog::new();
        dialog.set_title("Select Pronouns");
        dialog.add_button("Custom...", |siv| {
            siv.pop_layer();
            Self::edit_pronouns(siv);
        });
        dialog.add_button("None", |siv| {
            siv.with_user_data(|sender: &mut Sender<EditEvent>| {
                sender.send(EditEvent::Pronouns(None)).unwrap();
            });
            siv.call_on_name("pronouns_text", |view: &mut TextView| {
                view.set_content("<none>");
            })
            .unwrap();
            siv.pop_layer();
        });
        dialog.add_button("Cancel", |siv| {
            siv.pop_layer().unwrap();
        });
        siv.add_layer(dialog);
    }

    pub fn edit_pronouns(siv: &mut Cursive) {
        let labels = make_vertical_labels(&[
            "Case-sensitive:",
            "Plural:",
            "Subject:",
            "Object:",
            "Possessive:",
            "Possessive pronoun:",
            "Reflexive:",
        ])
        .fixed_width(20);

        let values = LinearLayout::vertical()
            .child(Checkbox::new().with_name("case_sensitive_edit"))
            .child(Checkbox::new().with_name("plural_edit"))
            .child(EditView::new().with_name("subject_edit"))
            .child(EditView::new().with_name("object_edit"))
            .child(EditView::new().with_name("possessive_edit"))
            .child(EditView::new().with_name("possessive_pronoun_edit"))
            .child(EditView::new().with_name("reflexive_edit"))
            .fixed_width(12);

        let columns = LinearLayout::horizontal().child(labels).child(values);
        let mut dialog = Dialog::around(columns);
        dialog.set_title("Edit Pronouns");
        dialog.add_button("Ok", |siv| {
            let case_sensitive = get_checkbox_contents(siv, "case_sensitive_edit");
            let plural = get_checkbox_contents(siv, "plural_edit");
            let subject = get_edit_contents(siv, "subject_edit");
            let object = get_edit_contents(siv, "object_edit");
            let possessive = get_edit_contents(siv, "possessive_edit");
            let possessive_pronoun = get_edit_contents(siv, "possessive_pronoun_edit");
            let reflexive = get_edit_contents(siv, "reflexive_edit");

            let pronouns = Pronouns {
                case_sensitive,
                plural,
                subject,
                object,
                possessive,
                possessive_pronoun,
                reflexive,
            };

            siv.call_on_name("pronouns_text", |view: &mut TextView| {
                view.set_content(pronouns.format_full());
            })
            .unwrap();

            siv.with_user_data(|sender: &mut Sender<EditEvent>| {
                sender.send(EditEvent::Pronouns(Some(pronouns))).unwrap();
            });

            siv.pop_layer();
        });
        dialog.add_button("Cancel", |siv| {
            siv.pop_layer().unwrap();
        });

        siv.add_layer(dialog);
    }
}

pub struct Identity {
    pub name: String,
    pub about: String,
    pub pronouns: Option<Pronouns>,
}

fn get_edit_contents(siv: &mut Cursive, name: &str) -> String {
    siv.call_on_name(name, |view: &mut EditView| view.get_content())
        .unwrap()
        .to_string()
}

fn get_checkbox_contents(siv: &mut Cursive, name: &str) -> bool {
    siv.call_on_name(name, |view: &mut Checkbox| view.is_checked())
        .unwrap()
}

fn make_vertical_labels(labels: &[&str]) -> LinearLayout {
    let mut layout = LinearLayout::vertical();
    for label in labels.iter() {
        layout.add_child(TextView::new(label.to_string()));
    }
    layout
}
