use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use tiny_skia::{
    Color, FilterQuality, LineCap, Paint, PathBuilder, Pixmap, PixmapPaint, Rect, Stroke, Transform,
};
use torchbearer::{
    bresenham::Angle,
    fov::{cone_of_view, VisionMap},
    path::{astar_path_fourwaygrid, PathMap},
    Point,
};
use winit::{
    dpi::{LogicalPosition, PhysicalSize},
    event::Event,
    event_loop::{ControlFlow, EventLoop},
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

        let from = (10, 10);
        let to = (12, 18);

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
            rendering.visible = cone_of_view(
                self,
                rendering.from,
                10,
                Angle::Degrees(15),
                Angle::Degrees(70),
            );
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
        self.walkable[index]
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let (window, p_width, p_height, mut _hidpi_factor) = create_window("Sample", &event_loop);

    let surface_texture = SurfaceTexture::new(p_width, p_height, &window);
    let mut map = ExampleMap::new(MAP_WIDTH, MAP_HEIGHT);
    map.set_walkable(2, 5, false);
    let mut pixels = Pixels::new(
        (MAP_WIDTH * SCALE) as u32,
        (MAP_HEIGHT * SCALE) as u32,
        surface_texture,
    )?;
    let mut rendering = Rendering::new();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            map.draw(pixels.get_frame_mut(), &mut rendering);
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(winit::event::VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.mouse_pressed(0) {
                let mouse_cell = input.mouse().map(|(mx, my)| {
                    let (mx_i, my_i) = pixels
                        .window_pos_to_pixel((mx, my))
                        .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));
                    (mx_i as i32 / SCALE, my_i as i32 / SCALE)
                });

                if let Some(mouse_cell) = mouse_cell {
                    let is_walkable = map.is_walkable(mouse_cell);
                    map.set_walkable(mouse_cell.0, mouse_cell.1, !is_walkable);
                    rendering.dirty = true;
                }
            }
            // Adjust high DPI factor
            if let Some(factor) = input.scale_factor_changed() {
                _hidpi_factor = factor;
            }
            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height).unwrap();
            }

            window.request_redraw();
        }
    });
}

fn create_window(
    title: &str,
    event_loop: &EventLoop<()>,
) -> (winit::window::Window, u32, u32, f64) {
    let window = winit::window::WindowBuilder::new()
        .with_visible(false)
        .with_title(title)
        .build(event_loop)
        .unwrap();

    let hidpi_factor = window.scale_factor();

    // Get dimensions
    let width = (MAP_WIDTH * SCALE) as f64;
    let height = (MAP_HEIGHT * SCALE) as f64;
    let (monitor_width, monitor_height) = {
        if let Some(monitor) = window.current_monitor() {
            let size = monitor.size().to_logical(hidpi_factor);
            (size.width, size.height)
        } else {
            (width, height)
        }
    };
    let scale = (monitor_height / height * 2.0 / 3.0).round().max(1.0);

    // Resize, center, and display the window
    let default_size: winit::dpi::LogicalSize<f64> =
        PhysicalSize::new(width, height).to_logical(hidpi_factor);
    let center = LogicalPosition::new(
        (monitor_width - width * scale) / 2.0,
        (monitor_height - height * scale) / 2.0,
    );
    window.set_inner_size(default_size);
    window.set_min_inner_size(Some(default_size));
    window.set_outer_position(center);
    window.set_visible(true);

    let size = default_size.to_physical::<f64>(hidpi_factor);

    (
        window,
        size.width.round() as u32,
        size.height.round() as u32,
        hidpi_factor,
    )
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
