## Patch Notes

### Version 1.1.0-beta.1
- New Widgets added
- New Hotreload system via `bevy_asset`
- HTML Bindsystem now works with macros. You can use a bevy system now for events handling
- Widgets have now better designs and more CSS support
- A bunch of examples was added
- Font can now be changed and use weight with `font-weight:` and `font-family;`
- Div widgets are now scrollable in x and y-axis
- UiRegistry was refactored and improved

#### Bug fixes
- Fixed a bug where widgets were not rendering correctly
- Fixed a bug where the HTML bind system was not working properly
- Fixed a bug where the font weight was not being applied correctly
- Fixed a bug where the div widgets were not scrolling properly
- Fixed a bug where the font family was not being applied correctly
- Fixed a bug where the ui-scale was not correct for the slider widget
- Fixed a bug which broke up button, text and checkbox updates