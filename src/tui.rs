use crate::pronouns::Pronouns;
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
        /*Self::edit_identity(&mut self.cursive);
        self.cursive.run();

        while let Ok(edit) = self.edit_receiver.try_recv() {
            match edit {
                EditEvent::Name(name) => self.identity.name = name,
                EditEvent::About(about) => self.identity.about = about,
                EditEvent::Pronouns(pronouns) => self.identity.pronouns = pronouns,
            }
        }

        Self::show_main(&mut self.cursive);
        self.cursive.run();*/

        Self::show_welcome_mat(&mut self.cursive);
        self.cursive.run();
    }

    pub fn show_welcome_mat(siv: &mut Cursive) {
        let logo = TextView::new(include_str!("logo.txt")).center();
        let labels = TextView::new("Identity:");
        let values = Button::new_raw("<none>", |siv| Self::edit_identity(siv));
        let config = LinearLayout::horizontal().child(labels).child(values);
        let layout = LinearLayout::vertical().child(logo).child(config);
        let dialog = Dialog::around(layout)
            .title("Welcome User!")
            .button("Link start!", |siv| Self::show_main(siv))
            .button("Quit", Cursive::quit);
        siv.add_layer(dialog);
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
            siv.pop_layer();
        });

        siv.add_layer(dialog);
    }

    fn make_example_usage_panel() -> impl View {
        let text = TextView::new("Highlight a pronoun set to preview its usage!")
            .with_name("pronoun_example_text")
            .fixed_width(50)
            .scrollable();
        Panel::new(text).title("Example Usage")
    }

    fn update_pronouns_edit(siv: &mut Cursive, pronouns: &Pronouns) {
        siv.call_on_name("pronoun_example_text", |view: &mut TextView| {
            view.set_content(pronouns.make_example_usage());
        });

        siv.with_user_data(|sender: &mut Sender<EditEvent>| {
            sender
                .send(EditEvent::Pronouns(Some(pronouns.clone())))
                .unwrap();
        });

        siv.call_on_name("pronouns_text", |view: &mut TextView| {
            view.set_content(pronouns.format_full());
        })
        .unwrap();
    }

    pub fn select_pronouns(siv: &mut Cursive) {
        let example = TextView::new("Highlight a pronoun set to preview its usage!")
            .with_name("pronoun_example_text")
            .fixed_width(35)
            .scrollable();

        let presets = SelectView::new()
            .with_all(
                crate::pronouns::make_presets()
                    .into_iter()
                    .map(|pronouns| (pronouns.format_full(), pronouns)),
            )
            .on_select(|siv, pronouns| Self::update_pronouns_edit(siv, pronouns))
            .scrollable();

        let layout = LinearLayout::horizontal()
            .child(Panel::new(presets).title("Presets"))
            .child(Panel::new(example).title("Example Usage"));

        let dialog = Dialog::around(layout)
            .title("Select Pronouns")
            .button("Custom...", |siv| {
                siv.pop_layer();
                Self::edit_pronouns(siv);
            })
            .dismiss_button("Ok")
            .button("None", |siv| {
                siv.with_user_data(|sender: &mut Sender<EditEvent>| {
                    sender.send(EditEvent::Pronouns(None)).unwrap();
                });
                siv.call_on_name("pronouns_text", |view: &mut TextView| {
                    view.set_content("<none>");
                })
                .unwrap();
                siv.pop_layer();
            });
        siv.add_layer(dialog);
    }

    fn get_edit_pronouns(siv: &mut Cursive) -> Pronouns {
        let case_sensitive = get_checkbox_contents(siv, "case_sensitive_edit");
        let plural = get_checkbox_contents(siv, "plural_edit");
        let subject = get_edit_contents(siv, "subject_edit");
        let object = get_edit_contents(siv, "object_edit");
        let possessive = get_edit_contents(siv, "possessive_edit");
        let possessive_pronoun = get_edit_contents(siv, "possessive_pronoun_edit");
        let reflexive = get_edit_contents(siv, "reflexive_edit");

        Pronouns {
            case_sensitive,
            plural,
            subject,
            object,
            possessive,
            possessive_pronoun,
            reflexive,
        }
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

        let mut values = LinearLayout::vertical();

        let checkboxes = &["case_sensitive_edit", "plural_edit"];

        for name in checkboxes.iter() {
            values.add_child(
                Checkbox::new()
                    .on_change(|siv, _value| {
                        let pronouns = Self::get_edit_pronouns(siv);
                        Self::update_pronouns_edit(siv, &pronouns);
                    })
                    .with_name(*name),
            );
        }

        let edit_views = &[
            "subject_edit",
            "object_edit",
            "possessive_edit",
            "possessive_pronoun_edit",
            "reflexive_edit",
        ];

        for name in edit_views.iter() {
            values.add_child(
                EditView::new()
                    .on_edit(|siv, _text, _cursor| {
                        let pronouns = Self::get_edit_pronouns(siv);
                        Self::update_pronouns_edit(siv, &pronouns);
                    })
                    .with_name(*name),
            );
        }

        let edit_layout = Panel::new(
            LinearLayout::horizontal()
                .child(labels)
                .child(values.fixed_width(12)),
        );

        let example = Self::make_example_usage_panel();

        let layout = LinearLayout::horizontal().child(edit_layout).child(example);

        let dialog = Dialog::around(layout)
            .title("Edit Pronouns")
            .button("Ok", |siv| {
                let pronouns = Self::get_edit_pronouns(siv);
                Self::update_pronouns_edit(siv, &pronouns);
                siv.pop_layer();
            })
            .dismiss_button("Cancel");

        siv.add_layer(dialog);
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
            .button("Create...", |siv| {
                siv.add_layer(Dialog::info("Room creation dialog goes here"))
            })
            .title("Rooms")
            .title_position(HAlign::Left)
            .with_name("room_select");

        let mut connections = SelectView::new();
        connections.add_item("Connection 1", "connection_1");
        let connections = Dialog::around(connections)
            .title("Connections")
            .title_position(HAlign::Left)
            .with_name("connection_s");

        let sidebar = LinearLayout::vertical().child(rooms).child(connections);

        let layout = LinearLayout::horizontal().child(sidebar).child(chat);

        siv.add_fullscreen_layer(layout);
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
