mod nes;
use gtk4::{prelude::*, Application, ApplicationWindow, Box, Button};

fn main() {
    let application = Application::builder().build();
    application.connect_activate(|application| ApplicationWindow::builder().title("MEMU").application(application).child(&buttons()).build().present());
    application.run();
}

fn buttons() -> Box {
    let buttons = Box::builder().build();
    for (label, callback) in [("NES", nes::main::nes), ("GB", nes::main::nes), ("SNES", nes::main::nes), ("GBA", nes::main::nes)] {
        let button = Button::builder().label(label).build();
        button.set_size_request(100, 100);
        button.connect_clicked(move |_| callback());
        buttons.append(&button);
    }
    buttons
}
