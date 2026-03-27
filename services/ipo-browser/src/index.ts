import express from "express";

const app = express();
const port = parseInt(process.env["PORT"] ?? "8081", 10);

app.use(express.json());

app.get("/health", (_request, response) => {
  response.json({ status: "ok" });
});

app.listen(port, "0.0.0.0", () => {
  console.log(`ipo-browser listening on port ${port}`);
});
