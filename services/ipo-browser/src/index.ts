import { createApp } from "./server.js";

const port = parseInt(process.env["PORT"] ?? "8081", 10);

createApp().listen(port, "0.0.0.0", () => {
  console.log(`ipo-browser listening on port ${port}`);
});
