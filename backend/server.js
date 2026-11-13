import http from "node:http";

const apiKey = process.env.GEMINI_API_KEY;
const model = process.env.GEMINI_MODEL || "gemini-2.5-flash";
const port = Number(process.env.PORT || 8787);
const buckets = new Map();

function send(res, status, data) {
  res.writeHead(status, {
    "Content-Type": "application/json; charset=utf-8",
    "Cache-Control": "no-store",
    "X-Content-Type-Options": "nosniff"
  });
  res.end(JSON.stringify(data));
}

function allowed(ip) {
  const now = Date.now();
  const entry = buckets.get(ip) || { start: now, count: 0 };
  if (now - entry.start > 60_000) { entry.start = now; entry.count = 0; }
  entry.count += 1;
  buckets.set(ip, entry);
  return entry.count <= 15;
}

const server = http.createServer(async (req, res) => {
  if (req.method === "GET" && req.url === "/health") return send(res, 200, { ok: true });
  if (req.method !== "POST" || req.url !== "/api/chat") return send(res, 404, { error: "Not found" });
  if (!apiKey) return send(res, 503, { error: "Assistant is not configured on the server" });
  const ip = req.socket.remoteAddress || "unknown";
  if (!allowed(ip)) return send(res, 429, { error: "Rate limit reached. Please wait a minute." });

  try {
    let raw = "";
    for await (const chunk of req) {
      raw += chunk;
      if (raw.length > 16_384) return send(res, 413, { error: "Message is too large" });
    }
    const body = JSON.parse(raw || "{}");
    const message = typeof body.message === "string" ? body.message.trim() : "";
    if (!message || message.length > 4_000) return send(res, 400, { error: "Enter a message up to 4000 characters" });

    const upstream = await fetch(
      `https://generativelanguage.googleapis.com/v1beta/models/${encodeURIComponent(model)}:generateContent?key=${encodeURIComponent(apiKey)}`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        signal: AbortSignal.timeout(25_000),
        body: JSON.stringify({
          systemInstruction: { parts: [{ text: "You are Crimson, a helpful Android device and gaming assistant. Give safe, reversible guidance. Never claim to change device settings unless an authorized tool actually did so. Do not recommend disabling thermal protections or blindly removing system apps." }] },
          contents: [{ role: "user", parts: [{ text: message }] }],
          generationConfig: { temperature: 0.5, maxOutputTokens: 700 }
        })
      }
    );
    const data = await upstream.json().catch(() => ({}));
    if (!upstream.ok) {
      const status = upstream.status === 429 ? 429 : 502;
      return send(res, status, { error: upstream.status === 429 ? "Gemini quota or rate limit reached" : "Gemini service request failed" });
    }
    const reply = data?.candidates?.[0]?.content?.parts?.map(p => p.text || "").join("").trim();
    if (!reply) return send(res, 502, { error: "Gemini returned no text" });
    return send(res, 200, { reply });
  } catch {
    return send(res, 400, { error: "Invalid request or temporary network failure" });
  }
});

server.listen(port, "0.0.0.0", () => {
  console.log(`Crimson Gemini backend listening on port ${port}`);
});
