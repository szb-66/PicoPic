import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "./App";
import { createBrowserBackend } from "./backend";
import { demoJobs } from "./demoData";

describe("PicoPic interface", () => {
  it("shows the offline empty drop state", async () => {
    render(<App />);
    expect(await screen.findByText("将图片或文件夹拖到这里")).toBeInTheDocument();
    expect(screen.getByText(/完全离线处理/)).toBeInTheDocument();
  });

  it("opens all three preference sections and cancels draft changes", async () => {
    render(<App />);
    await screen.findByText("将图片或文件夹拖到这里");
    fireEvent.click(screen.getByRole("button", { name: "设置" }));
    expect(screen.getByRole("dialog", { name: "偏好设置" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("switch", { name: "保留修改时间" }));
    fireEvent.click(screen.getByRole("button", { name: "质量" }));
    expect(screen.getByText("WebP")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "优化速度" }));
    expect(screen.getByRole("slider", { name: "优化级别" })).toHaveValue("1");
    fireEvent.click(screen.getByRole("button", { name: "取消" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    fireEvent.click(screen.getByRole("button", { name: "设置" }));
    expect(screen.getByRole("switch", { name: "保留修改时间" })).toHaveAttribute("aria-checked", "false");
  });

  it("removes terminal rows after clearing completed jobs", async () => {
    const appBackend = createBrowserBackend(demoJobs());
    render(<App appBackend={appBackend} />);
    expect(await screen.findByText("landing-hero.png")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "清理" }));
    await waitFor(() => expect(screen.queryByText("landing-hero.png")).not.toBeInTheDocument());
  });
});
