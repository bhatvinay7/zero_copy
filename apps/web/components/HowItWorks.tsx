"use client";

import { Upload, Cpu, Share2 } from "lucide-react";

const steps = [
  {
    step: "01",
    icon: <Upload size={28} />,
    color: "#2563eb",
    bg: "linear-gradient(135deg, #2563eb, #3b82f6)",
    title: "Upload via TUS to R2",
    description:
      "Drag-and-drop your video. Uppy breaks it into 5MB chunks and streams them directly to Cloudflare R2 using TUS protocol. Pause mid-upload, lose WiFi — it picks up exactly where it left off.",
    details: ["Chunked upload (5 MB parts)", "Pause & resume anytime", "Auto-retry on network drop", "Uploads land in your R2 bucket"],
  },
  {
    step: "02",
    icon: <Cpu size={28} />,
    color: "#7c3aed",
    bg: "linear-gradient(135deg, #7c3aed, #8b5cf6)",
    title: "High-Speed Parallel Transcoding",
    description:
      "Our dedicated high-speed processing engine automatically scales resources, utilizing all available processor cores in parallel to decode and encode your video into 5 different resolutions simultaneously.",
    details: ["Optimized memory handling", "High-performance processing", "5 parallel output streams", "H.264 & H.265 compatibility"],
  },
  {
    step: "03",
    icon: <Share2 size={28} />,
    color: "#059669",
    bg: "linear-gradient(135deg, #059669, #10b981)",
    title: "CDN-Ready on R2",
    description:
      "All five resolution variants are written back to R2 — 4K, 1080p, 720p, 480p, 360p — alongside an HLS manifest. Cloudflare's global CDN serves them to any device, anywhere, instantly.",
    details: ["4K, 1080p, 720p, 480p, 360p", "HLS manifests auto-generated", "Served via Cloudflare CDN", "Presigned download URLs"],
  },
];

export default function HowItWorks() {
  return (
    <section
      id="how-it-works"
      className="section"
      style={{
        background: "var(--bg-subtle)",
      }}
    >
      <div className="container">
        {/* Header */}
        <div style={{ textAlign: "center", marginBottom: "var(--space-16)" }}>
          <span className="badge badge-violet" style={{ marginBottom: 16 }}>How It Works</span>
          <h2 style={{ fontSize: "clamp(1.9rem, 4vw, 2.8rem)", fontWeight: 800, letterSpacing: "-0.03em", marginBottom: 16 }}>
            From raw upload to{" "}
            <span className="gradient-text">5 resolutions in minutes</span>
          </h2>
          <p style={{ fontSize: "1.05rem", color: "var(--text-secondary)", maxWidth: 540, margin: "0 auto", lineHeight: 1.7 }}>
            Three steps. One pipeline. No configuration needed.
          </p>
        </div>

        {/* Steps */}
        <div style={{ display: "flex", flexDirection: "column", gap: 24, maxWidth: 900, margin: "0 auto" }}>
          {steps.map((s, i) => (
            <div
              key={s.step}
              className="card"
              style={{
                padding: "32px 36px",
                display: "grid",
                gridTemplateColumns: "auto 1fr",
                gap: 32,
                alignItems: "flex-start",
                position: "relative",
                overflow: "hidden",
              }}
            >
              {/* Step number watermark */}
              <div style={{
                position: "absolute", right: 24, top: 12,
                fontSize: "6rem", fontWeight: 900,
                color: `${s.color}08`,
                lineHeight: 1,
                userSelect: "none",
                pointerEvents: "none",
              }}>
                {s.step}
              </div>

              {/* Icon circle */}
              <div style={{
                width: 64, height: 64, borderRadius: "18px",
                background: s.bg,
                color: "#fff",
                display: "flex", alignItems: "center", justifyContent: "center",
                flexShrink: 0,
                boxShadow: `0 8px 24px ${s.color}35`,
              }}>
                {s.icon}
              </div>

              {/* Content */}
              <div>
                <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 10 }}>
                  <span className="badge" style={{ background: `${s.color}15`, color: s.color, fontSize: "0.7rem" }}>
                    Step {s.step}
                  </span>
                </div>
                <h3 style={{ fontSize: "1.2rem", fontWeight: 700, marginBottom: 10 }}>{s.title}</h3>
                <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", lineHeight: 1.7, marginBottom: 16 }}>
                  {s.description}
                </p>

                {/* Detail pills */}
                <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
                  {s.details.map((d) => (
                    <span
                      key={d}
                      style={{
                        fontSize: "0.78rem", fontWeight: 500,
                        padding: "4px 12px",
                        borderRadius: "var(--radius-full)",
                        background: `${s.color}10`,
                        color: s.color,
                        border: `1px solid ${s.color}20`,
                      }}
                    >
                      ✓ {d}
                    </span>
                  ))}
                </div>
              </div>

              {/* Connector arrow */}
              {i < steps.length - 1 && (
                <div style={{
                  position: "absolute",
                  bottom: -20, left: "50%",
                  transform: "translateX(-50%)",
                  zIndex: 10,
                  width: 2, height: 20,
                  background: "linear-gradient(180deg, var(--border-medium), transparent)",
                }} />
              )}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
