use std::{env::current_dir, os::windows::thread};

#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use eframe::egui::{self, Button, Rect, Response, Sense, Ui, Widget, Window, debug_text::print};
use egui_extras;
use simple_server::Server;

mod myapp;
use myapp::MyApp;
mod graphnode;

fn main() -> eframe::Result {
	// unsafe {
	// 	std::env::set_var("RUST_BACKTRACE", "1");
	// }
	std::thread::spawn(||{
		let server = Server::new(|request, mut response| {
			Ok(response.body("Hello, world!\nGo to http://localhost:7979/index.html".as_bytes().to_vec())?)
		});
		server.listen("127.0.0.1", "7979");
	});
	
	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_inner_size([1280., 720.]),
		..Default::default()
	};

	eframe::run_native(
		"ViVerN Editor",
		options,
		Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
	)
}