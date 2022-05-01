"use strict";
// TODO: This does not work -- seems we need a full node.js project for these things...
// import CONFIG from "./config";
// Configurable parameters:
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
class Message {
    constructor(msg) {
        this.provider_name = msg.provider_name;
        this.room = msg.room;
        this.username = msg.username;
        this.message = msg.message;
        this.msgid = msg.msgid;
        this.timestamp = msg.timestamp;
        this.badges = new Array();
        for (let b in msg.badges) {
            let badge = new Badge(msg.badges[b]);
            this.badges.push(badge);
        }
        this.emotes = new Array();
        for (let e in msg.emotes) {
            let emote = new Emote(msg.emotes[e]);
            this.emotes.push(emote);
        }
    }
    provider_tag() {
        let tag = CONFIG.PROVIDER_TAG_MAP.get(this.provider_name);
        if (tag) {
            return tag;
        }
        return this.provider_name;
    }
    key() {
        return `${this.timestamp}|${this.provider_name}|${this.msgid}`;
    }
}
class Badge {
    constructor(badge) {
        this.name = badge.name;
        this.vid = badge.vid;
        this.url = badge.url;
    }
}
class Emote {
    constructor(emote) {
        this.id = emote.id;
        this.from = emote.from;
        this.to = emote.to;
        this.name = emote.name;
        this.url = emote.url;
    }
}
var messages = new Map();
var last_hash = "";
var last_linecount = 0;
var last_html = "";
var last_update_time = 0;
var last_load_time = 0;
const simpleHash = (str) => {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
        const char = str.charCodeAt(i);
        hash = (hash << 5) - hash + char;
        hash &= hash; // Convert to 32bit integer
    }
    return new Uint32Array([hash])[0].toString(36);
};
const onLoadData = (xhr) => {
    reqListener(xhr);
};
const onLoadLog = (xhr) => {
    const timestamp_ms = Date.now();
    if (timestamp_ms - last_load_time < CONFIG.DB_POLL_RATE_MS / 2) {
        // Prevent the browser from hibernating all the calls and sending them all at once when it wakes up.
        return;
    }
    last_load_time = timestamp_ms;
    let lines = reqListener(xhr);
    if (lines < last_linecount) {
        loadData();
    }
    last_linecount = lines;
};
const reqListener = (xhr) => {
    let resp = xhr.responseText;
    let h = simpleHash(resp);
    if (h == last_hash) {
        return last_linecount;
    }
    last_hash = h;
    let lines = resp.split("\n");
    for (const n in lines) {
        let line = lines[n].trim();
        if (line) {
            let obj = JSON.parse(line);
            if (obj.Message) {
                let msg = new Message(obj.Message);
                let key = msg.key();
                if (!messages.has(key)) {
                    console.log(`#${msg.provider_name}::${msg.username}> ${msg.message}`);
                    messages.set(key, msg);
                }
            }
        }
    }
    let toDelete = messages.size - CONFIG.MAX_MESSAGES;
    if (toDelete > 0) {
        let keys = [...messages.keys()].sort().slice(0, toDelete);
        for (let i in keys) {
            messages.delete(keys[i]);
        }
    }
    updateChat();
    return lines.length;
};
const escapeHtml = (unsafe) => {
    return unsafe.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;').replaceAll("'", '&#039;');
};
var stringToColour = function (str) {
    var hash = 0;
    for (var i = 0; i < str.length; i++) {
        hash = str.charCodeAt(i) + ((hash << 5) - hash);
    }
    let hue = hash % 360;
    return `hsl(${hue}, 80%, 70%)`;
};
const updateChat = () => {
    let chatHTML = "";
    let keys = [...messages.keys()].sort();
    const timestamp_ms = Date.now();
    const timestamp = timestamp_ms / 1000;
    if (timestamp_ms - last_update_time < CONFIG.DB_POLL_RATE_MS / 2) {
        // Prevent the browser from hibernating all the calls and sending them all at once when it wakes up.
        return;
    }
    last_update_time = timestamp_ms;
    let last_timestamp = timestamp - CONFIG.MAX_MESSAGE_AGE;
    let first_timestamp = timestamp - CONFIG.MAX_MESSAGE_AGE;
    for (let i in keys) {
        let k = keys[i];
        let msg = messages.get(k);
        if (!msg) {
            console.log(`missing data for key-${i} ${k}`);
            continue;
        }
        if (msg.timestamp < first_timestamp) {
            continue;
        }
        let badges = "";
        for (let i in msg.badges) {
            let badge = msg.badges[i];
            badges += `<img src="${badge.url}" alt="${badge.name}" class="badge">`;
        }
        let message = escapeHtml(msg.message);
        for (let i in msg.emotes) {
            // TODO: This code does not cut the emotes as specified and may result in undefined behavior.
            let emote = msg.emotes[i];
            let img = `<img src="${emote.url}" alt="${emote.name}" class="emote">`;
            message = message.replaceAll(emote.name, img);
        }
        let color = stringToColour(msg.username);
        let text = `
        <div class="shadow chatmsg chatmsg-${msg.provider_name}">
            <div class="provider provider-${msg.provider_name}">${msg.provider_tag()}
            </div><div class="badges badges-${msg.provider_name}">${badges}</div><div class="username" style="color: ${color}">${msg.username}
            </div><span class="separator">:</span><div class="message">${message}</div>
        </div>
        `;
        let spacing = (msg.timestamp - last_timestamp) * CONFIG.CHAT_SPEED;
        let preftext = "";
        for (let i = 0; i < spacing && i < CONFIG.MAX_SPACERS; i++) {
            preftext += `<div class="spacing"></div>`;
        }
        if (preftext != "") {
            text = `<div class="spacing_group">${preftext}</div>` + text;
        }
        chatHTML += text;
        last_timestamp = msg.timestamp;
    }
    let spacing = (timestamp - last_timestamp) * CONFIG.CHAT_SPEED;
    let preftext = "";
    for (let i = 0; i < spacing && i < CONFIG.MAX_SPACERS; i++) {
        preftext += `<div class="spacing"></div>`;
    }
    if (preftext != "") {
        chatHTML += `<div class="spacing_group">${preftext}</div>`;
    }
    if (last_html == chatHTML) {
        return;
    }
    const content = document.getElementById("content");
    if (content) {
        content.innerHTML = chatHTML;
    }
    else {
        console.log("unable to find #content");
    }
    last_html = chatHTML;
};
const loadData = () => {
    const req = new XMLHttpRequest();
    req.onload = (response) => {
        onLoadData(req);
    };
    req.open("get", "yarrdb_data.jsonl", true);
    req.send();
};
const loadLog = () => {
    const req = new XMLHttpRequest();
    req.onload = (response) => {
        onLoadLog(req);
    };
    ;
    // TODO: Sometimes messages take time to appear... ??
    // .. ok, the problem is in the web server that it returns same E-Tag or modified dates.
    // .. we probably need a proper way to send data from a web backend.
    // .. this happens because the file is never closed until it checkpoints.
    req.open("get", "yarrdb_log.jsonl?v=" + Math.random(), true);
    // TODO: Also we froze firefox after a few hours of working. We need to debug this.
    // .. this seems because we keep updating the HTML in the background and Firefox delays
    // .. these until it wakes up. Or JS itself might be stopped.
    req.send();
};
window.onload = () => {
    loadData();
    loadLog();
};
window.setInterval(loadLog, CONFIG.DB_POLL_RATE_MS);
window.setInterval(updateChat, 2 * CONFIG.CHAT_UPDATE_RATE_MS);
