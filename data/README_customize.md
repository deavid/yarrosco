Customizing the HTML App
==========================

Sadly we don't have config support here. If you want to personalize this you'll
probably need to have typescript installed.

If you change the `script.ts` file you need to run `tsc` (the Typescript to 
Javascript compiler) from this folder to update `script.js`.

You can also run `tsc -w` instead and it will keep updating the file in real 
time as you save.

## CSS Styles

The file `styles_base.css` is considered to be the "minimum" to have and it
has sensible defaults.

On top of this, `styles_yarr1.css` has the CSS responsible of the main design
of Yarrosco's chat.

You can, for instance, duplicate this file as `styles_yarr2.css`, and then
change the appropriate line in `yarrosco_chat.html` to reference your file instead:

    <link rel="stylesheet" href="styles_yarr1.css">

## Script.ts variations

TBD
