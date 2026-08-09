import {
  ArrowDownload20Regular,
  ArrowRight20Regular,
  Checkmark20Regular,
  Code20Regular,
  FolderOpen20Regular,
  ImageMultiple20Regular,
  LockClosed20Regular,
  ShieldCheckmark20Regular,
  Sparkle20Regular,
} from "@fluentui/react-icons";
import "./website.css";

const DOWNLOAD_URL = "https://github.com/szb-66/PicoPic/releases/latest";
const REPOSITORY_URL = "https://github.com/szb-66/PicoPic";

const features = [
  {
    icon: LockClosed20Regular,
    title: "图片不离开电脑",
    copy: "扫描、编码和验证全部在本机完成。无需上传，也不依赖云端服务。",
  },
  {
    icon: ImageMultiple20Regular,
    title: "整批拖入，自动处理",
    copy: "支持图片和文件夹，递归扫描子目录，自动去重并展示每项结果。",
  },
  {
    icon: ShieldCheckmark20Regular,
    title: "只在确实更小时替换",
    copy: "先写临时文件，再验证格式与尺寸；结果无效或没有变小，原图保持不动。",
  },
  {
    icon: Sparkle20Regular,
    title: "按格式精细控制",
    copy: "分别设置 JPEG、PNG、WebP 的有损模式、质量与优化强度。",
  },
  {
    icon: FolderOpen20Regular,
    title: "隐私元数据清理",
    copy: "可移除 EXIF、GPS、XMP、ICC 与非必要文本信息，并规范化图片方向。",
  },
  {
    icon: Code20Regular,
    title: "结果透明可追踪",
    copy: "清楚显示原始大小、压缩后大小、节省比例，以及跳过或失败的原因。",
  },
];

const formats = [
  { name: "JPEG", tone: "amber", mode: "质量重编码 / 无损容器优化", note: "支持 EXIF 方向规范化" },
  { name: "PNG", tone: "green", mode: "调色板量化 / 无损优化", note: "保留 Alpha 透明度" },
  { name: "WebP", tone: "violet", mode: "质量重编码 / 安全容器优化", note: "支持静态 WebP" },
];

