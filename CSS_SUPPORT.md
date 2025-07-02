# Css Support

Here is an overview of the currently supported CSS rules.
Please note that not all rules work exactly like in standard CSS, but most of them follow the standard specifications.
This document is meant to help you when designing your components.
If you have any questions, feel free to contact me on Discord: **`exepta`**.

## Size Rules

Currently, only the units pixels (px) and percent (%) are supported.
If you use any other units, they will be converted to `0px`!

#### Width / Height
```css
div {
    width: 100px;
    height: 100%;
}
```

#### Min-Width / Min-Height
```css
div {
    min-width: 10%;
    min-height: 500px;
}
```

#### Max-Width / Max-Height
```css
div {
    max-width: 75%;
    max-height: 100%;
}
```

The CSS attributes left, right, top, and bottom are also supported.
The same works for right top and bottom.
#### Left Positive
```css
div {
    left: 100px; /* % */
}
```

#### Left Negative
```css
div {
    left: -100px; /* % */
}
```

Next up are padding and margin.
The example is shown only once, as the behavior is the same for both properties.

Please note the order of values:

1. `all`
2. `horizontal vertical`
3. `left right top`
4. `left right top bottom`

#### Padding / Margin
```css
div {
    padding: 2px;
    padding-left: 3px;
    padding-right: 4px;
    padding-bottom: 1px;
}
```
___
## Border Rules

Unfortunately, at this point in time, the border can only have a single color.
This is because Bevy uses the BorderColor component, which only supports one fixed value.
Border only supports pixels!


The following color formats are supported:

- `rgb`
- `rgba`
- `hex (e.g. #ffffff)`
- `named (CSS standard color names)`

#### Border
```css
div {
    border: 2px;
    border: 4px green;
}
```

#### Border Direction
```css
div {
    border-left: 1px;
    border-right: 10px;
    border-bottom: 10px;
    border-top: 0;
}
```

#### Border-Radius
```css
div {
    /* top-left top-right bottom-left bottom-right */
    border-radius: 20px;
}
```

#### Border-Color
```css
div {
    border-color: #ff926a;
}
```

#### Border-Width
```css
div {
    /* left right top bottom */
    border-width: 10px 10px 20px 20px;
}
```

___
## Positions

Here are the available layout options.
Currently, only flex and block are supported.
Grid is not yet implemented and will have no effect.

Please note that layout behavior follows Bevy’s Display system.
The only supported positioning modes are absolute and relative.

#### Display
```css
div {
   display: flex;
}
```

#### Position
```css
div {
   position: relative;
}
```

#### Justify-Content
```css
div {
   justify-content: center;
}
```

Support:
- Start
- Flex-Start
- End
- Flex-End
- Center
- Space-Between
- Space-Around
- Space-Evenly
- Stretch

#### Align-Items
```css
div {
   align-items: center;
}
```

Support:
- Start
- Flex-Start
- End
- Flex-End
- Center
- Baseline
- Stretch

#### Flex-Direction
```css
div {
   flex-direction: column;
}
```

Support:
- Row
- Column
- Row-Reverse
- Column-Reverse

#### Flex-Grow
```css
div {
   flex-grow: 1;
}
```

#### Gap
```css
div {
   gap: 10px;
}
```

Gap support px and %. It will detect which direction you use for
the bevy `gap_row` and `gap_column` usage!

___
## Others

This section covers all elements that have not yet been categorized.

#### Font-Size
```css
div {
   font-size: 20px;
}
```

Currently only supports px!

#### Color
```css
div {
   color: #303033;
}
```

#### Background-Color
```css
div {
   background-color: #6a00ff;
}
```

Color Supports:
- `rgb`
- `rgba`
- `hex (e.g. #ffffff)`
- `named (CSS standard color names)`

#### Background-Image
```css
div {
   background-image: url("images/test-bg.png");
}
```

Note that bevy use as default the assets folder in your project! Don't write the
complete path. If you wish to change the folder path than make sure that your AssetServer
have this path set!

#### Background
```css
div {
   background: #6a00ff;
}
```

the same as `background-color`
plus it can take images like `background-image`

#### Box-Shadow
```css
div {
    /* x, y, blur, spread and color */
    box-shadow: 2px 2px 10px 20px black;
}
```

#### Overflow
```css
div {
    overflow: hidden;
}
```

This hide the x and y overflow of the current object

#### Overflow-x
```css
div {
    overflow-x: visible;
}
```

#### Overflow-y
```css
div {
    overflow-y: scroll;
}
```

#### Text-Wrap
```css
div {
    text-wrap: nowrap;
}
```

Supported for Bevy node:
- nowrap - LineBreak::NoWrap
- wrap - LineBreak::WordOrCharacter
- stable - LineBreak::WordOrCharacter
- pretty - LineBreak::WordBoundary
- balance - LineBreak::WordBoundary
- unset - LineBreak::AnyCharacter

#### Z-Index
```css
div {
    z-index: 999;
}
```

this value can be negative too!

#### Pointer-Events
```css
div {
    pointer-events: none;
}
```
The value `none` is bevy `Pickable::Ignore` all other ar `Pickable::default()`