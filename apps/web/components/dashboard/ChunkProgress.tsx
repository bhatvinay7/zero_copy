"use client";

import { Pause, Play, X, Wifi, CheckCircle, AlertCircle, Clock } from "lucide-react";
import type { UploadFileState } from "../../lib/types";

interface ChunkProgressProps {
  files: UploadFileState[];
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec < 1024 * 1024) return `${(bytesPerSec / 1024).toFixed(0)} KB/s`;
  return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
}

function formatEta(seconds: number): string {
  if (seconds < 60) return `${Math.ceil(seconds)}s`;
  if (seconds < 3600) return `${Math.ceil(seconds / 60)}m`;
  return `${(seconds / 3600).toFixed(1)}h`;
}

const STATUS_CONFIG = {
  added: { label: "Queued", color: "#64748b", bg: "#f1f5f9", icon: <Clock size={12} /> },
  uploading: { label: "Uploading", color: "#2563eb", bg: "#eff6ff", icon: <Wifi size={12} /> },
  paused: { label: "Paused", color: "#d97706", bg: "var(--bg-warning)", icon: <Pause size={12} /> },
  complete: { label: "Complete", color: "#059669", bg: "#ecfdf5", icon: <CheckCircle size={12} /> },
  error: { label: "Error", color: "#dc2626", bg: "var(--bg-error)", icon: <AlertCircle size={12} /> },
};

export default function ChunkProgress({ files }: ChunkProgressProps) {
  if (files.length === 0) {
    return (
      <div style={{
        textAlign: "center",
        padding: "32px 16px",
        color: "var(--text-muted)",
        fontSize: "0.87rem",
        borderRadius: "var(--radius-lg)",
        border: "1px dashed var(--border-medium)",
        background: "var(--bg-subtle)",
      }}>
        No active uploads. Add video files above to begin.
      </div>
    );
  }

  return (
    <div>
      <h3 style={{ fontSize: "var(--text-base)", fontWeight: 700, marginBottom: "var(--space-4)", color: "var(--text-primary)" }}>
        Upload Queue — {files.length} file{files.length !== 1 ? "s" : ""}
      </h3>

      <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}>
        {files.map((file) => {
          const cfg = STATUS_CONFIG[file.status] ?? STATUS_CONFIG.added;
          const chunkLabel = file.chunksTotal > 0
            ? `${file.chunksUploaded} / ${file.chunksTotal} chunks`
            : "Calculating…";

          return (
            <div
              key={file.id}
              className="card"
              style={{ padding: "var(--space-4) var(--space-4)", cursor: "default" }}
            >
              <div style={{ display: "flex", alignItems: "flex-start", flexWrap: "wrap", gap: "var(--space-2)", marginBottom: "var(--space-2)" }}>
                {/* File type icon placeholder */}
                <div style={{
                  width: 36, height: 36, borderRadius: 8, flexShrink: 0,
                  background: "linear-gradient(135deg, #2563eb20, #7c3aed20)",
                  display: "flex", alignItems: "center", justifyContent: "center",
                  fontSize: "1.1rem",
                }}>
                  🎬
                </div>

                {/* Name + meta */}
                <div style={{ flex: 1, overflow: "hidden" }}>
                  <div style={{
                    fontWeight: 600, fontSize: "var(--text-sm)",
                    whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis",
                    color: "var(--text-primary)", marginBottom: 3,
                  }}>
                    {file.name}
                  </div>
                  <div style={{ display: "flex", gap: "var(--space-2)", flexWrap: "wrap", fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
                    <span>{formatBytes(file.size)}</span>
                    <span style={{ color: "var(--border-medium)" }}>·</span>
                    <span>{chunkLabel}</span>
                    {file.status === "uploading" && file.speed && (
                      <>
                        <span style={{ color: "var(--border-medium)" }}>·</span>
                        <span style={{ color: "var(--brand-600)", fontWeight: 600 }}>
                          ↑ {formatSpeed(file.speed)}
                        </span>
                      </>
                    )}
                    {file.status === "uploading" && file.eta != null && file.eta > 0 && (
                      <>
                        <span style={{ color: "var(--border-medium)" }}>·</span>
                        <span>{formatEta(file.eta)} left</span>
                      </>
                    )}
                  </div>
                </div>

                {/* Status badge */}
                <span
                  className="badge"
                  style={{
                    background: cfg.bg,
                    color: cfg.color,
                    fontSize: "var(--text-xs)",
                    flexShrink: 0,
                    gap: 4,
                    display: "flex",
                    alignItems: "center",
                  }}
                >
                  {cfg.icon}
                  {cfg.label}
                </span>
              </div>

              {/* Progress bar */}
              <div style={{ marginBottom: 8 }}>
                <div style={{
                  height: 6,
                  borderRadius: 3,
                  background: "var(--border-light)",
                  overflow: "hidden",
                }}>
                  <div style={{
                    height: "100%",
                    width: `${file.progress}%`,
                    borderRadius: 3,
                    transition: "width 0.4s ease",
                    background: file.status === "error"
                      ? "#ef4444"
                      : file.status === "complete"
                      ? "linear-gradient(90deg, #059669, #34d399)"
                      : file.status === "paused"
                      ? "linear-gradient(90deg, #d97706, #fbbf24)"
                      : "linear-gradient(90deg, #2563eb, #7c3aed)",
                    position: "relative",
                    overflow: "hidden",
                  }}>
                    {/* Shimmer on active uploads */}
                    {file.status === "uploading" && (
                      <div style={{
                        position: "absolute", inset: 0,
                        background: "linear-gradient(90deg, transparent 0%, rgba(255,255,255,0.35) 50%, transparent 100%)",
                        backgroundSize: "200% 100%",
                        animation: "shimmer 1.5s infinite",
                      }} />
                    )}
                  </div>
                </div>
                <div style={{
                  display: "flex",
                  justifyContent: "space-between",
                  marginTop: 4,
                  fontSize: "var(--text-xs)",
                  color: "var(--text-muted)",
                }}>
                  <span>{formatBytes(file.bytesUploaded)} / {formatBytes(file.bytesTotal)}</span>
                  <span style={{ fontWeight: 600 }}>{file.progress}%</span>
                </div>
              </div>

              {/* Chunk dots */}
              {file.chunksTotal > 0 && file.chunksTotal <= 60 && (
                <div style={{ display: "flex", flexWrap: "wrap", gap: 3, marginBottom: 8 }}>
                  {Array.from({ length: file.chunksTotal }, (_, i) => (
                    <div
                      key={i}
                      title={`Chunk ${i + 1}`}
                      style={{
                        width: 8, height: 8,
                        borderRadius: 2,
                        background: i < file.chunksUploaded
                          ? "#2563eb"
                          : i === file.chunksUploaded && file.status === "uploading"
                          ? "#93c5fd"
                          : "var(--border-light)",
                        transition: "background 0.3s",
                      }}
                    />
                  ))}
                </div>
              )}

              {/* Error message */}
              {file.status === "error" && file.error && (
                <div style={{
                  fontSize: "0.75rem",
                  color: "#dc2626",
                  background: "var(--bg-error)",
                  borderRadius: 6,
                  padding: "6px 10px",
                  marginTop: 4,
                }}>
                  ⚠ {file.error}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
