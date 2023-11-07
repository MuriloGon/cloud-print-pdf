mod config;
mod logger;
mod message;
mod printer;

use config::AppConfig;
use inotify::{Inotify, WatchMask};
use printer::Printer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::setup_logger()?;

    let app_config = AppConfig::from_config_file();
    let printer = printer::Printer::new(&app_config);

    // Init dirs
    init_directories(&app_config);

    // Notification that list all directory change on {workdir}/pending
    let mut inotify = Inotify::init().expect("Error ao iniciar inotify");
    inotify
        .watches()
        .add("wdir/pending", WatchMask::CREATE | WatchMask::MOVED_TO)
        .expect("Falha ao adicionar inotify watcher");

    // Process that handles print
    init_printer_handler(&mut inotify, &printer);

    Ok(())
}

fn init_directories(config: &AppConfig) {
    AppConfig::generate_working_dir(config)
}

fn init_printer_handler(inotify: &mut Inotify, printer: &Printer) {
    let mut buffer = [0; 1024];

    loop {
        let events = inotify
            .read_events_blocking(&mut buffer)
            .expect("Error while reading events");

        for event in events {
            println!("{:?}", event.name);
            let printer = printer.clone();

            let path_string = event.name.unwrap().to_str().unwrap().to_string();
            let msg_result = printer.import_message(&path_string);

            println!("{:?}", msg_result);

            match &msg_result {
                Ok(msg_ok) => {
                    let print_result = printer.print_file(&msg_ok);
                    if let Err(error_msg) = print_result {
                        let mut msg = msg_ok.clone();
                        msg.set_error(error_msg);
                        printer.update_message(&path_string, &msg);
                        printer.move_message(printer::PrintStatus::Error, &path_string);
                    }

                    let mut msg = msg_ok.clone();
                    msg.set_successful();
                    printer.update_message(&path_string, &msg);
                    printer.move_message(printer::PrintStatus::Ok, &path_string);
                }
                Err(msg_error) => {
                    let msg = msg_error.clone();
                    printer.update_message(&path_string, &msg);
                    printer.move_message(printer::PrintStatus::Error, &path_string);
                }
            }
        }
    }
}
