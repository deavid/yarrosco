var messages = new Map();
var last_hash = "";
var last_linecount = 0;

const simpleHash = str => {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
        const char = str.charCodeAt(i);
        hash = (hash << 5) - hash + char;
        hash &= hash; // Convert to 32bit integer
    }
    return new Uint32Array([hash])[0].toString(36);
};
const onLoadData = response => {
    reqListener(response);
}
const onLoadLog = response => {
    let lines = reqListener(response);
    if (lines < last_linecount) {
        loadData();
    }
    last_linecount = lines;
}

const reqListener = response => {
    let resp = response.currentTarget.response;

    let h = simpleHash(resp);
    if (h == last_hash) {
        return;
    }
    last_hash = h;
    let lines = resp.split("\n");
    for (const n in lines) {
        let line = lines[n].trim();
        if (line) {
            let obj = JSON.parse(line);
            let msg = obj.Message;
            if (msg) {
                console.log(msg);
                key = `${msg.timestamp}|${msg.provider_name}|${msg.msgid}`
                messages.set(key, msg);
            }
        }
    }
    let toDelete = messages.size - 100;
    if (toDelete > 0) {
        let keys = [...messages.keys()].sort().slice(0, toDelete);
        for (i in keys) {
            messages.delete(keys[i]);
        }
    }
    updateChat();
    return lines.length;
};
const escapeHtml = (unsafe) => {
    return unsafe.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;').replaceAll("'", '&#039;');
}

var stringToColour = function (str) {
    var hash = 0;
    for (var i = 0; i < str.length; i++) {
        hash = str.charCodeAt(i) + ((hash << 5) - hash);
    }
    hue = hash % 360;
    return `hsl(${hue}, 80%, 70%)`;
}

const updateChat = () => {
    let chatHTML = "";
    let keys = [...messages.keys()].sort();
    let last_timestap = 0;
    for (i in keys) {
        let k = keys[i];
        let msg = messages.get(k);
        let color = stringToColour(msg.username);
        // msg.provider_name
        // msg.room
        // msg.message
        // msg.username
        // msg.timestamp
        let text = `
        <div class="shadow chatmsg chatmsg-${msg.provider_name}">
            <div class="provider provider-${msg.provider_name}">${msg.provider_name}
            </div><div class="username" style="color: ${color}">${msg.username}
            </div><span class="separator">:</span><div class="message">${escapeHtml(msg.message)}</div>
        </div>
        `;

        chatHTML += text;

    }

    const content = document.getElementById("content");
    content.innerHTML = chatHTML;

};

const loadData = () => {
    const req = new XMLHttpRequest();
    req.onload = onLoadData;
    req.open("get", "yarrdb_data.jsonl", true);
    req.send();
};
const loadLog = () => {
    const req = new XMLHttpRequest();
    req.onload = onLoadLog;
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