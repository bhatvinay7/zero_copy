"use client";

import { useEffect, useRef } from "react";
import {
  X,
  Download,
  Play,
  Monitor,
  Film,
  Info,
  ExternalLink,
  Clock,
  FileVideo,
} from "lucide-react";
import type { Job, Resolution } from "../../lib/types";

interface ResolutionPanelProps {
  job: Job | null;
  onClose: () => void;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function ResolutionCard({ res, index }: { res: Resolution; index: number }) {
  const isTop = index === 0;

  return (
    <div
      style={{
        border: `1.5px solid ${isTop ? "rgba(124,58,237,0.4)" : "var(--border-light)"}`,
        borderRadius: "var(--radius-md)",
        padding: "16px 18px",
        background: isTop ? "rgba(124,58,237,0.06)" : "var(--bg-surface)",
        position: "relative",
        overflow: "hidden",
        transition: "all 0.2s",
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.boxShadow = `0 4px 20px ${isTop ? "rgba(124,58,237,0.2)" : "rgba(37,99,235,0.2)"}`;
        e.currentTarget.style.borderColor = isTop ? "rgba(124,58,237,0.6)" : "rgba(37,99,235,0.6)";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.boxShadow = "none";
        e.currentTarget.style.borderColor = isTop ? "rgba(124,58,237,0.4)" : "var(--border-light)";
      }}
    >
      {isTop && (
        <div style={{
          position: "absolute", top: 8, right: 8,
          fontSize: "var(--text-xs)", fontWeight: 700, padding: "2px 8px",
          borderRadius: "var(--radius-full)",
          background: isTop ? "#7c3aed" : "#2563eb",
          color: "#fff",
        }}>
          HIGHEST
        </div>
      )}

      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)", marginBottom: "var(--space-2)" }}>
        <div style={{
          width: 38, height: 38, borderRadius: 10,
          background: isTop ? "rgba(124,58,237,0.15)" : "rgba(37,99,235,0.15)",
          display: "flex", alignItems: "center", justifyContent: "center",
        }}>
          <Monitor size={18} color={isTop ? "#7c3aed" : "#2563eb"} />
        </div>
        <div>
          <div style={{ fontWeight: 700, fontSize: "var(--text-sm)", color: "var(--text-primary)" }}>
            {res.label}
          </div>
          <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
            {res.resolution}
          </div>
        </div>
      </div>

      {/* Actions */}
      <div style={{ display: "flex", gap: 6 }}>
        <a
          href={res.url}
          target="_blank"
          rel="noopener noreferrer"
          id={`res-preview-${res.tag}`}
          style={{
            flex: 1,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            gap: 5,
            padding: "var(--space-2) var(--space-2)",
            borderRadius: 8,
            background: `rgba(37,99,235,0.12)`,
            color: "#2563eb",
            border: `1px solid rgba(37,99,235,0.25)`,
            fontSize: "var(--text-xs)",
            fontWeight: 600,
            transition: "all 0.15s",
            textDecoration: "none",
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = `rgba(37,99,235,0.22)`;
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = `rgba(37,99,235,0.12)`;
          }}
        >
          <Play size={12} fill="#2563eb" />
          Preview
        </a>
        <a
          href={res.url}
          download
          id={`res-download-${res.tag}`}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 5,
            padding: "var(--space-2) var(--space-2)",
            borderRadius: 8,
            background: "var(--bg-surface)",
            color: "var(--text-secondary)",
            border: "1px solid var(--border-light)",
            fontSize: "var(--text-xs)",
            fontWeight: 600,
            transition: "all 0.15s",
            textDecoration: "none",
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = "var(--bg-muted)";
            e.currentTarget.style.color = "var(--text-primary)";
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = "var(--bg-surface)";
            e.currentTarget.style.color = "var(--text-secondary)";
          }}
        >
          <Download size={12} />
          DL
        </a>
        <a
          href={res.url}
          target="_blank"
          rel="noopener noreferrer"
          style={{
            display: "flex",
            alignItems: "center",
            padding: "7px 8px",
            borderRadius: 8,
            background: "var(--bg-surface)",
            color: "var(--text-muted)",
            border: "1px solid var(--border-light)",
            transition: "all 0.15s",
            textDecoration: "none",
          }}
        >
          <ExternalLink size={12} />
        </a>
      </div>
    </div>
  );
}

