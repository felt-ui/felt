use felt_platform::application::{Application, WindowOptions};

fn main() {
    Application::new()
        .run(|cx| {
            cx.open_window(WindowOptions {
                transparent: Some(true),
                ..Default::default()
            });
        })
        .expect("failed to run application");
}
