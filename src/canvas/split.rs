
use color::Color;
use graphics::math::Scalar;
use position::{self, Dimensions, Direction, Point};
use theme::Theme;
use ui::{self, Ui};

use super::{CanvasId, Kind};

/// The length of a Split.
pub type Length = Scalar;

/// The current state of a Split.
#[derive(Clone, Debug, PartialEq)]
pub struct State;

/// A type of Canvas for flexibly designing and guiding widget layout as splits of a window.
pub struct Split<'a> {
    id: CanvasId,
    maybe_splits: Option<(Direction, &'a [Split<'a>])>,
    maybe_length: Option<f64>,
    style: Style,
    //maybe_adjustable: Option<Bounds>,
}

/// Describes the style of a Canvas Split.
#[derive(Clone, Debug, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Style {
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_color: Option<Color>,
    padding: Padding,
    margin: Margin,
}

/// The distance between the edge of a widget and the inner edge of a Canvas' frame.
#[derive(Clone, Debug, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Padding {
    maybe_top: Option<f64>,
    maybe_bottom: Option<f64>,
    maybe_left: Option<f64>,
    maybe_right: Option<f64>,
}

/// The distance between the edge of a Canvas' outer dimensions and the outer edge of its frame.
#[derive(Clone, Debug, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Margin {
    maybe_top: Option<f64>,
    maybe_bottom: Option<f64>,
    maybe_left: Option<f64>,
    maybe_right: Option<f64>,
}

impl<'a> Split<'a> {

