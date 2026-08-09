import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import Website from "./Website";
import { isTauriRuntime } from "./runtime";

describe("PicoPic website", () => {
  it("presents the product and official download path", () => {
    render(<Website />);
    expect(screen.getByRole("heading", { level: 1, name: /图片变轻/ })).toBeInTheDocument();
    const downloads = screen.getAllByRole("link", { name: /下载/ });
    expect(downloads.some((link) => link.getAttribute("href") === "https://github.com/szb-66/PicoPic/releases/latest")).toBe(true);
    expect(screen.getByText(/图片会上传到服务器吗/)).toBeInTheDocument();
  });

  it("detects the Tauri runtime without relying on the URL", () => {
    expect(isTauriRuntime({} as typeof globalThis)).toBe(false);
    expect(isTauriRuntime({ __TAURI_INTERNALS__: {} } as unknown as typeof globalThis)).toBe(true);
  });
});
