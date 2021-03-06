#[macro_use]
extern crate kiss_ui;

use kiss_ui::prelude::*;

use kiss_ui::button::Button;
use kiss_ui::container::Vertical;
use kiss_ui::dialog;
use kiss_ui::text::{Label, TextBox};

fn main() {
    kiss_ui::show_gui(|| {
        Dialog::new(
            Vertical::new(
                children![
                    Label::new("Enter a message:"),
                    TextBox::new()
                        .set_visible_columns(20)
                        .set_name("my_textbox"),
                    Button::new()
                        .set_label("Save")
                        .set_onclick(show_alert_message),
                ]
            )
        )
        .set_title("Textbox Test")
    });
}

fn show_alert_message(clicked: Button) {
    let dialog = clicked.get_dialog().unwrap();
    let text_box = dialog.get_child("my_textbox").unwrap()
        .try_downcast::<TextBox>().ok().expect("child my_textbox was not a TextBox!");
    let text = text_box.get_text();

    dialog::message_popup("Message saved!", format!("Your message: {}", text));
}
