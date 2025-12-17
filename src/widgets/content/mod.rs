use crate::widgets::content::divider::DividerWidget;
use crate::widgets::content::headline::HeadlineWidget;
use crate::widgets::content::image::ImageWidget;
use crate::widgets::content::paragraph::ParagraphWidget;
use bevy::prelude::*;

pub mod divider;
pub mod headline;
pub mod image;
pub mod paragraph;

pub struct ExtendedContentWidgets;

impl Plugin for ExtendedContentWidgets {
    fn build(&self, app: &mut App) {
        app.add_plugins((DividerWidget, HeadlineWidget, ImageWidget, ParagraphWidget));
    }
}
