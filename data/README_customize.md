Customizing the HTML App
==========================

Sadly we don't have a proper config support here. If you want to personalize 
this you'll probably need to have typescript installed.

If you change the `script.ts` file you need to run `tsc` (the Typescript to 
Javascript compiler) from this folder to update `script.js`. Same goes for `config.ts`.

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

## Config.ts variations

Most useful variables for tweaking are in `config.ts`.

```ts
const CONFIG = {
    // how many messages at most will be held on memory and displayed.
    MAX_MESSAGES: 50,
    // How many seconds will a message be displayed before removing.
    MAX_MESSAGE_AGE: 60 * 60 * 8,
    // How fast the chat will move up, in "spacers" per second. One spacer is 2px by default.
    CHAT_SPEED: 1 / 60.0,
    // Maximum amount of spacers to add (sets the maximum margin between messages).
    MAX_SPACERS: 10,
    // Time between queries to yarrosco's DB to check new messages (milliseconds).
    DB_POLL_RATE_MS: 250,
    // Time between chat updates - basically to implement the CHAT_SPEED.
    CHAT_UPDATE_RATE_MS: 2 * 1000,
    // Define how to display the different providers on-screen
    PROVIDER_TAG_MAP: new Map([
        ["twitch", "Tw@"],
        ["matrix", "Mx@"],
    ]),
}
```


### MAX_MESSAGES

Limits how many messages can be held in memory, and at the same time, how many
can be displayed at once at any time.

Setting this too large (>10000) may cause the program to run slower than it needs to.

If you only want to see the last 3 messages:

    MAX_MESSAGES: 3,

If you only want to have as much as it fits, just put a large value:

    MAX_MESSAGES: 200,

Take into account that Yarrosco mostly holds 100 messages at any time. So 
increasing this to a value greater than 100 might not make full effect at all 
times.


### MAX_MESSAGE_AGE

Makes chats disappear after a certain amount of seconds.

`MAX_MESSAGE_AGE` needs to be at least 2x higher than `CHAT_UPDATE_RATE_MS`
to ensure the messages are removed roughly at the right time.

If you want your messages to be "permanent", you can set this to several hours:

    MAX_MESSAGE_AGE: 60 * 60 * 8,

If you want the messages to disappear after 10 seconds:

    MAX_MESSAGE_AGE: 10,

> **WARNING:** This setting relies on your computer having the time properly set, 
> following an NTP server. If your computer's clock drifts, the messages may 
> disappear too soon, too late, or never show up at all.

> **NOTE:** How old is a message is defined by comparing your browser's time vs
> the time the server (Twitch, Matrix, ...) reports that the message was sent.
> It does not represent how much time was this message displayed.

### CHAT_SPEED

This setting enables a "self-scrolling chat", where the chat messages scroll up
slowly over time. This creates visual gaps that are cues for the viewer that 
long time has passed since the last message.

This can also be used to make messages disappear over time in a different way,
as they will be too far up in the screen to read.

This is defined in "spacers per second", where one spacer is usually 2px in height.

The app will send a `<div class="spacer">` for each spacer event. In the CSS
it is defined the height of the spacer.

> **WARNING:** Prefer to use decimal points on all numbers here to enforce 
> floating point math.

If you don't want this feature, set to zero to disable:

    CHAT_SPEED: 0.0,

If you want it to slowly move up:

    CHAT_SPEED: 1 / 60.0,

If you want to move up quickly:

    CHAT_SPEED: 1 / 2.0,

If you set this value too high, you might need to change `CHAT_UPDATE_RATE_MS`
to a low value to get a smooth animation:

    CHAT_UPDATE_RATE_MS: 500,

### MAX_SPACERS

Defines the maximum amount of spaces added by `CHAT_SPEED`, so this can set an
upper limit on how big these spaces may get.

If you want messages to basically scroll outside of the window:

    MAX_SPACERS: 1000,

Instead, if you just want a small indicator:

    MAX_SPACERS: 5,

Or, if you just want two states, with or without spacing:

    MAX_SPACERS: 1,

**WARNING:** The browser doesn't like when there are too many messages rendered
and thousands of spaces between each one. If you set this too high, you might 
want to limit either `MAX_MESSAGE_AGE` or `MAX_MESSAGES` to prevent crashes or
high CPU usage.


### DB_POLL_RATE_MS

Controls how fast to check for messages. Sets the delay between requests in 
millisecons.

Increasing this value will reduce the resources used by the browser, but the 
new chats will take a bit more time to appear:

    DB_POLL_RATE_MS: 2000,

Decreasing it too much can overwhelm the browser, filesystem and the web server.
Values below 100 are not recommended.

### CHAT_UPDATE_RATE_MS

This controls manual refreshes when no new messages arrive. This setting affects
the `CHAT_SPEED` and `MAX_MESSAGE_AGE`.

If you want fast reactivity and smooth animations, you'll need to set this
value to something small:

    CHAT_UPDATE_RATE_MS: 500,

But this will add CPU consumption as it has to render in memory each time.

Setting it too high may make `CHAT_SPEED` and `MAX_MESSAGE_AGE` feel clunky:

    CHAT_UPDATE_RATE_MS: 10 * 1000,

But if you're not using those, or at very low speeds, this will work nice and
save CPU.

### PROVIDER_TAG_MAP

Defines how to display the chat provider, which text to associate.

It supports HTML, so you can do:

    PROVIDER_TAG_MAP: new Map([
        ["twitch", "<img src='twitch.png'>"],
        ["matrix", "<img src='matrix.png'>"],
    ]),

But of course, you have to provide the images, or the URL to a remote server
that has them.