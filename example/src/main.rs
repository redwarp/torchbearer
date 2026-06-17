use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::sync::Arc;
use tiny_skia::{
    Color, FilterQuality, LineCap, Paint, PathBuilder, Pixmap, PixmapPaint, Rect, Stroke, Transform,
};
use torchbearer::{
    Point,
    fov::{VisionMap, field_of_view},
    path::{PathMap, astar_path_fourwaygrid},
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::{Window, WindowId},
};
use winit_input_helper::WinitInputHelper;

const MAP_WIDTH: i32 = 20;
const MAP_HEIGHT: i32 = 20;
const SCALE: i32 = 24;

struct Rendering {
    pixmap: Pixmap,
    goblin: Pixmap,
    target: Pixmap,
    from: (i32, i32),
    to: (i32, i32),
    lines: Vec<(f32, f32)>,
    visible: Vec<(i32, i32)>,
    dirty: bool,
}

impl Rendering {
    fn new() -> Self {
        let pixmap = Pixmap::new((MAP_WIDTH * SCALE) as u32, (MAP_HEIGHT * SCALE) as u32).unwrap();

        let sprites = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("sprites")
            .unwrap();

        let goblin = Pixmap::load_png(sprites.join("goblin.png")).unwrap();
        let target = Pixmap::load_png(sprites.join("target.png")).unwrap();

        let from = (2, 2);
        let to = (12, 8);

        Self {
            pixmap,
            goblin,
            target,
            from,
            to,
            lines: vec![],
            visible: vec![],
            dirty: true,
        }
    }
}

struct ExampleMap {
    width: i32,
    height: i32,
    walkable: Vec<bool>,
}

impl ExampleMap {
    fn new(width: i32, height: i32) -> Self {
        ExampleMap {
            width,
            height,
            walkable: vec![true; (width * height) as usize],
        }
    }

    fn set_walkable(&mut self, x: i32, y: i32, is_walkable: bool) {
        let index = (x + y * self.width) as usize;
        self.walkable[index] = is_walkable;
    }

    fn draw(&self, screen: &mut [u8], rendering: &mut Rendering) {
        if rendering.dirty {
            rendering.lines =
                if let Some(path) = astar_path_fourwaygrid(self, rendering.from, rendering.to) {
                    path_to_line_elements(path)
                } else {
                    vec![]
                };
            rendering.visible = field_of_view(self, rendering.from, 8);
            rendering.dirty = false;
        } else {
            return;
        }

        let mut paint = Paint::default();
        paint.set_color_rgba8(0xe6, 0xcc, 0xe6, 0xff);

        let pixmap_paint = PixmapPaint {
            quality: FilterQuality::Bilinear,
            ..Default::default()
        };

        let pixmap = &mut rendering.pixmap;
        pixmap.fill(Color::WHITE);

        pixmap.fill_rect(
            Rect::from_ltrb(
                0 as f32,
                0 as f32,
                (MAP_WIDTH * SCALE) as f32,
                (MAP_HEIGHT * SCALE) as f32,
            )
            .unwrap(),
            &paint,
            Transform::default(),
            None,
        );

        paint.set_color_rgba8(0x4d, 0, 0, 0xff);

        for x in 0..self.width {
            for y in 0..self.height {
                if !self.is_walkable((x, y)) {
                    pixmap.fill_rect(
                        Rect::from_ltrb(
                            (x * SCALE) as f32,
                            (y * SCALE) as f32,
                            ((x + 1) * SCALE) as f32,
                            ((y + 1) * SCALE) as f32,
                        )
                        .unwrap(),
                        &paint,
                        Transform::default(),
                        None,
                    );
                }
            }
        }

        if !rendering.visible.is_empty() {
            paint.set_color_rgba8(0xff, 0xff, 0, 0x33);

            for &(x, y) in rendering.visible.iter() {
                pixmap.fill_rect(
                    Rect::from_ltrb(
                        (x * SCALE) as f32,
                        (y * SCALE) as f32,
                        ((x + 1) * SCALE) as f32,
                        ((y + 1) * SCALE) as f32,
                    )
                    .unwrap(),
                    &paint,
                    Transform::default(),
                    None,
                );
            }
        }

        let path = {
            if rendering.lines.is_empty() {
                None
            } else {
                let pb = rendering.lines[1..].iter().fold(
                    {
                        let origin = rendering.lines[0];
                        let mut pb = PathBuilder::new();

                        pb.move_to(origin.0, origin.1);
                        pb
                    },
                    |mut pb, point| {
                        pb.line_to(point.0, point.1);
                        pb
                    },
                );

                pb.finish()
            }
        };

        if let Some(path) = path {
            paint.set_color_rgba8(0xff, 0, 0, 0xff);
            let stroke = Stroke {
                width: 6.0,
                line_cap: LineCap::Round,
                ..Default::default()
            };

            pixmap.stroke_path(&path, &paint, &stroke, Transform::default(), None);
        }

        let translate_x = SCALE * rendering.from.0;
        let translate_y = SCALE * rendering.from.1;
        // Hack there: The bitmaps are actually @2x, seems like I worked on a retina computer before.
        // So I'm scaling them down when drawing them.
        pixmap.draw_pixmap(
            0,
            0,
            rendering.goblin.as_ref(),
            &pixmap_paint,
            Transform::from_translate(translate_x as f32, translate_y as f32).pre_scale(0.5, 0.5),
            None,
        );

        let translate_x = SCALE * rendering.to.0;
        let translate_y = SCALE * rendering.to.1;
        pixmap.draw_pixmap(
            0,
            0,
            rendering.target.as_ref(),
            &pixmap_paint,
            Transform::from_translate(translate_x as f32, translate_y as f32).pre_scale(0.5, 0.5),
            None,
        );

        screen.copy_from_slice(pixmap.data());
    }
}

