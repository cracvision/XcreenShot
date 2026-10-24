// Drives the main window of an isolated dev instance through the WebView2 DevTools
// protocol, without touching the real mouse or keyboard. See README.md for how to
// start that instance. Coordinates are CSS pixels of the page.
//
//   node cdp.mjs eval "<expr>"            evaluate in the page, print JSON
//   node cdp.mjs shot out.png             screenshot of the page
//   node cdp.mjs rect "<css selector>"    position and size of an element
//   node cdp.mjs click x y [ctrl|shift|alt…]
//   node cdp.mjs drag x1 y1 x2 y2 [ctrl|shift|alt…]
//   node cdp.mjs move x y
//   node cdp.mjs key <key> <code> [ctrl|shift|alt…]   e.g. key z KeyZ ctrl
//   node cdp.mjs wheel x y deltaY [ctrl…]
import { writeFileSync } from "node:fs";

const PORT = 9333;
const targets = await (await fetch(`http://127.0.0.1:${PORT}/json`)).json();

function connect(url) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    let id = 0;
    const waiting = new Map();
    ws.onmessage = (m) => {
      const msg = JSON.parse(m.data);
      if (msg.id && waiting.has(msg.id)) {
        waiting.get(msg.id)(msg);
        waiting.delete(msg.id);
      }
    };
    ws.onopen = () =>
      resolve({
        send: (method, params = {}) =>
          new Promise((res) => {
            const i = ++id;
            waiting.set(i, res);
            ws.send(JSON.stringify({ id: i, method, params }));
          }),
        close: () => ws.close(),
      });
    ws.onerror = reject;
  });
}

async function evaluate(c, expression) {
  const r = await c.send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  if (r.result?.exceptionDetails)
    return {
      error: r.result.exceptionDetails.text,
      details: r.result.exceptionDetails.exception?.description,
    };
  return r.result?.result?.value;
}

let main = null;
for (const t of targets.filter((t) => t.type === "page")) {
  const c = await connect(t.webSocketDebuggerUrl);
  const label = await evaluate(
    c,
    "window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label",
  );
  if (label === "main") {
    main = c;
    break;
  }
  c.close();
}
if (!main) {
  console.error("main window not found");
  process.exit(1);
}

const [cmd, ...args] = process.argv.slice(2);
const mods = (list) =>
  list.reduce(
    (m, k) => m | ({ alt: 1, ctrl: 2, meta: 4, shift: 8 }[k] ?? 0),
    0,
  );
const mouse = (type, x, y, modifiers = 0, extra = {}) =>
  main.send("Input.dispatchMouseEvent", {
    type,
    x,
    y,
    modifiers,
    button: "left",
    buttons: type === "mouseReleased" ? 0 : 1,
    clickCount: 1,
    pointerType: "mouse",
    ...extra,
  });

switch (cmd) {
  case "eval":
    console.log(JSON.stringify(await evaluate(main, args[0]), null, 1));
    break;
  case "shot": {
    const r = await main.send("Page.captureScreenshot", { format: "png" });
    writeFileSync(args[0], Buffer.from(r.result.data, "base64"));
    console.log("saved", args[0]);
    break;
  }
  case "rect":
    console.log(
      JSON.stringify(
        await evaluate(
          main,
          `(() => { const r = document.querySelector(${JSON.stringify(args[0])})?.getBoundingClientRect(); return r && {x: r.x, y: r.y, w: r.width, h: r.height}; })()`,
        ),
      ),
    );
    break;
  case "move": {
    const [x, y] = args.map(Number);
    await mouse("mouseMoved", x, y, 0, { buttons: 0, button: "none" });
    break;
  }
  case "click": {
    const [x, y] = args.slice(0, 2).map(Number);
    const m = mods(args.slice(2));
    await mouse("mouseMoved", x, y, m, { buttons: 0, button: "none" });
    await mouse("mousePressed", x, y, m);
    await mouse("mouseReleased", x, y, m);
    break;
  }
  case "drag": {
    const [x1, y1, x2, y2] = args.slice(0, 4).map(Number);
    const m = mods(args.slice(4));
    await mouse("mouseMoved", x1, y1, m, { buttons: 0, button: "none" });
    await mouse("mousePressed", x1, y1, m);
    const steps = 12;
    for (let i = 1; i <= steps; i++) {
      await mouse(
        "mouseMoved",
        x1 + ((x2 - x1) * i) / steps,
        y1 + ((y2 - y1) * i) / steps,
        m,
      );
    }
    await mouse("mouseReleased", x2, y2, m);
    break;
  }
  case "key": {
    const [key, code, ...rest] = args;
    const m = mods(rest);
    const vk =
      { Delete: 46, Escape: 27, Backspace: 8 }[key] ??
      key.toUpperCase().charCodeAt(0);
    await main.send("Input.dispatchKeyEvent", {
      type: "rawKeyDown",
      key,
      code,
      modifiers: m,
      windowsVirtualKeyCode: vk,
    });
    await main.send("Input.dispatchKeyEvent", {
      type: "keyUp",
      key,
      code,
      modifiers: m,
      windowsVirtualKeyCode: vk,
    });
    break;
  }
  case "wheel": {
    const [x, y, dy] = args.slice(0, 3).map(Number);
    const m = mods(args.slice(3));
    await main.send("Input.dispatchMouseEvent", {
      type: "mouseWheel",
      x,
      y,
      deltaX: 0,
      deltaY: dy,
      modifiers: m,
    });
    break;
  }
}
await new Promise((r) => setTimeout(r, 400));
main.close();
process.exit(0);
