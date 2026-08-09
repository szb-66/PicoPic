import { useEffect, useMemo, useState } from "react";
import {
  Add20Regular,
  ArrowClockwise20Regular,
  CheckmarkCircle20Filled,
  Delete20Regular,
  Dismiss20Regular,
  DismissCircle20Filled,
  DocumentImage20Regular,
  FolderOpen20Regular,
  Settings20Regular,
  SubtractCircle20Regular,
} from "@fluentui/react-icons";
import { backend, type Backend } from "./backend";
import {
  DEFAULT_SETTINGS,
  summarizeJobs,
  type AppSettings,
  type FormatQualitySettings,
  type ImageFormat,
  type JobRecord,
  type JobStatus,
  type OptimizationLevel,
  type QueueSummary,
} from "./types";
import { desktopWindowActions, type WindowActions } from "./windowActions";

const statusCopy: Record<JobStatus, string> = {
  queued: "等待中",
  processing: "压缩中",
  success: "已完成",
  unchanged: "无需优化",
  skipped: "已跳过",
  failed: "失败",
  cancelled: "已取消",
};

function formatBytes(bytes: number | null): string {
  if (bytes == null) return "—";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(bytes < 10240 ? 1 : 0)} KB`;
  return `${(bytes / 1024 ** 2).toFixed(2)} MB`;
}

function StatusIcon({ status }: { status: JobStatus }) {
  if (status === "processing") return <span className="spinner" aria-label="压缩中" />;
  if (status === "success") return <CheckmarkCircle20Filled className="status-icon success" />;
  if (status === "failed") return <DismissCircle20Filled className="status-icon danger" />;
  if (status === "unchanged" || status === "skipped" || status === "cancelled") return <SubtractCircle20Regular className="status-icon muted" />;
  return <span className="queued-dot" aria-label="等待中" />;
}

function DropEmpty({ active }: { active: boolean }) {
  return (
    <section className={`drop-empty ${active ? "active" : ""}`} aria-label="图片拖放区域">
      <div className="drop-emblem"><DocumentImage20Regular /></div>
      <h1>{active ? "松开即可开始压缩" : "将图片或文件夹拖到这里"}</h1>
      <p>支持 JPEG、PNG、WebP · 完全离线处理 · 仅在变小时替换</p>
    </section>
  );
}

function JobTable({ jobs, selected, onToggle }: { jobs: JobRecord[]; selected: Set<string>; onToggle: (id: string) => void }) {
  return (
    <div className="table-wrap">
      <table>
        <thead><tr><th className="select-col" /><th>文件</th><th>原始大小</th><th>压缩后</th><th>已节省</th></tr></thead>
        <tbody>
          {jobs.map((job) => (
            <tr key={job.id} className={selected.has(job.id) ? "selected" : ""} onClick={() => onToggle(job.id)}>
              <td className="select-col"><StatusIcon status={job.status} /></td>
              <td>
                <div className="file-cell"><span className="file-name" title={job.path}>{job.fileName}</span><span className={`format-badge ${job.format ?? "unknown"}`}>{job.format?.toUpperCase() ?? "?"}</span></div>
                <div className="job-detail">{job.message || statusCopy[job.status]}</div>
              </td>
              <td>{formatBytes(job.originalBytes)}</td>
              <td>{formatBytes(job.resultBytes)}</td>
              <td className={job.status === "success" ? "saving" : ""}>{job.status === "success" ? `${job.savedPercent.toFixed(1)}%` : "—"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

type SettingsTab = "general" | "quality" | "speed";

function Toggle({ checked, onChange, label }: { checked: boolean; onChange: (value: boolean) => void; label: string }) {
  return <button type="button" role="switch" aria-checked={checked} aria-label={label} className={`toggle ${checked ? "on" : ""}`} onClick={() => onChange(!checked)}><span /></button>;
}

function QualityRow({ label, value, onChange }: { label: string; value: FormatQualitySettings; onChange: (value: FormatQualitySettings) => void }) {
  return (
    <div className="quality-card">
      <div className="quality-heading"><div><strong>{label}</strong><span>{value.lossy ? "有损压缩" : "无损优化"}</span></div><Toggle checked={value.lossy} onChange={(lossy) => onChange({ ...value, lossy })} label={`${label} 有损压缩`} /></div>
      <label className={!value.lossy ? "disabled" : ""}>
        <span>质量</span><input type="range" min="1" max="100" value={value.quality} disabled={!value.lossy} onChange={(event) => onChange({ ...value, quality: Number(event.target.value) })} /><output>{value.quality}%</output>
      </label>
      <p>数值越高越保真，文件通常也越大。</p>
    </div>
  );
}

function SettingsDialog({ settings, onClose, onSave }: { settings: AppSettings; onClose: () => void; onSave: (settings: AppSettings) => Promise<void> }) {
  const [draft, setDraft] = useState(() => structuredClone(settings));
  const [tab, setTab] = useState<SettingsTab>("general");
  const [saving, setSaving] = useState(false);
  const updateFormat = (format: ImageFormat, value: FormatQualitySettings) => setDraft((current) => ({ ...current, [format]: value }));
  const save = async () => { setSaving(true); try { await onSave(draft); } finally { setSaving(false); } };
  return (
    <div className="modal-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}>
      <section className="settings-dialog" role="dialog" aria-modal="true" aria-labelledby="settings-title">
        <header><div><span className="eyebrow">PICOPIC</span><h2 id="settings-title">偏好设置</h2></div><button className="icon-button close" aria-label="关闭设置" onClick={onClose}>×</button></header>
        <nav className="segmented" aria-label="设置页面">
          {([['general', '常规'], ['quality', '质量'], ['speed', '优化速度']] as const).map(([id, label]) => <button key={id} className={tab === id ? "active" : ""} onClick={() => setTab(id)}>{label}</button>)}
        </nav>
        <div className="settings-content">
          {tab === "general" && <div className="settings-stack">
            <div className="setting-row"><div><strong>递归扫描文件夹</strong><p>自动处理所选文件夹内的子文件夹。</p></div><Toggle checked={draft.recursiveFolders} onChange={(recursiveFolders) => setDraft({ ...draft, recursiveFolders })} label="递归扫描文件夹" /></div>
            <div className="setting-row"><div><strong>移除图片元数据</strong><p>规范化旋转方向后移除 EXIF、GPS、XMP 和色彩配置。</p></div><Toggle checked={draft.stripMetadata} onChange={(stripMetadata) => setDraft({ ...draft, stripMetadata })} label="移除图片元数据" /></div>
            <div className="setting-row"><div><strong>保留修改时间</strong><p>压缩成功后恢复原文件的创建和修改时间。</p></div><Toggle checked={draft.preserveTimestamps} onChange={(preserveTimestamps) => setDraft({ ...draft, preserveTimestamps })} label="保留修改时间" /></div>
            <div className="privacy-note">所有图片都在本机处理，PicoPic 不会上传任何文件。</div>
          </div>}
          {tab === "quality" && <div className="quality-grid">
            <QualityRow label="JPEG" value={draft.jpeg} onChange={(value) => updateFormat("jpeg", value)} />
            <QualityRow label="PNG" value={draft.png} onChange={(value) => updateFormat("png", value)} />
            <QualityRow label="WebP" value={draft.webp} onChange={(value) => updateFormat("webp", value)} />
          </div>}
          {tab === "speed" && <div className="speed-panel">
            <div className="speed-title"><strong>优化级别</strong><span>{({ fast: "快速", normal: "普通", extra: "额外", extreme: "疯狂" } as const)[draft.optimizationLevel]}</span></div>
            <input aria-label="优化级别" type="range" min="0" max="3" step="1" value={(["fast", "normal", "extra", "extreme"] as OptimizationLevel[]).indexOf(draft.optimizationLevel)} onChange={(event) => setDraft({ ...draft, optimizationLevel: (["fast", "normal", "extra", "extreme"] as OptimizationLevel[])[Number(event.target.value)] })} />
            <div className="speed-labels"><span>快速</span><span>普通</span><span>额外</span><span>疯狂</span></div>
            <p>更高的优化级别会尝试更多编码组合，可能获得更小文件，但处理时间更长。质量设置不会改变。</p>
          </div>}
        </div>
        <footer><button onClick={onClose}>取消</button><button className="primary" disabled={saving} onClick={save}>{saving ? "正在保存…" : "保存设置"}</button></footer>
      </section>
    </div>
  );
}

export default function App({ appBackend = backend, windowActions = desktopWindowActions }: { appBackend?: Backend; windowActions?: WindowActions }) {
  const [jobs, setJobs] = useState<JobRecord[]>([]);
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [summary, setSummary] = useState<QueueSummary>(() => summarizeJobs([]));
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [dropActive, setDropActive] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    const unlisten: Array<() => void> = [];
    Promise.all([appBackend.loadSettings(), appBackend.initialJobs()]).then(([loaded, initial]) => { if (alive) { setSettings(loaded); setJobs(initial); setSummary(summarizeJobs(initial)); } }).catch((error) => setNotice(String(error)));
    appBackend.onJobUpdated((updated) => setJobs((current) => { const exists = current.some(({ id }) => id === updated.id); const next = exists ? current.map((job) => job.id === updated.id ? updated : job) : [...current, updated]; setSummary(summarizeJobs(next)); return next; })).then((fn) => unlisten.push(fn));
    appBackend.onSummary(setSummary).then((fn) => unlisten.push(fn));
    appBackend.onNativeDrop((paths) => { setDropActive(false); void addPaths(paths); }).then((fn) => unlisten.push(fn));
    return () => { alive = false; unlisten.forEach((fn) => fn()); };
  }, [appBackend]);

  const selectedIds = useMemo(() => [...selected], [selected]);
  const addPaths = async (paths: string[]) => { if (!paths.length) return; setNotice(null); try { const result = await appBackend.addPaths(paths); if (result.warnings.length) setNotice(`已忽略 ${result.warnings.length} 个项目`); setJobs((current) => { const known = new Set(current.map(({ id }) => id)); return [...current, ...result.jobs.filter(({ id }) => !known.has(id))]; }); } catch (error) { setNotice(String(error)); } };
  const pick = async (kind: "files" | "folder") => addPaths(await (kind === "files" ? appBackend.pickFiles() : appBackend.pickFolder()));
  const toggle = (id: string) => setSelected((current) => { const next = new Set(current); next.has(id) ? next.delete(id) : next.add(id); return next; });
  const saveSettings = async (draft: AppSettings) => { const saved = await appBackend.saveSettings(draft); setSettings(saved); setSettingsOpen(false); };
  const clearCompleted = async () => { const remaining = await appBackend.clearJobs("completed"); setJobs(remaining); setSummary(summarizeJobs(remaining)); setSelected(new Set()); };
  const actionIds = selectedIds.length ? selectedIds : jobs.map(({ id }) => id);

  return (
    <main className="app-shell" onDragEnter={(event) => { event.preventDefault(); setDropActive(true); }} onDragOver={(event) => event.preventDefault()} onDragLeave={(event) => { if (event.currentTarget === event.target) setDropActive(false); }} onDrop={(event) => { event.preventDefault(); setDropActive(false); if (event.dataTransfer.files.length) void addPaths([...event.dataTransfer.files].map((file) => file.name)); }}>
      <header className="app-header">
        <div className="title-drag-region" data-tauri-drag-region>
          <div className="brand" data-tauri-drag-region><span className="brand-mark"><DocumentImage20Regular /></span><div data-tauri-drag-region><strong data-tauri-drag-region>PicoPic</strong><span data-tauri-drag-region>本地图片压缩</span></div></div>
        </div>
        <div className="window-controls" aria-label="窗口控制">
          <button type="button" className="window-control" aria-label="最小化" title="最小化" onClick={() => void windowActions.minimize()}><span className="minimize-glyph" /></button>
          <button type="button" className="window-control" aria-label="最大化或还原" title="最大化或还原" onClick={() => void windowActions.toggleMaximize()}><span className="maximize-glyph" /></button>
          <button type="button" className="window-control close-window" aria-label="关闭" title="关闭" onClick={() => void windowActions.close()}><span className="close-glyph" /></button>
        </div>
      </header>
      <section className="workspace">
        {jobs.length === 0 ? <DropEmpty active={dropActive} /> : <JobTable jobs={jobs} selected={selected} onToggle={toggle} />}
        {notice && <div className="notice" role="status">{notice}</div>}
      </section>
      <footer className="command-bar">
        <div className="command-group"><button className="command" title="添加图片" onClick={() => void pick("files")}><Add20Regular /><span>添加图片</span></button><button className="command" title="添加文件夹" onClick={() => void pick("folder")}><FolderOpen20Regular /><span>文件夹</span></button></div>
        <div className="summary"><strong>{summary.savedBytes > 0 ? `已节省 ${formatBytes(summary.savedBytes)} · ${summary.savedPercent.toFixed(1)}%` : "等待添加图片"}</strong><span>{summary.active > 0 ? `${summary.active} 个任务处理中` : `${summary.total} 个任务 · 完全离线`}</span></div>
        <div className="command-group right"><button className="command" aria-label="取消任务" disabled={!summary.active} onClick={() => void appBackend.cancelJobs(actionIds)}><Dismiss20Regular /><span>取消</span></button><button className="command" disabled={!jobs.length} onClick={() => void appBackend.retryJobs(actionIds)}><ArrowClockwise20Regular /><span>重新优化</span></button><button className="command" disabled={!jobs.length} onClick={() => void clearCompleted()}><Delete20Regular /><span>清理</span></button><button className="command" onClick={() => setSettingsOpen(true)}><Settings20Regular /><span>设置</span></button></div>
      </footer>
      {settingsOpen && <SettingsDialog settings={settings} onClose={() => setSettingsOpen(false)} onSave={saveSettings} />}
    </main>
  );
}
