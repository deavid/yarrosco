Rendering to PNG
================


Having a browser and capturing with OBS the browser is a bit cumbersome.

If OBS Browser is not available (it has bugs on Gnome that makes it crash), it
can be a bit inconvenient to run a browser and capture the output in OBS.

Rendering to PNG and showing this on OBS might be easier.

This attempts to use a headless google chrome to perform this task.

Problems with this approach
---------------------------

* Around 200ms to render an average page. It might consume too much CPU on some systems.
* We need to wait for page load and synchronize properly which might not be trivial.
* The background is not transparent when rendered - even after changing the CSS.
    * Google Chrome has a way to fix this, but the Rust library doesn't seem to have this implemented.