    /// Construct a default Canvas Split.
    pub fn new(id: CanvasId) -> Split<'a> {
        Split {
            id: id,
            maybe_splits: None,
            maybe_length: None,
            //maybe_adjustable: None,
            style: Style::new(),
        }
    }

    /// Set the dimension of the Split.
    pub fn length(mut self, length: Length) -> Split<'a> {
        self.maybe_length = Some(length);
        self
    }
    
    /// Set the child Canvas Splits of the current Canvas flowing in a given direction.
    pub fn flow(mut self, dir: Direction, splits: &'a [Split<'a>]) -> Split<'a> {
        self.maybe_splits = Some((dir, splits));
        self
    }

    /// Set the child Canvasses flowing downwards.
    pub fn flow_down(self, splits: &'a [Split<'a>]) -> Split<'a> {
        self.flow(Direction::Down, splits)
    }

    /// Set the child Canvasses flowing upwards.
    pub fn flow_up(self, splits: &'a [Split<'a>]) -> Split<'a> {
        self.flow(Direction::Up, splits)
    }

    /// Set the child Canvasses flowing to the right.
    pub fn flow_right(self, splits: &'a [Split<'a>]) -> Split<'a> {
        self.flow(Direction::Right, splits)
    }

    /// Set the child Canvasses flowing to the left.
    pub fn flow_left(self, splits: &'a [Split<'a>]) -> Split<'a> {
        self.flow(Direction::Left, splits)
    }

    /// Set the padding from the left edge.
    pub fn pad_left(mut self, pad: Scalar) -> Split<'a> {
        self.style.padding.maybe_left = Some(pad);
        self
    }

    /// Set the padding from the right edge.
    pub fn pad_right(mut self, pad: Scalar) -> Split<'a> {
        self.style.padding.maybe_right = Some(pad);
        self
    }

    /// Set the padding from the top edge.
    pub fn pad_top(mut self, pad: Scalar) -> Split<'a> {
        self.style.padding.maybe_top = Some(pad);
        self
    }

    /// Set the padding from the bottom edge.
    pub fn pad_bottom(mut self, pad: Scalar) -> Split<'a> {
        self.style.padding.maybe_bottom = Some(pad);
        self
    }

    /// Set the padding for all edges.
    pub fn pad(self, pad: Scalar) -> Split<'a> {
        self.pad_left(pad).pad_right(pad).pad_top(pad).pad_bottom(pad)
    }

    /// Set the margin from the left edge.
    pub fn margin_left(mut self, pad: Scalar) -> Split<'a> {
        self.style.margin.maybe_left = Some(pad);
        self
    }

    /// Set the margin from the right edge.
    pub fn margin_right(mut self, pad: Scalar) -> Split<'a> {
        self.style.margin.maybe_right = Some(pad);
        self
    }

    /// Set the margin from the top edge.
    pub fn margin_top(mut self, pad: Scalar) -> Split<'a> {
        self.style.margin.maybe_top = Some(pad);
        self
    }

    /// Set the margin from the bottom edge.
    pub fn margin_bottom(mut self, pad: Scalar) -> Split<'a> {
        self.style.margin.maybe_bottom = Some(pad);
        self
    }

    /// Set the margin for all edges.
    pub fn margin(self, pad: Scalar) -> Split<'a> {
        self.margin_left(pad).margin_right(pad).margin_top(pad).margin_bottom(pad)
    }

    /// Store the Canvas and it's children within the `Ui`. Each Canvas can be accessed via it's
    /// unique identifier `CanvasId`.
    pub fn set<C>(self, ui: &mut Ui<C>) {
        let dim = [ui.win_w as f64, ui.win_h as f64];
        self.into_ui(dim, [0.0, 0.0], ui);
    }

    /// Construct a Canvas from a Split.
    fn into_ui<C>(&self, dim: Dimensions, xy: Point, ui: &mut Ui<C>) {
        use elmesque::form::{rect, collage};
        use vecmath::{vec2_add, vec2_sub};

        let Split { id, ref maybe_splits, ref style, .. } = *self;

        let color = style.color(&ui.theme);
        let frame_color = style.frame_color(&ui.theme);
        let frame = style.frame(&ui.theme);
        let pad = style.padding(&ui.theme);
        let mgn = style.margin(&ui.theme);

        let mgn_offset = [(mgn.left - mgn.right), (mgn.bottom - mgn.top)];
        let dim = vec2_sub(dim, [mgn.left + mgn.right, mgn.top + mgn.bottom]);
        let frame_dim = vec2_sub(dim, [frame * 2.0; 2]);
        let pad_offset = [(pad.bottom - pad.top), (pad.left - pad.right)];
        let pad_dim = vec2_sub(frame_dim, [pad.left + pad.right, pad.top + pad.bottom]);

        // Offset xy so that it is in the center of the given margin.
        let xy = vec2_add(xy, mgn_offset);

        if let Some((direction, splits)) = *maybe_splits {
            use Direction::{Up, Down, Left, Right};

            // Offset xy so that it is in the center of the padded area.
            let xy = vec2_add(xy, pad_offset);
            let (stuck_length, num_not_stuck) =
                splits.iter().fold((0.0, splits.len()), |(total, remaining), split| {
                    match split.maybe_length {
                        Some(length) => (total + length, remaining - 1),
                        None => (total, remaining),
                    }
                });

            // Dimensions for Splits that haven't been given a specific length.
            let split_dim = match num_not_stuck {
                0 => [0.0, 0.0],
                _ => match direction {
                    Up   | Down  => {
                        let remaining_height = pad_dim[1] - stuck_length;
                        let height = match remaining_height > 0.0 {
                            true  => remaining_height / num_not_stuck as f64,
                            false => 0.0,
                        };
                        [pad_dim[0], height]
                    },
                    Left | Right => {
                        let remaining_width = pad_dim[0] - stuck_length;
                        let width = match remaining_width > 0.0 {
                            true  => remaining_width / num_not_stuck as f64,
                            false => 0.0
                        };
                        [width, pad_dim[1]]
                    },
                },
            };

            // The length of the previous split.
            let mut prev_length = 0.0;

            // Initialise the `current_xy` at the beginning of the pad_dim.
            let mut current_xy = match direction {
                Down  => [xy[0], xy[1] + pad_dim[1] / 2.0],
                Up    => [xy[0], xy[1] - pad_dim[1] / 2.0],
                Left  => [xy[0] + pad_dim[0] / 2.0, xy[1]],
                Right => [xy[0] - pad_dim[0] / 2.0, xy[1]],
            };

            // Update every split within the Ui.
            for split in splits.iter() {
                let split_dim = match split.maybe_length {
                    Some(len) => match direction {
                        Up   | Down  => [split_dim[0], len],
                        Left | Right => [len, split_dim[1]],
                    },
                    None => split_dim,
                };

                // Shift xy into position for the current split.
                match direction {
                    Down => {
                        current_xy[1] -= split_dim[1] / 2.0 + prev_length / 2.0;
                        prev_length = split_dim[1];
                    },
                    Up   => {
                        current_xy[1] += split_dim[1] / 2.0 + prev_length / 2.0;
                        prev_length = split_dim[1];
                    },
                    Left => {
                        current_xy[0] -= split_dim[0] / 2.0 + prev_length / 2.0;
                        prev_length = split_dim[0];
                    },
                    Right => {
                        current_xy[0] += split_dim[0] / 2.0 + prev_length / 2.0;
                        prev_length = split_dim[0];
                    },
                }

                split.into_ui(split_dim, current_xy, ui);
            }
        }

        let frame_form = rect(dim[0], dim[1]).filled(frame_color);
        let inner_form = rect(frame_dim[0], frame_dim[1]).filled(color);
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(inner_form).into_iter())
            .map(|form| form.shift(xy[0], xy[1]));

        let element = collage(frame_dim[0] as i32, frame_dim[1] as i32, form_chain.collect());

        let widget_area_xy = xy;
        let widget_area_dim = dim;

        ui::update_canvas(ui, id, Kind::Split(State), xy, widget_area_xy,
                          widget_area_dim, pad, Some(element));
    }

}


