use {
    nannou::{app::App, frame::Frame},
    nannou_imageutil::capture::CaptureHelper,
    std::fs,
};

fn main() {
    nannou::app(model).exit(exit).run();
}

struct Model {
    capture_helper: CaptureHelper,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(250, 200)
        .title("capture_same_aspect_ratio")
        .view(view)
        .build()
        .unwrap();

    // Make sure the directory where we will save images to exists.
    fs::create_dir_all(&capture_directory(app)).unwrap();

    Model {
        capture_helper: CaptureHelper::from_main_window(app, [2000, 2000]),
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    let elapsed_frames = app.main_window().elapsed_frames();
    draw.ellipse()
        .x_y(0.0, 0.0)
        .radius(300.0 + ((elapsed_frames as f32) * 10.0));

    model.capture_helper.render_image(app, &draw);
    model.capture_helper.display_in_window(&frame);

    let path = capture_directory(app)
        .join(elapsed_frames.to_string())
        .with_extension("png");
    model.capture_helper.write_to_file(path).unwrap();
}

fn exit(app: &App, mut model: Model) {
    model.capture_helper.close(app).unwrap();
}

fn capture_directory(app: &App) -> std::path::PathBuf {
    app.project_path()
        .expect("Could not locate project path.")
        .join("example_images")
        .join(app.exe_name().unwrap())
}
