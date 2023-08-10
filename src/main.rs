mod nes;
use gtk4::{prelude::*, Application, ApplicationWindow, Box, Button};

fn main() {
    let application = Application::builder().build();
    application.connect_activate(ui);
    application.run();
}

fn ui(application: &Application) {
    let nes = Button::builder().label("NES").build();
    let gb = Button::builder().label("GB").build();
    let snes = Button::builder().label("SNES").build();
    let gba = Button::builder().label("GBA").build();
    nes.set_size_request(200, 200);
    gb.set_size_request(200, 200);
    snes.set_size_request(200, 200);
    gba.set_size_request(200, 200);
    nes.connect_clicked(|_| nes::main::nes());
    let buttons = Box::builder().build();
    buttons.append(&nes);
    buttons.append(&gb);
    buttons.append(&snes);
    buttons.append(&gba);
    ApplicationWindow::builder().title("MEMU").application(application).child(&buttons).build().present();
}
