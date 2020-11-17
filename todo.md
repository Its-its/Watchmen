URL
 - Cookies
 - Headers
 - Etc..

Fix error on other crate:
	thread 'main' panicked at 'html5ever: Custom { kind: TimedOut, error: "timed out" }', D:\Coding\Rust\2020\xpath\src\lib.rs:30:10
	note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
	thread '<unnamed>' panicked at 'called `Result::unwrap()` on an `Err` value: "PoisonError { inner: .. }"', src\core.rs:80:23
	thread 'actix-rt:worker:3' panicked at 'called `Result::unwrap()` on an `Err` value: "PoisonError { inner: .. }"', src\core.rs:80:23
	thread '<unnamed>' panicked at 'called `Result::unwrap()` on an `Err` value: Error(Hyper(hyper::Error(Io, Os { code: 10093, kind: Other, message: "Either the application has not called WSAStartup, or WSAStartup failed." })))', src\feature\telegram\mod.rs:77:66

Popup fixes
Socket/RPC updates. like actually responding to the active sockets instantly.


TODO: Split up into two crates.
The Watchmen crate and the GUI Crate.

Watchmen Crate will always run in daemon(?)
GUI Crate will call to Watchmen Crate.