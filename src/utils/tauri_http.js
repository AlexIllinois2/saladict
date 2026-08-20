// Compatibility layer exposing the Tauri v1 `@tauri-apps/api/http` surface
// (fetch/Body/ResponseType) on top of the Tauri v2 WHATWG fetch from
// `@tauri-apps/plugin-http`, so the service plugins keep working unchanged.
import { fetch as tauriFetch } from '@tauri-apps/plugin-http';

export const ResponseType = {
    JSON: 1,
    Text: 2,
    Binary: 3,
};

export class Body {
    constructor(type, payload) {
        this.type = type;
        this.payload = payload;
    }

    static form(data) {
        return new Body('Form', data);
    }

    static json(data) {
        return new Body('Json', data);
    }

    static text(value) {
        return new Body('Text', value);
    }

    static bytes(bytes) {
        return new Body('Bytes', bytes);
    }
}

function hasHeader(headers, name) {
    const target = name.toLowerCase();
    return Object.keys(headers).some((key) => key.toLowerCase() === target);
}

function withQuery(url, query) {
    if (!query) {
        return url;
    }
    const parsed = new URL(url);
    for (const [key, value] of Object.entries(query)) {
        if (value === undefined || value === null) {
            continue;
        }
        parsed.searchParams.append(key, String(value));
    }
    return parsed.toString();
}

function buildBody(body, headers) {
    if (!(body instanceof Body)) {
        return body;
    }
    switch (body.type) {
        case 'Json':
            if (!hasHeader(headers, 'content-type')) {
                headers['Content-Type'] = 'application/json';
            }
            return JSON.stringify(body.payload);
        case 'Text':
            if (!hasHeader(headers, 'content-type')) {
                headers['Content-Type'] = 'text/plain';
            }
            return body.payload;
        case 'Form': {
            if (typeof FormData !== 'undefined' && body.payload instanceof FormData) {
                // Let the runtime set the multipart boundary.
                for (const key of Object.keys(headers)) {
                    if (key.toLowerCase() === 'content-type') {
                        delete headers[key];
                    }
                }
                return body.payload;
            }
            const params = new URLSearchParams();
            for (const [key, value] of Object.entries(body.payload ?? {})) {
                if (value === undefined || value === null) {
                    continue;
                }
                params.append(key, String(value));
            }
            if (!hasHeader(headers, 'content-type')) {
                headers['Content-Type'] = 'application/x-www-form-urlencoded';
            }
            return params.toString();
        }
        case 'Bytes':
            return body.payload instanceof Uint8Array ? body.payload : new Uint8Array(body.payload);
        default:
            return body.payload;
    }
}

export async function fetch(url, options = {}) {
    const { method = 'GET', headers = {}, query, body, responseType, timeout, ...rest } = options;

    const requestHeaders = { ...headers };
    const requestBody = buildBody(body, requestHeaders);

    const init = { ...rest, method, headers: requestHeaders };
    if (requestBody !== undefined) {
        init.body = requestBody;
    }
    if (timeout) {
        init.connectTimeout = timeout;
    }

    const response = await tauriFetch(withQuery(url, query), init);

    const headersObject = {};
    const rawHeaders = {};
    response.headers.forEach((value, key) => {
        headersObject[key] = value;
        rawHeaders[key] = key.toLowerCase() === 'set-cookie' ? value.split(/,\s*(?=[^;=]+=)/) : [value];
    });

    let data;
    switch (responseType) {
        case ResponseType.Binary:
            data = new Uint8Array(await response.arrayBuffer());
            break;
        case ResponseType.Text:
            data = await response.text();
            break;
        default: {
            const text = await response.text();
            try {
                data = JSON.parse(text);
            } catch {
                data = text;
            }
        }
    }

    return {
        url: response.url,
        status: response.status,
        ok: response.ok,
        headers: headersObject,
        rawHeaders,
        data,
    };
}
