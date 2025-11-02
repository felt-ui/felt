use felt_platform::application::{Application, WindowOptions};

fn main() {
    Application::new()
        .run(|cx| {
            cx.open_window(WindowOptions::default());
        })
        .expect("failed to run application");
}