export default function Website() {
  return (
    <div className="website">
      <a className="skip-link" href="#main-content">跳到主要内容</a>
      <header className="site-header">
        <a className="site-brand" href="#top" aria-label="PicoPic 首页">
          <img src="./picopic-icon.png" alt="" width="42" height="42" />
          <span><strong>PicoPic</strong><small>本地图片压缩</small></span>
        </a>
        <nav aria-label="主导航">
          <a href="#features">功能</a>
          <a href="#formats">格式</a>
          <a href="#faq">常见问题</a>
          <a href={REPOSITORY_URL} target="_blank" rel="noreferrer">GitHub</a>
        </nav>
        <a className="nav-download" href={DOWNLOAD_URL} target="_blank" rel="noreferrer">
          <ArrowDownload20Regular />前往下载
        </a>
      </header>

      <main id="main-content">
        <section className="hero" id="top">
          <div className="hero-glow hero-glow-one" aria-hidden="true" />
          <div className="hero-glow hero-glow-two" aria-hidden="true" />
          <div className="hero-copy">
            <div className="hero-kicker"><span /> Windows 图片压缩工具 · 完全离线</div>
            <h1>图片变轻，<br /><em>隐私不动。</em></h1>
            <p className="hero-lead">PicoPic 在你的电脑上批量压缩 JPEG、PNG 和 WebP。不上云，不收集，只在结果更小时安全替换原图。</p>
            <div className="hero-actions">
              <a className="button button-primary" href={DOWNLOAD_URL} target="_blank" rel="noreferrer">
                <ArrowDownload20Regular /> 下载 Windows 版
              </a>
              <a className="button button-secondary" href="#how-it-works">
                看看如何工作 <ArrowRight20Regular />
              </a>
            </div>
            <ul className="hero-proof" aria-label="产品特点">
              <li><Checkmark20Regular /> 无需订阅</li>
              <li><Checkmark20Regular /> 无需登录</li>
              <li><Checkmark20Regular /> 64 位 Windows</li>
            </ul>
            <p className="download-note">0.x 安装包暂未代码签名，请仅从官方 Releases 下载并核对 SHA-256。</p>
          </div>

          <div className="product-stage" aria-label="PicoPic 软件界面预览">
            <div className="stage-orbit orbit-one" aria-hidden="true" />
            <div className="stage-orbit orbit-two" aria-hidden="true" />
            <div className="app-preview">
              <div className="preview-titlebar">
                <div className="preview-brand"><img src="./picopic-icon.png" alt="" /><span><strong>PicoPic</strong><small>本地图片压缩</small></span></div>
                <div className="preview-controls" aria-hidden="true"><i /><i /><i /></div>
              </div>
              <div className="preview-body">
                <div className="preview-heading"><span>图片</span><span>原始大小</span><span>结果</span></div>
                <div className="preview-row">
                  <span className="file"><b className="file-type jpeg">JPG</b><i><strong>camera-roll.jpg</strong><small>照片 · 已移除元数据</small></i></span>
                  <span>4.82 MB</span><span className="saved">− 71%</span>
                </div>
                <div className="preview-row">
                  <span className="file"><b className="file-type png">PNG</b><i><strong>landing-hero.png</strong><small>界面素材 · 保留透明度</small></i></span>
                  <span>2.16 MB</span><span className="saved">− 43%</span>
                </div>
                <div className="preview-drop"><ImageMultiple20Regular /><strong>拖入图片或文件夹</strong><span>JPEG · PNG · WebP</span></div>
              </div>
              <div className="preview-footer"><span>＋ 添加图片　▣ 文件夹</span><strong>完全离线处理</strong><span>设置</span></div>
            </div>
            <div className="privacy-chip"><LockClosed20Regular /><span><strong>0 次上传</strong><small>文件只在本机处理</small></span></div>
          </div>
        </section>

        <section className="trust-strip" aria-label="产品承诺">
          <p>你的图片，始终是你的。</p>
          <div><span>无云端</span><i /><span>无账户</span><i /><span>无追踪</span><i /><span>无订阅</span></div>
        </section>

        <section className="section features-section" id="features">
          <div className="section-intro">
            <span className="section-label">为日常图片而做</span>
            <h2>小工具，也把每一步<br />做得足够认真。</h2>
            <p>不是简单地“另存为”。PicoPic 在压缩之前识别格式，在替换之前验证结果，在结束之后告诉你发生了什么。</p>
          </div>
          <div className="feature-grid">
            {features.map(({ icon: Icon, title, copy }, index) => (
              <article className="feature-card" key={title}>
                <span className="feature-number">0{index + 1}</span>
                <div className="feature-icon"><Icon /></div>
                <h3>{title}</h3>
                <p>{copy}</p>
              </article>
            ))}
          </div>
        </section>

        <section className="section how-section" id="how-it-works">
          <div className="how-panel">
            <div className="how-copy">
              <span className="section-label light">安全不是一句口号</span>
              <h2>原图只在结果<br />通过检查后才替换。</h2>
              <p>PicoPic 会先在同一目录生成临时结果，验证文件格式与图片尺寸，再比较体积。任何一步不符合预期，都会保留原图。</p>
              <a href={REPOSITORY_URL} target="_blank" rel="noreferrer">在 GitHub 查看项目 <ArrowRight20Regular /></a>
            </div>
            <ol className="process-list">
              <li><span>01</span><div><strong>读取与识别</strong><p>检查真实文件格式、静态图片状态与当前设置。</p></div></li>
              <li><span>02</span><div><strong>本地编码</strong><p>按格式和质量生成同目录临时文件，不上传。</p></div></li>
              <li><span>03</span><div><strong>验证与比较</strong><p>确认格式、尺寸有效，并且结果严格小于原图。</p></div></li>
              <li><span>04</span><div><strong>安全替换</strong><p>仅在所有检查通过时替换，否则保留原文件。</p></div></li>
            </ol>
          </div>
        </section>

        <section className="section formats-section" id="formats">
          <div className="section-intro centered">
            <span className="section-label">三种常用格式</span>
            <h2>各用各的最佳方式。</h2>
            <p>每种图片格式都有独立设置。优化强度只影响搜索时间，不会暗中改变你选择的质量。</p>
          </div>
          <div className="format-grid">
            {formats.map((format) => (
              <article className="format-card" key={format.name}>
                <span className={`format-orb ${format.tone}`}>{format.name}</span>
                <h3>{format.name}</h3>
                <p>{format.mode}</p>
                <small><Checkmark20Regular /> {format.note}</small>
              </article>
            ))}
          </div>
          <p className="format-limit">为安全起见，APNG、动画 WebP 以及扩展名与内容不匹配的图片会被明确跳过。</p>
        </section>

        <section className="section faq-section" id="faq">
          <div className="faq-heading"><span className="section-label">常见问题</span><h2>下载前，<br />你可能想知道。</h2></div>
          <div className="faq-list">
            <details><summary>图片会上传到服务器吗？<span>＋</span></summary><p>不会。PicoPic 的扫描、编码与结果验证都在你的 Windows 电脑本地完成，不需要账户或云端服务。</p></details>
            <details><summary>压缩会破坏我的原图吗？<span>＋</span></summary><p>PicoPic 会原位替换压缩成功的图片，但只会在临时结果通过格式、尺寸与体积检查后执行。重要图片首次处理前仍建议保留备份。</p></details>
            <details><summary>支持哪些图片格式？<span>＋</span></summary><p>当前支持静态 JPEG、PNG 和 WebP。动画图片、内容与扩展名不匹配的文件会被跳过。</p></details>
            <details><summary>为什么 Windows 显示“未知发布者”？<span>＋</span></summary><p>0.x 阶段安装包尚未进行代码签名。请只从本项目的 GitHub Releases 页面下载，并使用同一 Release 中的 .sha256 文件核验。</p></details>
          </div>
        </section>

        <section className="download-banner">
          <div className="banner-icon"><img src="./picopic-icon.png" alt="PicoPic 图标" /></div>
          <div><span>准备好让图片轻一点了吗？</span><h2>下载 PicoPic，<br />从下一张图片开始。</h2></div>
          <a className="button button-light" href={DOWNLOAD_URL} target="_blank" rel="noreferrer"><ArrowDownload20Regular /> 下载 Windows 版</a>
        </section>
      </main>

      <footer className="site-footer">
        <div className="footer-brand"><img src="./picopic-icon.png" alt="" /><span><strong>PicoPic</strong><small>本地、轻巧、安全的图片压缩工具</small></span></div>
        <div className="footer-links"><a href={DOWNLOAD_URL}>下载</a><a href={REPOSITORY_URL}>GitHub</a><a href={`${REPOSITORY_URL}/issues`}>反馈问题</a></div>
        <p>© 2026 PicoPic · 文件只在你的电脑上处理</p>
      </footer>
    </div>
  );
}
