"use strict";
// Configurable parameters:
Object.defineProperty(exports, "__esModule", { value: true });
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
};
exports.default = CONFIG;