impl Style {

    /// Construct a default Style.
    pub fn new() -> Style {
        Style {
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_color: None,
            padding: Padding::new(),
            margin: Margin::new(),
        }
    }

    /// Get the color for the Split's Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_canvas_split.as_ref().map(|style| {
            style.maybe_color.unwrap_or(theme.background_color)
        })).unwrap_or(theme.background_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_canvas_split.as_ref().map(|style| {
            style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_canvas_split.as_ref().map(|style| {
            style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the Padding for the Canvas Split.
    pub fn padding(&self, theme: &Theme) -> position::Padding {
        position::Padding {
            top: self.padding.maybe_top.or(theme.maybe_canvas_split.as_ref().map(|style| {
                style.padding.maybe_top.unwrap_or(theme.padding.top)
            })).unwrap_or(theme.padding.top),
            bottom: self.padding.maybe_bottom.or(theme.maybe_canvas_split.as_ref().map(|style| {
                style.padding.maybe_bottom.unwrap_or(theme.padding.bottom)
            })).unwrap_or(theme.padding.bottom),
            left: self.padding.maybe_left.or(theme.maybe_canvas_split.as_ref().map(|style| {
                style.padding.maybe_left.unwrap_or(theme.padding.left)
            })).unwrap_or(theme.padding.left),
            right: self.padding.maybe_right.or(theme.maybe_canvas_split.as_ref().map(|style| {
                style.padding.maybe_right.unwrap_or(theme.padding.right)
            })).unwrap_or(theme.padding.right),
        }
    }

    /// Get the Margin for the Canvas Split.
    pub fn margin(&self, theme: &Theme) -> position::Margin {
        position::Margin {
            top: self.margin.maybe_top.or(theme.maybe_canvas_split.as_ref().map(|style| {
                style.margin.maybe_top.unwrap_or(theme.margin.top)
            })).unwrap_or(theme.margin.top),
            bottom: self.margin.maybe_bottom.or(theme.maybe_canvas_split.as_ref().map(|style| {
                style.margin.maybe_bottom.unwrap_or(theme.margin.bottom)
            })).unwrap_or(theme.margin.bottom),
            left: self.margin.maybe_left.or(theme.maybe_canvas_split.as_ref().map(|style| {
                style.margin.maybe_left.unwrap_or(theme.margin.left)
            })).unwrap_or(theme.margin.left),
            right: self.margin.maybe_right.or(theme.maybe_canvas_split.as_ref().map(|style| {
                style.margin.maybe_right.unwrap_or(theme.margin.right)
            })).unwrap_or(theme.margin.right),
        }
    }

}

impl Padding {
    /// Construct a defualt Padding.
    pub fn new() -> Padding {
        Padding {
            maybe_top: None,
            maybe_bottom: None,
            maybe_left: None,
            maybe_right: None,
        }
    }
}

impl Margin {
    /// Construct a defualt Margin.
    pub fn new() -> Margin {
        Margin {
            maybe_top: None,
            maybe_bottom: None,
            maybe_left: None,
            maybe_right: None,
        }
    }
}


impl<'a> ::color::Colorable for Split<'a> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a> ::frame::Frameable for Split<'a> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}

