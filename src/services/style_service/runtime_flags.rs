use bevy::prelude::*;

use crate::styles::{BackdropFilter, Style};

/// Cached hints for expensive style post-processing systems.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StyleRuntimeFlags {
    pub(crate) uses_calc: bool,
    pub(crate) uses_background_gradient: bool,
    pub(crate) uses_background_image: bool,
    pub(crate) uses_backdrop_filter: bool,
}

impl StyleRuntimeFlags {
    pub(crate) fn is_empty(self) -> bool {
        !self.uses_calc
            && !self.uses_background_gradient
            && !self.uses_background_image
            && !self.uses_backdrop_filter
    }
}

pub(super) fn style_runtime_flags(style: &Style) -> StyleRuntimeFlags {
    let (uses_background_gradient, uses_background_image) = style
        .background
        .as_ref()
        .map(|background| (background.gradient.is_some(), background.image.is_some()))
        .unwrap_or_default();

    StyleRuntimeFlags {
        uses_calc: style_uses_calc(style),
        uses_background_gradient,
        uses_background_image,
        uses_backdrop_filter: matches!(
            style.backdrop_filter.as_ref(),
            Some(BackdropFilter::Blur(radius)) if *radius > 0.0
        ),
    }
}

pub(super) fn style_uses_calc(style: &Style) -> bool {
    style.width_calc.is_some()
        || style.min_width_calc.is_some()
        || style.max_width_calc.is_some()
        || style.height_calc.is_some()
        || style.min_height_calc.is_some()
        || style.max_height_calc.is_some()
        || style.left_calc.is_some()
        || style.right_calc.is_some()
        || style.top_calc.is_some()
        || style.bottom_calc.is_some()
        || style.flex_basis_calc.is_some()
        || style.gap_calc.is_some()
        || style.row_gap_calc.is_some()
        || style.column_gap_calc.is_some()
}

pub(super) fn sync_style_runtime_flags(commands: &mut Commands, entity: Entity, style: &Style) {
    let flags = style_runtime_flags(style);
    if flags.is_empty() {
        commands.entity(entity).remove::<StyleRuntimeFlags>();
    } else {
        commands.entity(entity).insert(flags);
    }
}
