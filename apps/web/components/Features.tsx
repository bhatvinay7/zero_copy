"use client";

import { Zap, Shield, Cpu, Globe, Clock, BarChart3 } from "lucide-react";

const features = [
  {
    icon: <Cpu size={24} />,
    color: "#2563eb",
    bg: "#eff6ff",
    title: "Ultra-Fast Processing Engine",
    description:
      "Optimized data pipelines send raw video bytes directly to the transcoder without unnecessary memory copies, ensuring maximum efficiency and zero system overhead.",
    tags: ["direct-memory", "high-efficiency", "zero-overhead"],
  },
  {
    icon: <Globe size={24} />,
    color: "#7c3aed",
    bg: "#f5f3ff",
    title: "Secure Cloud Storage",
    description:
      "Uploads are sent directly from the browser to secure global storage, bypassing intermediate servers. Includes secure temporary access links and instant CDN distribution.",
    tags: ["cloud-storage", "secure", "CDN"],
  },
  {
    icon: <BarChart3 size={24} />,
    color: "#059669",
    bg: "#ecfdf5",
    title: "5 Resolutions — Automatically",
    description:
      "Upload once. Get 4K UHD, 1080p FHD, 720p HD, 480p SD, and 360p in parallel transcoding streams. HLS manifests generated automatically for adaptive bitrate playback.",
    tags: ["4K", "1080p", "HLS", "ABR"],
  },
  {
    icon: <Shield size={24} />,
    color: "#dc2626",
    bg: "var(--bg-error)",
    title: "Resilient Uploads",
    description:
      "Pause any upload mid-stream and resume later — even days later. Multipart state is preserved in R2. Auto-retry on network blips with exponential backoff.",
    tags: ["pause/resume", "retry", "offline-safe"],
  },
  {
    icon: <Zap size={24} />,
    color: "#d97706",
    bg: "var(--bg-warning)",
    title: "10× Faster Throughput",
    description:
      "Our high-speed parallel scheduling engine saturates all processor cores dynamically. Transcoding latency is measured in seconds, not minutes — even for large 4K files.",
    tags: ["parallel", "high-performance", "multi-core"],
  },
  {
    icon: <Clock size={24} />,
    color: "#0891b2",
    bg: "#ecfeff",
    title: "Real-Time Job Tracking",
    description:
      "WebSocket-powered job status — watch chunks upload, frames decode, and resolution streams finish in real time. Per-chunk progress bars for complete visibility.",
    tags: ["WebSocket", "live", "chunks"],
  },
];

export default function Features() {
  return (
    <section id="features" className="section" style={{ background: "var(--bg-surface)" }}>
      <div className="container">
        {/* Header */}
        <div style={{ textAlign: "center", marginBottom: "var(--space-16)" }}>
          <span className="badge badge-blue" style={{ marginBottom: "var(--space-4)" }}>
            Why PixelPipe
          </span>
          <h2 style={{ fontSize: "var(--text-4xl)", fontWeight: 800, letterSpacing: "-0.03em", marginBottom: "var(--space-4)" }}>
            Built for speed.{" "}
            <span className="gradient-text">Engineered for scale.</span>
          </h2>
          <p style={{ fontSize: "var(--text-lg)", color: "var(--text-secondary)", maxWidth: 560, margin: "0 auto", lineHeight: 1.7 }}>
            Every component — from storage to codec — is chosen to eliminate bottlenecks and maximize throughput.
          </p>
        </div>

        {/* Grid */}
        <div style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 320px), 1fr))",
          gap: "var(--space-6)",
        }}>
          {features.map((f) => (
            <div
              key={f.title}
              className="card"
              style={{ padding: "var(--space-8)", cursor: "default" }}
            >
              {/* Icon */}
              <div style={{
                width: 52, height: 52,
                borderRadius: 14,
                background: f.bg,
                color: f.color,
                display: "flex", alignItems: "center", justifyContent: "center",
                marginBottom: 18,
              }}>
                {f.icon}
              </div>

              <h3 style={{ fontSize: "var(--text-base)", fontWeight: 700, marginBottom: "var(--space-2)", color: "var(--text-primary)" }}>
                {f.title}
              </h3>
              <p style={{ fontSize: "var(--text-sm)", color: "var(--text-secondary)", lineHeight: 1.65, marginBottom: "var(--space-4)" }}>
                {f.description}
              </p>

              {/* Tags */}
              <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
                {f.tags.map((tag) => (
                  <span
                    key={tag}
                    className="badge"
                    style={{
                      background: f.bg,
                      color: f.color,
                      fontSize: "var(--text-xs)",
                    }}
                  >
                    {tag}
                  </span>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
