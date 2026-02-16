use crate::widgets::content::divider::DividerWidget;
use crate::widgets::content::headline::HeadlineWidget;
use crate::widgets::content::image::ImageWidget;
use crate::widgets::content::paragraph::ParagraphWidget;
use crate::widgets::content::tooltip::ToolTipWidget;
use bevy::prelude::*;

pub mod divider;
pub mod headline;
pub mod image;
pub mod paragraph;
pub mod tooltip;

/// Plugin that registers content-oriented widgets.
pub struct ExtendedContentWidgets;

impl Plugin for ExtendedContentWidgets {
    /// Adds content widget plugins.
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DividerWidget,
            HeadlineWidget,
            ImageWidget,
            ParagraphWidget,
            ToolTipWidget,
        ));
    }
}
