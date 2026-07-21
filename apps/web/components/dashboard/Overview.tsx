"use client";

import {
  HardDrive,
  Video,
  Loader2,
  CheckCircle2,
  TrendingUp,
  Zap,
} from "lucide-react";
import type { Job } from "../../lib/types";

interface Stats {
  totalUploads: number;
  activeJobs: number;
  completedJobs: number;
  totalOutputFiles: number;
}

interface OverviewProps {
  stats: Stats;
  recentJobs: Job[];
  onGoToJobs: () => void;
  onGoToUpload: () => void;
}

function StatCard({
  icon,
  label,
  value,
  sub,
  color,
  bg,
}: {
  icon: React.ReactNode;
  label: string;
  value: string | number;
  sub?: string;
  color: string;
  bg: string;
}) {
  return (
    <div
      className="card"
      style={{ padding: "var(--space-6) var(--space-8)", cursor: "default" }}
    >
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 14 }}>
        <div style={{
          width: 40, height: 40, borderRadius: 10,
          background: bg, color,
          display: "flex", alignItems: "center", justifyContent: "center",
        }}>
          {icon}
        </div>
      </div>
      <div style={{ fontSize: "var(--text-4xl)", fontWeight: 900, letterSpacing: "-0.04em", color: "var(--text-primary)", lineHeight: 1, marginBottom: "var(--space-2)" }}>
        {value}
      </div>
      <div style={{ fontSize: "var(--text-sm)", fontWeight: 600, color: "var(--text-secondary)", marginBottom: 2 }}>{label}</div>
      {sub && <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>{sub}</div>}
    </div>
  );
}

export default function Overview({ stats, recentJobs, onGoToJobs, onGoToUpload }: OverviewProps) {
  const done = recentJobs.filter((j) => j.status === "done").slice(0, 4);

  return (
    <div>
      {/* Welcome banner */}
      <div style={{
        padding: "var(--space-8) var(--space-10)",
        borderRadius: "var(--radius-xl)",
        background: "linear-gradient(135deg, #1e1b4b 0%, #1e3a8a 60%, #0f172a 100%)",
        color: "#fff",
        marginBottom: "var(--space-10)",
        position: "relative",
        overflow: "hidden",
      }}>
        <div style={{
          position: "absolute", top: -40, right: -40,
          width: 200, height: 200, borderRadius: "50%",
          background: "rgba(99,102,241,0.15)",
          pointerEvents: "none",
        }} />
        <div style={{ position: "relative" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)", marginBottom: "var(--space-2)" }}>
            <Zap size={16} fill="#fbbf24" color="#fbbf24" />
            <span style={{ fontSize: "var(--text-xs)", fontWeight: 600, color: "rgba(255,255,255,0.6)" }}>
              Automated Video Pipeline
            </span>
          </div>
          <h1 style={{ fontSize: "var(--text-2xl)", fontWeight: 800, color: "#fff", marginBottom: "var(--space-2)", letterSpacing: "-0.02em" }}>
            Welcome back 👋
          </h1>
          <p style={{ fontSize: "var(--text-sm)", color: "rgba(255,255,255,0.6)", marginBottom: "var(--space-6)" }}>
            You have {stats.activeJobs} active job{stats.activeJobs !== 1 ? "s" : ""} running. {stats.completedJobs} completed.
          </p>
          <div style={{ display: "flex", gap: "var(--space-4)", flexWrap: "wrap" }}>
            <button
              id="overview-upload-btn"
              onClick={onGoToUpload}
              className="btn btn-primary btn-sm"
              style={{ display: "flex", alignItems: "center" }}
            >
              + Upload Video
            </button>
            <button
              onClick={onGoToJobs}
              className="btn btn-sm"
              style={{
                display: "flex", alignItems: "center",
                background: "rgba(255,255,255,0.12)",
                color: "#fff",
                border: "1px solid rgba(255,255,255,0.2)",
              }}
            >
              View All Jobs
            </button>
          </div>
        </div>
      </div>

      {/* Stat cards */}
      <div style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))",
        gap: 16,
        marginBottom: 28,
      }}>
        <StatCard
          icon={<Video size={18} />}
          label="Total Uploads"
          value={stats.totalUploads}
          sub="All time"
          color="#2563eb"
          bg="#eff6ff"
        />
        <StatCard
          icon={<Loader2 size={18} />}
          label="Active Jobs"
          value={stats.activeJobs}
          sub="Uploading or transcoding"
          color="#d97706"
          bg="#fffbeb"
        />
        <StatCard
          icon={<CheckCircle2 size={18} />}
          label="Completed"
          value={stats.completedJobs}
          sub={`${stats.totalOutputFiles} output files`}
          color="#059669"
          bg="#ecfdf5"
        />
      </div>



      {/* Recent completed jobs */}
      {done.length > 0 && (
        <div>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "var(--space-4)" }}>
            <h2 style={{ fontSize: "var(--text-lg)", fontWeight: 700 }}>Recent Completions</h2>
            <button
              onClick={onGoToJobs}
              style={{
                fontSize: "var(--text-xs)", fontWeight: 600,
                color: "var(--brand-600)", background: "none",
                border: "none", cursor: "pointer", display: "flex", alignItems: "center", gap: 4,
              }}
            >
              View all <TrendingUp size={13} />
            </button>
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            {done.map((job) => (
              <div
                key={job.id}
                className="card"
                style={{
                  padding: "12px 16px",
                  display: "flex",
                  alignItems: "center",
                  gap: 12,
                }}
              >
                <div style={{
                  width: 40, height: 28, borderRadius: 6,
                  background: "linear-gradient(135deg, #2563eb20, #7c3aed20)",
                  display: "flex", alignItems: "center", justifyContent: "center",
                  fontSize: "0.9rem",
                }}>
                  🎬
                </div>
                <div style={{ flex: 1, overflow: "hidden" }}>
                  <div style={{
                    fontSize: "var(--text-sm)", fontWeight: 600, color: "var(--text-primary)",
                    whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis",
                  }}>
                    {job.filename}
                  </div>
                  <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
                    {job.resolutions?.length ?? 0} resolutions
                  </div>
                </div>
                <div style={{ display: "flex", gap: 4 }}>
                  {(job.resolutions ?? []).slice(0, 3).map((r) => (
                    <span key={r.tag} className="badge" style={{ background: `rgba(37,99,235,0.15)`, color: "#2563eb", fontSize: "0.65rem" }}>
                      {r.tag}
                    </span>
                  ))}
                </div>
                <CheckCircle2 size={16} color="#059669" />
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
