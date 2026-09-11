"use client";

import { useEffect, useRef, useState } from "react";
import { useTheme } from "next-themes";
import Uppy from "@uppy/core";
import Tus from "@uppy/tus";
import Dashboard from "@uppy/dashboard";
import { CloudUpload, Info } from "lucide-react";
import type { UploadFileState } from "../../lib/types";

const CHUNK_SIZE = 5 * 1024 * 1024; // 5 MB

const TUS_ENDPOINT = process.env.NEXT_PUBLIC_TUS_ENDPOINT || "http://localhost:1081/files/";

interface UploadPanelProps {
  onUploadComplete: (fileId: string, filename: string, fileSize: number) => void;
  onFilesChange: (files: UploadFileState[]) => void;
}

export default function UploadPanel({ onUploadComplete, onFilesChange }: UploadPanelProps) {
  const mountRef = useRef<HTMLDivElement>(null);
  const uppyRef = useRef<Uppy | null>(null);
  const [initialized, setInitialized] = useState(false);
  const { theme, systemTheme } = useTheme();
  
  // Resolve current actual theme
  const currentTheme = theme === "system" ? systemTheme : theme;

  useEffect(() => {
    if (!mountRef.current || uppyRef.current) return;

    // ── Inject Uppy CSS from jsDelivr CDN ─────────────────────
    const CSS_HREFS = [
      "https://cdn.jsdelivr.net/npm/@uppy/core@5.2.0/dist/style.min.css",
      "https://cdn.jsdelivr.net/npm/@uppy/dashboard@5.1.1/dist/style.min.css",
      "https://cdn.jsdelivr.net/npm/@uppy/status-bar@5.1.0/dist/style.min.css",
    ];
    const injectedLinks: HTMLLinkElement[] = [];
    CSS_HREFS.forEach((href) => {
      if (!document.querySelector(`link[href="${href}"]`)) {
        const link = document.createElement("link");
        link.rel = "stylesheet";
        link.href = href;
        document.head.appendChild(link);
        injectedLinks.push(link);
      }
    });

    // ── Uppy instance ──────────────────────────────────────────
    const uppy = new Uppy({
      id: "transcoder-uppy-v2", // changed to bypass cached broken uploads
      autoProceed: false,
      allowMultipleUploadBatches: true,
      restrictions: {
        allowedFileTypes: [
          "video/*",
          ".mp4", ".mkv", ".mov", ".avi", ".webm", ".flv", ".wmv", ".m4v",
        ],
        maxFileSize: 5 * 1024 * 1024 * 1024, // 5 GB
      },
    });

    // ── TUS plugin — chunked, resumable, auto-retry ────────────
    uppy.use(Tus, {
      endpoint: TUS_ENDPOINT,
      chunkSize: CHUNK_SIZE,
      retryDelays: [0, 3_000, 5_000, 10_000, 20_000],
      removeFingerprintOnSuccess: false,
    });

    // ── Dashboard UI plugin ────────────────────────────────────
    uppy.use(Dashboard, {
      target: mountRef.current,
      inline: true,
      proudlyDisplayPoweredByUppy: false,
      hideProgressDetails: false,
      height: 550,
      width: "100%",
      theme: (currentTheme === "dark" ? "dark" : "light") as any,
      locale: {
        strings: {
          dropPasteFiles: "Drop video files here or %{browseFiles}",
          browseFiles: "browse",
        },
      },
    });

    // ── Per-file state tracking ────────────────────────────────
    const fileStates = new Map<string, UploadFileState>();
    const lastTs = new Map<string, number>();

    const sync = () => onFilesChange(Array.from(fileStates.values()));

    uppy.on("file-added", (file) => {
      fileStates.set(file.id, {
        id: file.id,
        name: file.name ?? "video",
        size: file.size ?? 0,
        type: file.type ?? "video/mp4",
        progress: 0,
        bytesUploaded: 0,
        bytesTotal: file.size ?? 0,
        status: "added",
        chunksUploaded: 0,
        chunksTotal: Math.ceil((file.size ?? 0) / CHUNK_SIZE),
      });
      sync();
    });

    uppy.on("file-removed", (file) => {
      fileStates.delete(file.id);
      lastTs.delete(file.id);
      sync();
    });

    uppy.on("upload-progress", (file, progress) => {
      if (!file) return;
      const state = fileStates.get(file.id);
      if (!state) return;

      const uploaded = progress.bytesUploaded ?? 0;
      const total = progress.bytesTotal ?? file.size ?? 0;
      const now = Date.now();
      const prev = lastTs.get(file.id) ?? now;
      const elapsed = (now - prev) / 1000;
      const delta = uploaded - state.bytesUploaded;
      const speed = elapsed > 0.5 && delta > 0 ? delta / elapsed : (state.speed ?? 0);
      const eta = speed > 0 ? (total - uploaded) / speed : undefined;
      lastTs.set(file.id, now);

      fileStates.set(file.id, {
        ...state,
        progress: total > 0 ? Math.round((uploaded / total) * 100) : 0,
        bytesUploaded: uploaded,
        bytesTotal: total,
        status: "uploading",
        chunksUploaded: Math.floor(uploaded / CHUNK_SIZE),
        speed,
        eta,
      });
      sync();
    });

    uppy.on("upload-success", (file) => {
      if (!file) return;
      const state = fileStates.get(file.id);
      if (state) {
        fileStates.set(file.id, {
          ...state,
          progress: 100,
          chunksUploaded: state.chunksTotal,
          status: "complete",
          speed: undefined,
          eta: 0,
        });
        sync();
      }
      onUploadComplete(file.id, file.name ?? "video", file.size ?? 0);
    });

    uppy.on("upload-error", (file, error) => {
      if (!file) return;
      const state = fileStates.get(file.id);
      if (state) {
        fileStates.set(file.id, { ...state, status: "error", error: error.message });
        sync();
      }
    });

    uppyRef.current = uppy;
    setInitialized(true);

    return () => {
      uppy.destroy();
      uppyRef.current = null;
      injectedLinks.forEach((l) => l.remove());
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Sync theme dynamically when it changes
  useEffect(() => {
    if (uppyRef.current) {
      const dashboard = uppyRef.current.getPlugin("Dashboard");
      if (dashboard) {
        dashboard.setOptions({ theme: currentTheme === "dark" ? "dark" : "light" });
      }
    }
  }, [currentTheme]);

  return (
    <div>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", flexWrap: "wrap", gap: "var(--space-4)", marginBottom: "var(--space-6)" }}>
        <div>
          <h2 style={{ fontSize: "var(--text-lg)", fontWeight: 700, color: "var(--text-primary)", marginBottom: "var(--space-2)" }}>
            Upload Video
          </h2>
          <p style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)" }}>
            Drag &amp; drop or browse to upload your high-resolution videos.
          </p>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
          <CloudUpload size={16} color="var(--brand-600)" />
          <span className="badge badge-blue" style={{ fontSize: "var(--text-xs)" }}>Resumable Uploads</span>
        </div>
      </div>

      {/* Info banner */}
      <div style={{
        display: "flex", gap: "var(--space-2)", padding: "var(--space-2) var(--space-4)",
        borderRadius: "var(--radius-md)",
        background: "var(--brand-50)", border: "1px solid var(--brand-100)",
        marginBottom: "var(--space-4)", alignItems: "flex-start",
      }}>
      </div>

      {/* Uppy Dashboard mount target */}
      <div
        ref={mountRef}
        id="uppy-dashboard-mount"
        style={{
          borderRadius: "var(--radius-lg)",
          overflow: "hidden",
          border: initialized ? "none" : "2px dashed var(--border-medium)",
          minHeight: 550,
          background: initialized ? "transparent" : "var(--bg-subtle)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          boxShadow: initialized ? "var(--shadow-md)" : "none",
        }}
      >
        {!initialized && (
          <span style={{ color: "var(--text-muted)", fontSize: "var(--text-sm)" }}>
            Initialising uploader…
          </span>
        )}
      </div>
      
      {/* CSS Override to make the inner uppy dashboard stretch fully on large screens */}
      <style dangerouslySetInnerHTML={{ __html: `
        .uppy-Root,
        .uppy-Dashboard,
        .uppy-Dashboard--inline,
        .uppy-Dashboard-inner,
        .uppy-Dashboard-innerWrap,
        .uppy-Dashboard-AddFilesPanel,
        .uppy-Dashboard-AddFiles,
        .uppy-Dashboard-files,
        .uppy-Dashboard-Item,
        .uppy-DashboardItem,
        .uppy-Dashboard-Item-preview {
          width: 100% !important;
          max-width: 100% !important;
        }
        .uppy-Dashboard-inner,
        .uppy-Dashboard-innerWrap,
        .uppy-Dashboard-AddFilesPanel,
        .uppy-Dashboard-AddFiles {
          height: 100% !important;
          min-height: 550px !important;
        }
        /* Make the entire drop area clickable by stretching the browse button's hit area */
        .uppy-Dashboard-AddFiles {
          position: relative !important;
        }
        .uppy-Dashboard-AddFiles-title {
          position: static !important;
        }
        .uppy-Dashboard-browse::after {
          content: "";
          position: absolute;
          top: 0; left: 0; right: 0; bottom: 0;
          cursor: pointer;
          z-index: 10;
        }
      `}} />
    </div>
  );
}