impl VisionMap for ExampleMap {
    fn dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn is_transparent(&self, (x, y): Point) -> bool {
        let index = (x + y * self.width) as usize;
        self.walkable[index]
    }
}

impl PathMap for ExampleMap {
    fn dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn is_walkable(&self, (x, y): Point) -> bool {
        let index = (x + y * self.width) as usize;
        self.walkable.get(index).cloned().unwrap_or(false)
    }
}

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    input: WinitInputHelper,
    rendering: Rendering,
    map: ExampleMap,
    hidpi_factor: f64,
}

impl App {
    fn new() -> Self {
        let mut map = ExampleMap::new(MAP_WIDTH, MAP_HEIGHT);
        map.set_walkable(2, 5, false);
        Self {
            window: None,
            pixels: None,
            input: WinitInputHelper::new(),
            rendering: Rendering::new(),
            map,
            hidpi_factor: 1.0,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let width = (MAP_WIDTH * SCALE) as f64;
            let height = (MAP_HEIGHT * SCALE) as f64;
            let logical_size = LogicalSize::new(width, height);

            let window_attributes = Window::default_attributes()
                .with_title("Torchbearer Example")
                .with_inner_size(logical_size)
                .with_min_inner_size(logical_size);

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.hidpi_factor = window.scale_factor();

            // Center the window
            if let Some(monitor) = window.current_monitor() {
                let monitor_size = monitor.size().to_logical::<f64>(self.hidpi_factor);
                let window_size = window.inner_size().to_logical::<f64>(self.hidpi_factor);

                let center = LogicalPosition::new(
                    (monitor_size.width - window_size.width) / 2.0,
                    (monitor_size.height - window_size.height) / 2.0,
                );
                window.set_outer_position(center);
            }

            let physical_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(physical_size.width, physical_size.height, window.clone());

            let pixels = Pixels::new(
                (MAP_WIDTH * SCALE) as u32,
                (MAP_HEIGHT * SCALE) as u32,
                surface_texture,
            )
            .expect("Pixels error");

            self.window = Some(window);
            self.pixels = Some(pixels);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if self.input.process_window_event(&event) {
            if let WindowEvent::CloseRequested = event {
                event_loop.exit();
                return;
            }

            if let Some(size) = self.input.window_resized() {
                if let Some(pixels) = &mut self.pixels {
                    pixels.resize_surface(size.width, size.height).unwrap();
                }
            }

            if let Some(factor) = self.input.scale_factor_changed() {
                self.hidpi_factor = factor;
            }

            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }

        if let WindowEvent::RedrawRequested = event {
            if let (Some(pixels), Some(_window)) = (&mut self.pixels, &self.window) {
                self.map.draw(pixels.frame_mut(), &mut self.rendering);
                if let Err(err) = pixels.render() {
                    error!("pixels.render() failed: {err}");
                    event_loop.exit();
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.input.end_step();

        if self.input.key_pressed(KeyCode::Escape) {
            event_loop.exit();
            return;
        }

        if self.input.mouse_pressed(winit::event::MouseButton::Left) {
            let mouse_cell = self.input.cursor().map(|(mx, my)| {
                if let Some(pixels) = &self.pixels {
                    let (mx_i, my_i) = pixels
                        .window_pos_to_pixel((mx, my))
                        .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));
                    (mx_i as i32 / SCALE, my_i as i32 / SCALE)
                } else {
                    (0, 0)
                }
            });

            if let Some(mouse_cell) = mouse_cell {
                let is_walkable = self.map.is_walkable(mouse_cell);
                self.map
                    .set_walkable(mouse_cell.0, mouse_cell.1, !is_walkable);
                self.rendering.dirty = true;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        self.input.step();
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new().map_err(|e| Error::UserDefined(Box::new(e)))?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop
        .run_app(&mut app)
        .map_err(|e| Error::UserDefined(Box::new(e)))
}

fn path_to_line_elements(path: Vec<Point>) -> Vec<(f32, f32)> {
    path.into_iter()
        .map(|a| {
            (
                (a.0 as f32 + 0.5) * SCALE as f32,
                (a.1 as f32 + 0.5) * SCALE as f32,
            )
        })
        .collect()
}
