const worker = new Worker(new URL("./worker.js", import.meta.url), {
  type: "module",
  name: "worker1"
});
worker.postMessage("ok");