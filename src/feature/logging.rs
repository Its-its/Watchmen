use fern::Dispatch;
use fern::colors::{Color, ColoredLevelConfig};
use log::{LevelFilter, Metadata, Record};


pub fn configure() {
	let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        // we actually don't need to specify the color for debug and info, they are white by default
        .info(Color::White)
        .debug(Color::White)
        // depending on the terminals color scheme, this is the same as the background color
        .trace(Color::BrightBlack);

	let colors_level: ColoredLevelConfig = colors_line.clone().info(Color::Green);

	Dispatch::new()
		.chain(std::io::stdout())
		.format(move |out, message, record| {
			out.finish(format_args!(
                "{color_line}[{date}][{target}][{level}{color_line}] {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ))
		})
		.level(LevelFilter::Info)
		.level_for("tokio_reactor", LevelFilter::Info)
		.apply()
		.expect("fern logging");

	// log::set_max_level(LevelFilter::Off);
	// log_reroute::init().expect("log_reroute::init()");
	// log_reroute::reroute(ConsoleLog);
}

// pub struct ConsoleLog;

// impl Log for ConsoleLog {
//     fn enabled(&self, _metadata: &Metadata) -> bool {
//         false
//     }

//     fn log(&self, record: &Record) {
// 		let mut lines = super::PRINT_LINES.lock().unwrap();

// 		lines.push((format!("{}", record.args()), record.level()));

// 		if lines.len() > 40 {
// 			lines.pop();
// 		}
// 	}

//     fn flush(&self) {}
// }