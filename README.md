# bevy_extended_ui
___
[![license](https://img.shields.io/badge/license-Apache-blue.svg)](./LICENSE)


Since I've been writing a game in the [_Bevy_](https://bevyengine.org/) engine lately, 
I created this crate. In my game, 
I need more complex UI elements that aren't currently supported by Bevy. 
These include sliders, choice boxes, check boxes, radio buttons, and so on. 
So I set about implementing these elements using the standard bevy_ui system. 
I'd like to make this project available to you so that you can use elements other 
than just nodes, buttons, or images. If you're missing a widget and know how 
to create it, it would be great if you could add it. 
Otherwise, feel free to create a ticket.

> *Note:* This project is currently under construction and not suitable for large projects!.

### Todo:
> Here's my current to-do list, which I'm working on. Note that this list isn't final. If you have any ideas, open a ticket and we'll talk about it!

- [x] Create System for handle Ui States
- [x] Create Style handling for:
  - [x] Base
  - [x] Hover
  - [x] Focus
- [ ] Create Widget Div
- [ ] Create Widget Button
- [ ] Create Widget InputField
  - [ ] Type Text
  - [ ] Type Number
  - [ ] Type Password
- [ ] Create Widget Slider
- [ ] Create Widget CheckBox
- [ ] Create Widget ChoiceBox
- [ ] Create Widget Radio Button
- [ ] Create Widget Toggle Button

| `Bevy` version | `bevy_extended_ui` version |
|----------------|----------------------------|
| 0.15.3         | 0.1.0                      |