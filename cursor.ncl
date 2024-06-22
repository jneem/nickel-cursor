let
  rotate = fun deg c => c & { rotation_degrees = deg }
in
let
  hex_digits: {_: Number} = { "0" = 0, "1" = 1, "2" = 2, "3" = 3, "4" = 4, "5" = 5, "6" = 6, "7" = 7, "8" = 8, "9" = 9, a = 10, b = 11, c = 12, d = 13, e = 14, f = 15 }
in
let
  hex_digit: String -> Number = fun s => std.record.get s hex_digits
in
let
  hex: String -> Number = fun s =>
    s |> std.string.characters
      |> std.array.map hex_digit
      |> std.array.fold_left (fun acc x => acc * 16 + x) 0
in
{
  Point = {
    x | Number,
    y | Number,
  },

  Color
  | doc "An RGBA color. Each entry should be between 0 and 255 inclusive."
  = {
    r | Number,
    g | Number,
    b | Number,
    a | Number,
  },

  ColorString
    | doc m%"A contract for strings that can represent RGB or RGBA colors in HTML-style hex.
          This should have 6 or 8 characters, each of which is a lower-case hex digit."%
    = fun label value =>
      if std.string.length value == 8 || std.string.length value == 6 then
        if std.string.characters value |> std.array.all (fun s => std.record.has_field s hex_digits) then
          value
        else
          std.contract.blame_with_message "only (lower-case) hex digits are allowed" label
      else
        std.contract.blame_with_message "color strings must have 6 or 8 characters" label,

  color.from_hex: String -> { r: Number, g: Number, b: Number, a: Number }
    | doc m%"
        Convert a hex string to a color.

        # Examples

        ```nickel
        color.from_hex "ff0000" =>
          { r: 255, g: 0, b: 0, a: 255 }

        color.from_hex "ff0000dd" =>
          { r: 255, g: 0, b: 0, a: 221 }
        ```
      "%
    = fun s =>
      let s = s | ColorString in
      {
         r = hex (std.string.substring 0 2 s),
         g = hex (std.string.substring 2 4 s),
         b = hex (std.string.substring 4 6 s),
         a = if (std.string.length s) == 8 then hex (std.string.substring 6 8 s) else 255,
      },

  Cursor = {
    paths
      | Array String
      | doc "A path, in svg format. The path should assume a view box from (0, 0) to (256, 256)."
      ,

    hot
      | Point
      | doc "The cursor's hot point, assuming a 256x256 image. The top-left corner is (0, 0), and positive y points down."
      ,

    rotation_degrees
      | Number
      | doc "If set, the paths and the hot point will be rotated by this amount."
      | default
      = 0
      ,
  },

  CursorTheme = {
    name | String,
    cursors | CursorSet,
    style | Style,
    links | default
    | doc "A collection of cursor name aliases. I don't know of a canonical reference, so just copied from Adwaita."
    = {
      "00008160000006810000408080010102" = "sb_v_double_arrow",
      "028006030e0e7ebffc7f7070c0600140" = "sb_h_double_arrow",
      "08e8e1c95fe2fc01f976f1e063a24ccd" = "left_ptr_watch",
      "14fef782d02440884392942c11205230" = "sb_h_double_arrow",
      "2870a09082c103050810ffdffffe0204" = "sb_v_double_arrow",
      "3ecb610c1bf2410f44200f48c40d3599" = "left_ptr_watch",
      "9d800788f1b08800ae810202380a0822" = "hand2",
      "arrow" = "left_ptr",
      c7088f0f3e6c8088236ef8e1e3e70000 = "bd_double_arrow",
      col-resize = "sb_h_double_arrow",
      "default" = "left_ptr",
      double_arrow = "sb_v_double_arrow",
      e29285e634086352946a0e7090d73106 = "hand2",
      ew-resize = "sb_h_double_arrow",
      fcf1c3c7cd4491d801f1e1c78f100000 = "fd_double_arrow",
      grab = "hand1",
      ibeam = "xterm",
      n-resize = "top_side",
      ne-resize = "top_right_corner",
      nesw-resize = "fd_double_arrow",
      ns-resize = "sb_v_double_arrow",
      nw-resize = "top_left_corner",
      nwse-resize = "bd_double_arrow",
      openhand = "left_ptr",
      pointer = "hand2",
      pointing_hand = "hand2",
      progress = "left_ptr_watch",
      row-resize = "sb_v_double_arrow",
      s-resize = "bottom_side",
      se-resize = "bottom_right_corner",
      size_bdiag = "fd_double_arrow",
      size_fdiag = "bd_double_arrow",
      size_hor = "sb_h_double_arrow",
      size_ver = "sb_v_double_arrow",
      split_h = "sb_h_double_arrow",
      split_v = "sb_v_double_arrow",
      sw-resize = "bottom_left_corner",
      text = "xterm",
      top_left_arrow = "left_ptr",
      up_arrow = "sb_up_arrow",
      b_double_arrow = "sb_v_double_arrow",
      w-resize = "left_side",
      wait = "watch",
    },
  },

  Style
  | doc "Options for styling cursors"
  = {
    sizes | Array Number | doc "Cursor size in pixels. (All cursor images will be square.)",
    fill_color | Color,
    stroke_width | Number,
    stroke_color | Color,
  },

  CursorSet
  | doc m%"A set of cursors with standardized names.

    You do not need to define all of them; many default to rotated copies of other cursors.
    "%
  = {
    left_ptr | Cursor | doc "The default cursor",
    left_ptr_watch | Cursor | doc "The default cursor, but busy",
    hand1 | Cursor | doc "Open hand",
    hand2 | Cursor | doc "Pointing hand",
    xterm | Cursor | doc "Text caret",

    top_left_corner | Cursor | doc "Arrow pointing to the top left",
    top_right_corner | Cursor | doc "Arrow pointing to the top right" | default = rotate 90 top_left_corner,
    bottom_right_corner | Cursor | doc "Arrow pointing to the bottom right" | default = rotate 180 top_left_corner,
    bottom_left_corner | Cursor | doc "Arrow pointing to the bottom left" | default = rotate 270 top_left_corner,

    top_side | Cursor,
    right_side | Cursor | default = rotate 90 top_side,
    bottom_side | Cursor | default = rotate 180 top_side,
    left_side | Cursor | default = rotate 270 top_side,

    # In Adwaita, these aren't links; they're a bit different. We default them to be aliases, so that the lazy
    # cursor designer can skip them
    watch | Cursor | default = left_ptr_watch,
    sb_up_arrow | Cursor | default = top_side,
    sb_right_arrow | Cursor | default = right_side,
    sb_down_arrow | Cursor | default = bottom_side,
    sb_left_arrow | Cursor | default = left_side,

    bd_double_arrow
      | Cursor
      | doc "Double-arrow pointing to the bottom-left and top-right"
      ,

    fd_double_arrow
      | Cursor
      | doc "Double-arrow pointing to the top-left and bottom-right"
      | default = rotate 90 bd_double_arrow
      ,

    sb_h_double_arrow
      | Cursor
      | doc "Double-arrow pointing left and right"
      ,
    sb_v_double_arrow
      | Cursor
      | doc "Double-arrow pointing up and down"
      | default = rotate 90 sb_h_double_arrow
      ,
  },
}
