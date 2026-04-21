import { readAppConfig } from "./infrastructure/config/app-config.js";
import { createApp } from "./server.js";

const config = readAppConfig();
const app = createApp();

app.listen(config.port, "0.0.0.0", () => {
  console.log(`ipo-browser listening on port ${config.port}`);
});
