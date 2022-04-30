"use strict";
// PLEASE UPDATE THE TYPESCRIPT FILE. script.js is automatically updated when running "tsc" on this folder.
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
    }
    provider_tag() {
        switch (this.provider_name) {
            case "twitch": return "Tw";
            case "matrix": return "Mx";
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
var messages = new Map();
var last_hash = "";
var last_linecount = 0;
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
    let toDelete = messages.size - 100;
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
    const timestamp = Date.now() / 1000;
    // const max_message_age = 60;
    const max_message_age = 60 * 60 * 8;
    let last_timestamp = timestamp - max_message_age;
    let first_timestamp = timestamp - max_message_age;
    const chat_speed = 1 / 60.0;
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
        let color = stringToColour(msg.username);
        let text = `
        <div class="shadow chatmsg chatmsg-${msg.provider_name}">
            <div class="provider provider-${msg.provider_name}">${msg.provider_tag()}@
            </div><div class="badges badges-${msg.provider_name}">${badges}</div><div class="username" style="color: ${color}">${msg.username}
            </div><span class="separator">:</span><div class="message">${escapeHtml(msg.message)}</div>
        </div>
        `;
        let spacing = (msg.timestamp - last_timestamp) * chat_speed;
        for (let i = 0; i < spacing && i < 10; i++) {
            let prefix = `<div class="spacing"></div>`;
            text = prefix + text;
        }
        chatHTML += text;
        last_timestamp = msg.timestamp;
    }
    let spacing = (timestamp - last_timestamp) * chat_speed;
    for (let i = 0; i < spacing && i < 10; i++) {
        let prefix = `<div class="spacing"></div>`;
        chatHTML += prefix;
    }
    const content = document.getElementById("content");
    if (content) {
        content.innerHTML = chatHTML;
    }
    else {
        console.log("unable to find #content");
    }
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
    req.send();
};
window.onload = () => {
    loadData();
    loadLog();
};
window.setInterval(loadLog, 50);
window.setInterval(updateChat, 2 * 1000);
