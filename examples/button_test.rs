#[macro_use]
extern crate kiss_ui;

use kiss_ui::prelude::*;

use kiss_ui::button::Button;
use kiss_ui::container::{Horizontal, Vertical};
use kiss_ui::dialog::{self, AlertPopupBuilder};

fn main() {
    kiss_ui::show_gui(||
        Dialog::new(
            Vertical::new(children![
	            Horizontal::new(
	                children![
	                    Button::new()
	                        .set_label("Message #1")
	                        .set_name("Message_1")
	                        .set_onclick(show_message_dialog),
	                    Button::new()
	                        .set_label("Message #2")
	                        .set_name("Message_2")
	                        .set_onclick(show_message_dialog),
	                    Button::new()
	                        .set_label("change the name")
	                        .set_name("bar")
	                        .set_onclick(show_change_name),
	                ]
	            )
	            .set_elem_spacing_pixels(10),
	            Horizontal::new(
	                children![
	                    Button::new()
	                        .set_label("Alert")
	                        .set_onclick(show_alert_dialog),
	                    Button::new()
	                        .set_label("Close")
	                        .set_onclick(close_dialog),
	                ]
	            )
	            .set_elem_spacing_pixels(10)
            ])
        )
        .set_title("Button test!")
    )
}

fn show_message_dialog(btn: Button) {
    let name = btn.get_name().unwrap();
    dialog::message_popup("Good job!", format!("You clicked the button {:?}!", name));
}

fn show_change_name(btn: Button) {
    let name = btn.get_name().unwrap();
    dialog::message_popup("Lets try it!", format!("The button's name is '{}'!\n
        Now we try to change it!\n
        You will get a Panic if you still use the 'name'", name));
//    drop(name); // <= uncomment this line
    btn.set_name("foo");
    show_message_dialog(btn);
}

fn show_alert_dialog(_: Button) {
    let res = AlertPopupBuilder::new("Alert!", "You clicked the other button!", "Yes")
        .button2("No")
        .button3("Cancel")
        .popup();

    println!("Alert result = {}", res);
}

fn close_dialog(_: Button) -> CallbackStatus {
    println!("Closing dialog!");
    CallbackStatus::Close
}
