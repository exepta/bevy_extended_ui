use bevy::prelude::*;
use crate::styling::convert::CssFile;
use crate::styling::system::WidgetStyle;

pub struct CssService;

impl Plugin for CssService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_css_conventions);
    }
}

fn update_css_conventions(query: Query<(Entity, &CssFile)>, mut widget_query: Query<&mut WidgetStyle>) {
    for (entity, file) in query.iter() {
        if let Ok(mut widget_style) = widget_query.get_mut(entity) {
            if widget_style.css_path.eq(&file.0.to_string()) {
                continue;
            }
            widget_style.css_path = file.0.to_string();
        }
    }
}
