"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  CheckCircle2,
  Loader2,
  Clock,
  AlertTriangle,
  Upload,
  ChevronDown,
  ChevronRight,
  Eye,
  Trash2,
  RotateCcw,
  Video,
} from "lucide-react";
import type { Job, JobStatus, Resolution } from "../../lib/types";

// A small component to render the video player
function VideoPlayer({ url }: { url: string }) {
  return (
    <div style={{ marginTop: 12, borderRadius: 8, overflow: "hidden", background: "#000" }}>
      <video
        src={url}
        controls
        style={{ width: "100%", maxHeight: 400, display: "block" }}
      />
    </div>
  );
}

interface JobsListProps {
  jobs: Job[];
  onSelectJob: (job: Job) => void;
  onDeleteJob: (id: string) => void;
  onRetryJob: (id: string) => void;
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleString("en-US", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

const STATUS_META: Record<JobStatus, { label: string; color: string; bg: string; icon: React.ReactNode }> = {
  queued: {
    label: "Queued",
    color: "var(--text-secondary)",
    bg: "var(--bg-subtle)",
    icon: <Clock size={13} />,
  },
  uploading: {
    label: "Uploading",
    color: "var(--brand-600)",
    bg: "var(--brand-50)",
    icon: <Upload size={13} />,
  },
  processing: {
    label: "Processing",
    color: "var(--processing-text)",
    bg: "var(--processing-bg)",
    icon: <Loader2 size={13} className="spin-icon" />,
  },
  done: {
    label: "Done",
    color: "var(--success-text)",
    bg: "var(--success-bg)",
    icon: <CheckCircle2 size={13} />,
  },
  error: {
    label: "Error",
    color: "#dc2626",
    bg: "var(--bg-error)",
    icon: <AlertTriangle size={13} />,
  },
};

// Color hash for file thumbnails
function colorFromName(name: string) {
  let h = 0;
  for (const c of name) h = c.charCodeAt(0) + ((h << 5) - h);
  const hue = Math.abs(h) % 360;
  return `hsl(${hue}, 65%, 55%)`;
}

function JobRow({
  job,
  onSelect,
  onDelete,
  onRetry,
}: {
  job: Job;
  onSelect: () => void;
  onDelete: () => void;
  onRetry: () => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const meta = STATUS_META[job.status];
  const thumbColor = colorFromName(job.filename);

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.95 }}
      transition={{ duration: 0.2 }}
      style={{
        border: "1px solid var(--border-light)",
        borderRadius: "var(--radius-md)",
        overflow: "hidden",
        background: "var(--bg-surface)",
        transition: "box-shadow 0.2s",
      }}
      onMouseEnter={(e) => (e.currentTarget.style.boxShadow = "var(--shadow-sm)")}
      onMouseLeave={(e) => (e.currentTarget.style.boxShadow = "none")}
    >
      {/* Main row */}
      <div className="job-row" style={{ display: "flex", alignItems: "center", padding: "14px 16px", gap: 14 }}>
        {/* Thumbnail placeholder */}
        <div style={{
          width: 48, height: 34, borderRadius: 6, flexShrink: 0,
          background: `linear-gradient(135deg, ${thumbColor}55, ${thumbColor}22)`,
          display: "flex", alignItems: "center", justifyContent: "center",
          fontSize: "var(--text-base)",
          border: `1px solid ${thumbColor}30`,
        }}>
          🎬
        </div>

        {/* File info */}
        <div className="job-row-main" style={{ flex: 1, overflow: "hidden" }}>
          <div style={{
            fontWeight: 600, fontSize: "var(--text-sm)", color: "var(--text-primary)",
            whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis", marginBottom: 2,
          }}>
            {job.filename}
          </div>
          <div style={{ display: "flex", gap: "var(--space-2)", fontSize: "var(--text-xs)", color: "var(--text-muted)", flexWrap: "wrap" }}>
            <span>{formatDate(job.createdAt)}</span>
          </div>
        </div>

        {/* Progress bar for active jobs */}
        {(job.status === "processing" || job.status === "uploading") && (
          <div style={{ width: 120, flexShrink: 0 }}>
            <div style={{ height: 4, borderRadius: 2, background: "var(--border-light)", overflow: "hidden" }}>
              <div style={{
                height: "100%",
                width: `${job.progress}%`,
                borderRadius: 2,
                background: job.status === "uploading"
                  ? "linear-gradient(90deg, #2563eb, #3b82f6)"
                  : "linear-gradient(90deg, #7c3aed, #a78bfa)",
                transition: "width 0.5s ease",
                position: "relative", overflow: "hidden",
              }}>
                <div style={{
                  position: "absolute", inset: 0,
                  background: "linear-gradient(90deg, transparent, rgba(255,255,255,0.3), transparent)",
                  animation: "shimmer 1.5s infinite",
                }} />
              </div>
            </div>
            <div style={{ fontSize: "0.68rem", color: "var(--text-muted)", marginTop: 3, textAlign: "right" }}>
              {job.progress}%
            </div>
          </div>
        )}

        {/* Resolution count */}
        {job.status === "done" && job.resolutions && (
          <div style={{ flexShrink: 0, textAlign: "center" }}>
            <div style={{ fontWeight: 700, fontSize: "var(--text-base)", color: "var(--brand-600)", lineHeight: 1 }}>
              {job.resolutions.length}
            </div>
            <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", fontWeight: 500 }}>
              resolutions
            </div>
          </div>
        )}

        {/* Status badge */}
        <span
          className="badge"
          style={{
            background: meta.bg,
            color: meta.color,
            fontSize: "var(--text-xs)",
            display: "flex",
            alignItems: "center",
            gap: 4,
            flexShrink: 0,
          }}
        >
          {meta.icon}
          {meta.label}
        </span>

        {/* Actions */}
        <div style={{ display: "flex", gap: 4, flexShrink: 0 }}>
          {job.status === "done" && (
            <button
              id={`job-view-${job.id}`}
              onClick={onSelect}
              title="View resolutions"
              style={{
                padding: "var(--space-2) var(--space-2)",
                borderRadius: 8,
                border: "1px solid var(--border-light)",
                background: "var(--bg-surface)",
                color: "var(--brand-600)",
                cursor: "pointer",
                fontSize: "var(--text-xs)",
                fontWeight: 600,
                display: "flex",
                alignItems: "center",
                gap: 4,
                transition: "all 0.15s",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = "var(--brand-50)";
                e.currentTarget.style.borderColor = "var(--brand-300)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = "var(--bg-surface)";
                e.currentTarget.style.borderColor = "var(--border-light)";
              }}
            >
              <Eye size={13} /> View
            </button>
          )}
          {job.status === "error" && (
            <button
              onClick={onRetry}
              title="Retry"
              style={{
                padding: "6px 8px", borderRadius: 8,
                border: "1px solid var(--border-error)", background: "var(--bg-error)",
                color: "#dc2626", cursor: "pointer", display: "flex", alignItems: "center",
                transition: "all 0.15s",
              }}
            >
              <RotateCcw size={13} />
            </button>
          )}
          <button
            onClick={onDelete}
            title="Delete"
            style={{
              padding: "6px 8px", borderRadius: 8,
              border: "1px solid var(--border-light)", background: "var(--bg-surface)",
              color: "var(--text-muted)", cursor: "pointer", display: "flex", alignItems: "center",
              transition: "all 0.15s",
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = "var(--bg-error)";
              e.currentTarget.style.color = "#dc2626";
              e.currentTarget.style.borderColor = "#fca5a5";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = "var(--bg-surface)";
              e.currentTarget.style.color = "var(--text-muted)";
              e.currentTarget.style.borderColor = "var(--border-light)";
            }}
          >
            <Trash2 size={13} />
          </button>
        </div>

        {/* Expand toggle */}
        <button
          onClick={() => setExpanded(!expanded)}
          style={{
            background: "none", border: "none",
            padding: 8, cursor: "pointer",
            color: "var(--text-muted)",
            display: "flex", alignItems: "center", justifyContent: "center",
          }}
        >
          {expanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
        </button>
      </div>

      {/* Expanded Inline Panel via Framer Motion */}
      <AnimatePresence>
        {expanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
            style={{ overflow: "hidden", borderTop: "1px solid var(--border-light)", background: "var(--bg-base)" }}
          >
            <div style={{ padding: "16px" }}>
              {/* If Processing: Show per-resolution progress */}
              {job.status === "processing" && job.progress_details && Object.keys(job.progress_details).length > 0 && (
                <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                  <div style={{ fontSize: "var(--text-xs)", fontWeight: 600, color: "var(--text-secondary)", marginBottom: 4 }}>
                    Detailed Resolution Progress
                  </div>
                  {Object.entries(job.progress_details).map(([resLabel, p]) => (
                    <div key={resLabel} style={{ display: "flex", alignItems: "center", gap: 12 }}>
                      <div style={{ width: 60, fontSize: "var(--text-xs)", color: "var(--text-muted)", fontWeight: 500 }}>
                        {resLabel}
                      </div>
                      <div style={{ flex: 1, height: 6, borderRadius: 3, background: "var(--border-light)", overflow: "hidden" }}>
                        <motion.div
                          initial={{ width: 0 }}
                          animate={{ width: `${p}%` }}
                          transition={{ duration: 0.5, ease: "easeOut" }}
                          style={{
                            height: "100%",
                            background: p >= 100 ? "#059669" : "linear-gradient(90deg, #7c3aed, #a78bfa)",
                          }}
                        />
                      </div>
                      <div style={{ width: 30, fontSize: "var(--text-xs)", color: "var(--text-muted)", textAlign: "right" }}>
                        {p}%
                      </div>
                    </div>
                  ))}
                </div>
              )}

              {/* If Done: Show Video Preview */}
              {job.status === "done" && job.resolutions && job.resolutions.length > 0 && (
                <div>
                  <div style={{ fontSize: "var(--text-xs)", fontWeight: 600, color: "var(--text-secondary)", marginBottom: 8, display: "flex", alignItems: "center", gap: 6 }}>
                    <Video size={14} /> Video Preview
                  </div>
                  {/* Select highest resolution for preview by default (e.g. index 0 assuming it's sorted) */}
                  <VideoPlayer url={job.resolutions[0].url} />
                </div>
              )}

              {/* If no detailed data yet */}
              {((job.status === "processing" && !job.progress_details) || (job.status === "queued")) && (
                <div style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", padding: "12px 0" }}>
                  Waiting for task details...
                </div>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Error expansion */}
      {job.status === "error" && job.error && (
        <div style={{
          padding: "10px 16px",
          background: "var(--bg-error)",
          borderTop: "1px solid #fecaca",
          fontSize: "0.78rem",
          color: "#dc2626",
          display: "flex",
          alignItems: "flex-start",
          gap: 8,
        }}>
          <AlertTriangle size={13} style={{ flexShrink: 0, marginTop: 1 }} />
          {job.error}
        </div>
      )}
    </motion.div>
  );
}

export default function JobsList({ jobs, onSelectJob, onDeleteJob, onRetryJob }: JobsListProps) {
  const [filter, setFilter] = useState<JobStatus | "all">("all");

  const filtered = filter === "all" ? jobs : jobs.filter((j) => j.status === filter);
  const counts = jobs.reduce(
    (acc, j) => { acc[j.status] = (acc[j.status] ?? 0) + 1; return acc; },
    {} as Record<string, number>
  );

  const filters: { label: string; value: JobStatus | "all" }[] = [
    { label: `All (${jobs.length})`, value: "all" },
    { label: `Done (${counts.done ?? 0})`, value: "done" },
    { label: `Processing (${counts.processing ?? 0})`, value: "processing" },
    { label: `Uploading (${counts.uploading ?? 0})`, value: "uploading" },
    { label: `Queued (${counts.queued ?? 0})`, value: "queued" },
    { label: `Error (${counts.error ?? 0})`, value: "error" },
  ];

  return (
    <div>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 20 }}>
        <h2 style={{ fontSize: "1.2rem", fontWeight: 700, color: "var(--text-primary)" }}>
          Transcoding Jobs
        </h2>
        <span style={{ fontSize: "0.8rem", color: "var(--text-muted)" }}>
          Click <strong>View</strong> on any completed job to see its resolution outputs
        </span>
      </div>

      {/* Filter tabs */}
      <div style={{ display: "flex", gap: 6, marginBottom: 16, flexWrap: "wrap" }}>
        {filters.map((f) => (
          <button
            key={f.value}
            id={`jobs-filter-${f.value}`}
            onClick={() => setFilter(f.value)}
            style={{
              padding: "5px 12px",
              borderRadius: "var(--radius-full)",
              border: `1px solid ${filter === f.value ? "var(--brand-300)" : "var(--border-light)"}`,
              background: filter === f.value ? "var(--brand-50)" : "var(--bg-surface)",
              color: filter === f.value ? "var(--brand-600)" : "var(--text-secondary)",
              fontSize: "0.78rem",
              fontWeight: 600,
              cursor: "pointer",
              transition: "all 0.15s",
            }}
          >
            {f.label}
          </button>
        ))}
      </div>

      {/* List */}
      {filtered.length === 0 ? (
        <div style={{
          textAlign: "center", padding: "48px 16px",
          color: "var(--text-muted)", fontSize: "0.875rem",
          border: "1px dashed var(--border-medium)",
          borderRadius: "var(--radius-lg)",
          background: "var(--bg-subtle)",
        }}>
          No {filter !== "all" ? filter : ""} jobs found.
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {filtered.map((job) => (
            <JobRow
              key={job.id}
              job={job}
              onSelect={() => onSelectJob(job)}
              onDelete={() => onDeleteJob(job.id)}
              onRetry={() => onRetryJob(job.id)}
            />
          ))}
        </div>
      )}

      <style>{`
        @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
        .spin-icon { animation: spin 1s linear infinite; }
      `}</style>
    </div>
  );
}
