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

const updateChat = () => {
    let chatHTML = "";
    let keys = [...messages.keys()].sort();
    let last_timestap = 0;
    for (i in keys) {
        let k = keys[i];
        let msg = messages.get(k);
        // msg.provider_name
        // msg.room
        // msg.message
        // msg.username
        // msg.timestamp
        let text = `#${msg.provider_name}::${msg.username}> ${msg.message}\n`;

        chatHTML += text;

    }

    const content = document.getElementById("content");
    content.innerText = chatHTML;

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
    req.open("get", "yarrdb_log.jsonl", true);
    req.send();
};

window.onload = () => {
    loadData();
    loadLog();
};
window.setInterval(loadLog, 500);