export default function ResolutionPanel({ job, onClose }: ResolutionPanelProps) {
  const panelRef = useRef<HTMLDivElement>(null);

  // Close on Escape key
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [onClose]);

  if (!job) return null;

  return (
    <>
      {/* Backdrop */}
      <div
        onClick={onClose}
        style={{
          position: "fixed", inset: 0,
          background: "rgba(15,23,42,0.45)",
          backdropFilter: "blur(4px)",
          zIndex: 200,
          animation: "fadeIn 0.2s ease",
        }}
      />

      {/* Panel */}
      <div
        ref={panelRef}
        className="polished-scroll"
        style={{
          position: "fixed",
          top: "10vh",
          left: "50%",
          transform: "translateX(-50%)",
          width: "min(1000px, 92vw)",
          maxHeight: "80vh",
          background: "var(--bg-surface)",
          zIndex: 201,
          display: "flex",
          flexDirection: "column",
          boxShadow: "0 20px 40px rgba(15,23,42,0.25)",
          borderRadius: "24px",
          animation: "scaleIn 0.28s var(--ease-spring)",
          overflowY: "auto",
        }}
      >
        <style dangerouslySetInnerHTML={{ __html: `
          @keyframes scaleIn {
            from { opacity: 0; transform: translate(-50%, 10px) scale(0.98); }
            to { opacity: 1; transform: translate(-50%, 0) scale(1); }
          }
          
          /* Polished Scroller */
          .polished-scroll::-webkit-scrollbar {
            width: 8px;
          }
          .polished-scroll::-webkit-scrollbar-track {
            background: transparent;
          }
          .polished-scroll::-webkit-scrollbar-thumb {
            background-color: rgba(156, 163, 175, 0.5);
            border-radius: 20px;
            border: 2px solid transparent;
            background-clip: padding-box;
          }
          .polished-scroll::-webkit-scrollbar-thumb:hover {
            background-color: rgba(156, 163, 175, 0.8);
          }
        `}} />
        {/* Header */}
        <div style={{
          padding: "20px 24px",
          borderBottom: "1px solid var(--border-light)",
          position: "sticky", top: 0,
          background: "var(--bg-surface)", zIndex: 10,
        }}>
          <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 12 }}>
            <div style={{ flex: 1, overflow: "hidden" }}>
              <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 6 }}>
                <span className="badge badge-green" style={{ fontSize: "0.7rem" }}>
                  ✓ Transcoding Complete
                </span>
              </div>
              <h2 style={{
                fontSize: "var(--text-base)", fontWeight: 700, color: "var(--text-primary)",
                whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis",
                marginBottom: 4,
              }}>
                {job.filename}
              </h2>
              <div style={{ display: "flex", gap: "var(--space-2)", fontSize: "var(--text-xs)", color: "var(--text-muted)", flexWrap: "wrap" }}>
                <span style={{ display: "flex", alignItems: "center", gap: 4 }}>
                  <FileVideo size={12} /> {job.original_url ? "Original file uploaded" : "File uploaded"}
                </span>
                <span style={{ display: "flex", alignItems: "center", gap: 4 }}>
                  <Info size={12} /> Done {formatDate(job.createdAt)}
                </span>
              </div>
            </div>
            <button
              id="resolution-panel-close"
              onClick={onClose}
              style={{
                padding: 8, borderRadius: 8, border: "1px solid var(--border-light)",
                background: "var(--bg-surface)", color: "var(--text-muted)", cursor: "pointer",
                flexShrink: 0, display: "flex", alignItems: "center",
                transition: "all 0.15s",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = "var(--bg-error)";
                e.currentTarget.style.color = "#dc2626";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = "var(--bg-surface)";
                e.currentTarget.style.color = "var(--text-muted)";
              }}
            >
              <X size={16} />
            </button>
          </div>
        </div>

        {/* Video preview (mini player placeholder) */}
        <div style={{
          margin: "20px 24px 0",
          borderRadius: "var(--radius-md)",
          background: "linear-gradient(135deg, #0f172a, #1e1b4b)",
          height: 160,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          gap: 10,
          position: "relative",
          overflow: "hidden",
        }}>
          <div style={{ fontSize: "2.5rem" }}>🎬</div>
          <div style={{ fontSize: "0.8rem", color: "rgba(255,255,255,0.6)" }}>
            Click Preview on a resolution below to play
          </div>
          {/* Decorative scanlines */}
          <div style={{
            position: "absolute", inset: 0,
            backgroundImage: "repeating-linear-gradient(0deg, transparent, transparent 3px, rgba(255,255,255,0.015) 4px)",
            pointerEvents: "none",
          }} />
        </div>

        {/* Summary bar */}
        <div style={{
          margin: "16px 24px",
          padding: "12px 16px",
          borderRadius: "var(--radius-md)",
          background: "var(--bg-subtle)",
          display: "flex",
          gap: 24,
        }}>
          {[
            { label: "Resolutions", value: `${job.resolutions?.length ?? 0}` },
          ].map((item) => (
            <div key={item.label}>
              <div style={{ fontSize: "0.65rem", fontWeight: 600, color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: "0.07em", marginBottom: 2 }}>
                {item.label}
              </div>
              <div style={{ fontSize: "0.85rem", fontWeight: 700, color: "var(--text-primary)" }}>
                {item.value}
              </div>
            </div>
          ))}
        </div>

        {/* Resolution cards */}
        <div style={{ padding: "0 24px 24px", display: "flex", flexDirection: "column", gap: 10 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 4 }}>
            <Film size={15} color="var(--brand-600)" />
            <span style={{ fontSize: "0.85rem", fontWeight: 700, color: "var(--text-primary)" }}>
              Output Resolutions
            </span>
            <span className="badge badge-blue" style={{ fontSize: "0.65rem" }}>
              Served via Cloudflare CDN
            </span>
          </div>

          {(job.resolutions ?? []).map((res, i) => (
            <ResolutionCard key={res.tag} res={res} index={i} />
          ))}

          {(!job.resolutions || job.resolutions.length === 0) && (
            <div style={{
              padding: "32px 16px", textAlign: "center",
              color: "var(--text-muted)", fontSize: "0.875rem",
              border: "1px dashed var(--border-medium)",
              borderRadius: "var(--radius-md)",
            }}>
              No output resolutions available yet.
            </div>
          )}
        </div>
      </div>
    </>
  );
}
