// PLEASE UPDATE THE TYPESCRIPT FILE. script.js is automatically updated when running "tsc" on this folder.
class Message {
    provider_name: string
    room: string
    username: string
    message: string
    msgid: string
    timestamp: number
    badges: Array<Badge>
    emotes: Array<Emote>

    constructor(msg: any) {
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
    provider_tag(): string {
        switch (this.provider_name) {
            case "twitch": return "Tw"
            case "matrix": return "Mx"
        }
        return this.provider_name;
    }
    key() {
        return `${this.timestamp}|${this.provider_name}|${this.msgid}`
    }
}

class Badge {
    name: string
    vid: string
    url: string
    constructor(badge: any) {
        this.name = badge.name;
        this.vid = badge.vid;
        this.url = badge.url;
    }
}

class Emote {
    id: string
    from: number
    to: number
    name: string
    url: string
    constructor(emote: any) {
        this.id = emote.id;
        this.from = emote.from;
        this.to = emote.to;
        this.name = emote.name;
        this.url = emote.url;
    }
}

var messages: Map<string, Message> = new Map();
var last_hash: string = "";
var last_linecount = 0;
var last_html = "";

const simpleHash = (str: string) => {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
        const char = str.charCodeAt(i);
        hash = (hash << 5) - hash + char;
        hash &= hash; // Convert to 32bit integer
    }
    return new Uint32Array([hash])[0].toString(36);
};
const onLoadData = (xhr: XMLHttpRequest) => {
    reqListener(xhr);
}
const onLoadLog = (xhr: XMLHttpRequest) => {
    let lines = reqListener(xhr);
    if (lines < last_linecount) {
        loadData();
    }
    last_linecount = lines;
}

const reqListener = (xhr: XMLHttpRequest): number => {
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
    let toDelete = messages.size - 50;
    if (toDelete > 0) {
        let keys = [...messages.keys()].sort().slice(0, toDelete);
        for (let i in keys) {
            messages.delete(keys[i]);
        }
    }
    updateChat();
    return lines.length;
};
const escapeHtml = (unsafe: string) => {
    return unsafe.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;').replaceAll("'", '&#039;');
}

var stringToColour = function (str: string) {
    var hash = 0;
    for (var i = 0; i < str.length; i++) {
        hash = str.charCodeAt(i) + ((hash << 5) - hash);
    }
    let hue = hash % 360;
    return `hsl(${hue}, 80%, 70%)`;
}

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
            console.log(`missing data for key-${i} ${k}`)
            continue;
        }
        if (msg.timestamp < first_timestamp) {
            continue;
        }
        let badges: string = "";
        for (let i in msg.badges) {
            let badge = msg.badges[i];
            badges += `<img src="${badge.url}" alt="${badge.name}" class="badge">`
        }
        let message = escapeHtml(msg.message);
        for (let i in msg.emotes) {
            // TODO: This code does not cut the emotes as specified and may result in undefined behavior.
            let emote = msg.emotes[i];
            let img = `<img src="${emote.url}" alt="${emote.name}" class="emote">`
            message = message.replaceAll(emote.name, img);
        }
        let color = stringToColour(msg.username);
        let text = `
        <div class="shadow chatmsg chatmsg-${msg.provider_name}">
            <div class="provider provider-${msg.provider_name}">${msg.provider_tag()}@
            </div><div class="badges badges-${msg.provider_name}">${badges}</div><div class="username" style="color: ${color}">${msg.username}
            </div><span class="separator">:</span><div class="message">${message}</div>
        </div>
        `;
        let spacing = (msg.timestamp - last_timestamp) * chat_speed;
        let preftext = "";
        for (let i = 0; i < spacing && i < 10; i++) {
            preftext += `<div class="spacing"></div>`;
        }
        if (preftext != "") {
            text = `<div class="spacing_group">${preftext}</div>` + text;
        }

        chatHTML += text;
        last_timestamp = msg.timestamp;

    }
    let spacing = (timestamp - last_timestamp) * chat_speed;
    let preftext = "";
    for (let i = 0; i < spacing && i < 10; i++) {
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
    } else {
        console.log("unable to find #content")
    }
    last_html = chatHTML;

};

const loadData = () => {
    const req = new XMLHttpRequest();
    req.onload = (response: ProgressEvent<EventTarget>) => {
        onLoadData(req);
    };
    req.open("get", "yarrdb_data.jsonl", true);
    req.send();
};
const loadLog = () => {
    const req = new XMLHttpRequest();
    req.onload = (response: ProgressEvent<EventTarget>) => {
        onLoadLog(req);
    };;
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
window.setInterval(loadLog, 250);
window.setInterval(updateChat, 2 * 1000);