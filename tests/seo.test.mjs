import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const rootUrl = new URL("../", import.meta.url);

test("homepage exposes canonical, social and structured metadata", async () => {
  const html = await readFile(new URL("index.html", rootUrl), "utf8");
  assert.match(html, /<html lang="zh-CN">/);
  assert.match(html, /rel="canonical" href="https:\/\/szb-66\.github\.io\/PicoPic\/"/);
  assert.match(html, /property="og:image"/);
  assert.match(html, /"@type": "SoftwareApplication"/);
});

test("crawler discovery files point to the production website", async () => {
  const [robots, sitemap] = await Promise.all([
    readFile(new URL("public/robots.txt", rootUrl), "utf8"),
    readFile(new URL("public/sitemap.xml", rootUrl), "utf8"),
  ]);
  assert.match(robots, /Sitemap: https:\/\/szb-66\.github\.io\/PicoPic\/sitemap\.xml/);
  assert.match(sitemap, /<loc>https:\/\/szb-66\.github\.io\/PicoPic\/<\/loc>/);
});
