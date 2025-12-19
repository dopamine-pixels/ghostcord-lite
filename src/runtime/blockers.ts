const BLOCK_SUBSTRINGS = [
  "sentry.io",
  "analytics",
  "telemetry",
  "datadog",
  "segment",
  "amplitude",
  "/science",              // Discord “science” analytics endpoints often include this
  "bugsnag"
];

// Optional: block large animated media endpoints (tradeoff)
const OPTIONAL_HEAVY = [
  "tenor.com",
  "giphy.com"
];

function shouldBlock(url: string): boolean {
  const u = url.toLowerCase();
  return BLOCK_SUBSTRINGS.some(s => u.includes(s));
}

export function installBlockers() {
  // fetch
  const origFetch = window.fetch.bind(window);
  window.fetch = async (input: any, init?: any) => {
    const url = typeof input === "string" ? input : input?.url;
    if (url && shouldBlock(url)) {
      return new Response(null, { status: 204, statusText: "Blocked by Ghostcord" });
    }
    return origFetch(input, init);
  };

  // XHR
  const OrigXHR = window.XMLHttpRequest;
  class XHR extends OrigXHR {
    open(method: string, url: string | URL, async?: boolean, user?: string | null, password?: string | null) {
      const u = String(url);
      if (shouldBlock(u)) {
        // “fake open” to keep callers from exploding
        (this as any).__ghostcord_blocked__ = true;
        return super.open(method, "about:blank", async ?? true, user ?? undefined, password ?? undefined);
      }
      return super.open(method, u, async ?? true, user ?? undefined, password ?? undefined);
    }
    send(body?: Document | BodyInit | null) {
      if ((this as any).__ghostcord_blocked__) {
        queueMicrotask(() => {
          (this as any).readyState = 4;
          this.dispatchEvent(new Event("readystatechange"));
          this.dispatchEvent(new Event("load"));
          this.dispatchEvent(new Event("loadend"));
        });
        return;
      }
      return super.send(body as any);
    }
  }
  window.XMLHttpRequest = XHR as any;

  // WebSocket (optional)
  const OrigWS = window.WebSocket;
  window.WebSocket = function(url: string | URL, protocols?: string | string[]) {
    const u = String(url);
    if (shouldBlock(u)) {
      throw new Error("WebSocket blocked by Ghostcord");
    }
    // @ts-ignore
    return new OrigWS(url, protocols);
  } as any;

  console.info("[Ghostcord] blockers installed");
}
