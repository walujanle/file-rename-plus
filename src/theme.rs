// Design system tokens for consistent styling

use iced::Color;

// Font sizes
pub const FONT_SM: f32 = 13.0;
pub const FONT_MD: f32 = 14.0;
pub const FONT_LG: f32 = 16.0;
pub const FONT_XL: f32 = 24.0;

// Spacing
pub const SPACING_XS: u16 = 2;
pub const SPACING_SM: u16 = 5;
pub const SPACING_MD: u16 = 10;
pub const SPACING_LG: u16 = 20;

// Heights
pub const LIST_HEIGHT: f32 = 300.0;

// Colors for dark mode
pub const COLOR_ERROR: Color = Color::from_rgb(1.0, 0.4, 0.4);
pub const COLOR_SUCCESS: Color = Color::from_rgb(0.3, 1.0, 0.5);
pub const COLOR_INFO: Color = Color::from_rgb(0.3, 0.8, 1.0);
pub const COLOR_MUTED_DARK: Color = Color::from_rgb(0.5, 0.5, 0.5);
pub const COLOR_CONFLICT: Color = Color::from_rgb(1.0, 0.3, 0.3);

// Input limits
pub const MAX_PATTERN_LENGTH: usize = 1024;
pub const MAX_TEMPLATE_LENGTH: usize = 256;
pub const MAX_FILES: usize = 10000;

// Window
pub const WINDOW_WIDTH: f32 = 900.0;
pub const WINDOW_HEIGHT: f32 = 650.0;
