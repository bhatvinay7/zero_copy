"use client";

import { useState, useEffect, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useVideos, useCancelVideo, useDeleteVideo, useLogin, useSignup } from "../../lib/api";
import { Bell, Search, Menu, LogOut, Lock, User, FileVideo, RefreshCw, XCircle } from "lucide-react";
import Sidebar from "../../components/dashboard/Sidebar";
import Overview from "../../components/dashboard/Overview";
import UploadPanel from "../../components/dashboard/UploadPanel";
import ChunkProgress from "../../components/dashboard/ChunkProgress";
import JobsList from "../../components/dashboard/JobsList";
import ResolutionPanel from "../../components/dashboard/ResolutionPanel";
import type { Job, UploadFileState } from "../../lib/types";

type Tab = "overview" | "upload" | "jobs" | "settings";

export default function DashboardPage() {
  const [token, setToken] = useState<string | null>(null);
  const [userEmail, setUserEmail] = useState<string>("");
  const [authEmail, setAuthEmail] = useState("");
  const [authPassword, setAuthPassword] = useState("");
  const [isSignUp, setIsSignUp] = useState(false);
  const [authError, setAuthError] = useState("");
  const [authLoading, setAuthLoading] = useState(false);
  const [mounted, setMounted] = useState(false);

  const [activeTab, setActiveTab] = useState<Tab>("overview");
  const queryClient = useQueryClient();
  const { data: fetchedJobs, refetch } = useVideos(token);
  const { mutateAsync: cancelVideo } = useCancelVideo();
  const { mutateAsync: deleteVideo } = useDeleteVideo();
  const { mutateAsync: loginApi } = useLogin();
  const { mutateAsync: signupApi } = useSignup();

  const jobs = fetchedJobs || [];
  const [selectedJob, setSelectedJob] = useState<Job | null>(null);
  const [uploadFiles, setUploadFiles] = useState<UploadFileState[]>([]);
  const [sidebarOpen, setSidebarOpen] = useState(false);

  // ── 1. Check for token in localStorage on mount ────────────────────────
  useEffect(() => {
    setMounted(true);
    const savedToken = localStorage.getItem("jwt_token");
    const savedEmail = localStorage.getItem("jwt_email");
    if (savedToken) {
      setToken(savedToken);
      if (savedEmail) setUserEmail(savedEmail);
    }
  }, []);

  // ── 4. Cancel Video Transcode API Call ────────────────────────────────
  const handleDeleteJob = useCallback(
    async (id: string) => {
      if (!token) return;
      try {
        await deleteVideo(id);
      } catch (err) {
        console.error("Failed to delete transcode job:", err);
      }
    },
    [token, deleteVideo]
  );

  // ── 5. Auth Handlers ──────────────────────────────────────────────────
  const handleAuthSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setAuthError("");
    setAuthLoading(true);

    try {
      const data = isSignUp 
        ? await signupApi({ email: authEmail, password: authPassword })
        : await loginApi({ email: authEmail, password: authPassword });

      if (!data.success) {
        setAuthError(data.error || "Authentication failed. Please check credentials.");
        setAuthLoading(false);
        return;
      }

      if (isSignUp) {
        setIsSignUp(false);
        setAuthError("Registration successful! Please sign in using your credentials.");
      } else if (data.token) {
        localStorage.setItem("jwt_token", data.token);
        localStorage.setItem("jwt_email", data.user?.email || authEmail);
        setToken(data.token);
        setUserEmail(data.user?.email || authEmail);
      }
    } catch (err: any) {
      setAuthError(err?.message || "Server unavailable. Please verify http-server is running.");
    } finally {
      setAuthLoading(false);
    }
  };

  const handleSignOut = () => {
    localStorage.removeItem("jwt_token");
    localStorage.removeItem("jwt_email");
    setToken(null);
    setUserEmail("");
    queryClient.clear();
    setActiveTab("overview");
  };

  const handleUploadComplete = useCallback(
    (_fileId: string, _filename: string, _fileSizeBytes: number) => {
      // Reload video list from database on upload finish to get new video details
      if (token) refetch();
    },
    [token, refetch]
  );

  const handleFilesChange = useCallback((files: UploadFileState[]) => {
    setUploadFiles(files);
  }, []);

  const handleRetryJob = useCallback((id: string) => {
    queryClient.setQueryData<Job[]>(["videos"], (old) => {
      if (!old) return old;
      return old.map(job => {
        if (job.id === id) {
          return { ...job, status: "queued", progress: 0, error: undefined };
        }
        return job;
      });
    });
  }, [queryClient]);

  // ── 6. Live Metrics Calculation (No Mock Stats) ──────────────────────────
  const liveStats = {
    totalUploads: jobs.length,
    activeJobs: jobs.filter((j: Job) => j.status === "processing" || j.status === "queued").length,
    completedJobs: jobs.filter((j: Job) => j.status === "done").length,
    totalOutputFiles: jobs.reduce((acc: number, j: Job) => acc + (j.resolutions?.length || 0), 0),
  };

  // ── Render Auth Screen if not Logged In (Split Screen UI) ──────────────────────────────
  if (!mounted) {
    return <div style={{ minHeight: "100vh", background: "var(--bg-base)" }} />;
  }

  if (!token) {
    return (
      <div className="auth-container">
        {/* Left Side: Visual / Brand Info */}
        {/* Left Side: Visual / Brand Info */}
        <div className="auth-left-pane">
          {/* Decorative radial blur backdrop */}
          <div style={{
            position: "absolute",
            top: "-20%",
            right: "-20%",
            width: 400,
            height: 400,
            borderRadius: "50%",
            background: "radial-gradient(circle, rgba(37,99,235,0.3) 0%, transparent 70%)",
            filter: "blur(60px)",
            zIndex: 1,
          }} />
          <div style={{
            position: "absolute",
            bottom: "-10%",
            left: "-10%",
            width: 300,
            height: 300,
            borderRadius: "50%",
            background: "radial-gradient(circle, rgba(124,58,237,0.2) 0%, transparent 70%)",
            filter: "blur(50px)",
            zIndex: 1,
          }} />

          <div style={{ position: "relative", zIndex: 10, maxWidth: 480 }}>
            {/* Logo */}
            <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 40 }}>
              <div style={{
                width: "var(--space-10)", height: "var(--space-10)", borderRadius: 10,
                background: "linear-gradient(135deg, #2563eb, #7c3aed)",
                display: "flex", alignItems: "center", justifyContent: "center",
                boxShadow: "0 4px 12px rgba(37,99,235,0.3)",
              }}>
                <Lock size={18} color="#fff" />
              </div>
              <span style={{ fontWeight: 800, fontSize: "var(--text-xl)", letterSpacing: "-0.02em", color: "#fff" }}>
                Pixel<span style={{ color: "#3b82f6" }}>Pipe</span>
              </span>
            </div>

            <h1 style={{
              fontSize: "var(--text-4xl)",
              fontWeight: 900,
              lineHeight: 1.15,
              letterSpacing: "-0.04em",
              marginBottom: "var(--space-6)",
              background: "linear-gradient(135deg, #fff 40%, rgba(255,255,255,0.6) 100%)",
              WebkitBackgroundClip: "text",
              WebkitTextFillColor: "transparent",
            }}>
              Zero-Copy Video Transcoding at Scale
            </h1>

            <p style={{
              fontSize: "var(--text-lg)",
              color: "rgba(255,255,255,0.7)",
              lineHeight: 1.6,
              marginBottom: "var(--space-10)",
            }}>
              Leverage high-performance Linux io_uring kernel bypasses and Cloudflare R2 resumable multipart uploads for blazing-fast chunked transcoding.
            </p>

            {/* Micro stats strip */}
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--space-8)" }}>
              <div>
                <div style={{ fontSize: "var(--text-4xl)", fontWeight: 800, color: "#3b82f6", lineHeight: 1 }}>10×</div>
                <div style={{ fontSize: "var(--text-sm)", color: "rgba(255,255,255,0.5)", marginTop: "var(--space-2)", fontWeight: 500 }}>
                  Faster processing via parallel io_uring chunking
                </div>
              </div>
              <div>
                <div style={{ fontSize: "var(--text-4xl)", fontWeight: 800, color: "#8b5cf6", lineHeight: 1 }}>0 MB</div>
                <div style={{ fontSize: "var(--text-sm)", color: "rgba(255,255,255,0.5)", marginTop: "var(--space-2)", fontWeight: 500 }}>
                  Extra memory buffer or context-switch copies
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Right Side: Auth Form */}
        <div className="auth-right-pane">
          <div style={{ maxWidth: 360, width: "100%", margin: "0 auto" }}>
            <div style={{ marginBottom: "var(--space-8)" }}>
              <h2 style={{ fontSize: "var(--text-2xl)", fontWeight: 800, color: "var(--text-primary)", letterSpacing: "-0.03em" }}>
                {isSignUp ? "Create Your Account" : "Welcome Back"}
              </h2>
              <p style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", marginTop: "var(--space-2)" }}>
                {isSignUp ? "Sign up to start transcoding" : "Sign in to manage your video pipelines"}
              </p>
            </div>

            <form onSubmit={handleAuthSubmit} style={{ display: "flex", flexDirection: "column", gap: 18 }}>
              {authError && (
                <div style={{
                  background: "rgba(239, 68, 68, 0.08)",
                  border: "1px solid rgba(239, 68, 68, 0.2)",
                  color: "#ef4444",
                  padding: "var(--space-2)", borderRadius: 8,
                  fontSize: "var(--text-xs)", textAlign: "center",
                }}>
                  {authError}
                </div>
              )}

              <div>
                <label style={{ display: "block", fontSize: "var(--text-xs)", fontWeight: 600, color: "var(--text-secondary)", marginBottom: "var(--space-2)" }}>
                  Email Address
                </label>
                <input
                  type="email"
                  required
                  value={authEmail}
                  onChange={(e) => setAuthEmail(e.target.value)}
                  placeholder="you@example.com"
                  style={{
                    width: "100%", padding: "var(--space-2) var(--space-4)", borderRadius: 8,
                    border: "1px solid var(--border-light)", background: "var(--bg-subtle)",
                    outline: "none", fontSize: "var(--text-sm)", color: "var(--text-primary)",
                    transition: "border-color 0.2s",
                  }}
                />
              </div>

              <div>
                <label style={{ display: "block", fontSize: "var(--text-xs)", fontWeight: 600, color: "var(--text-secondary)", marginBottom: "var(--space-2)" }}>
                  Password
                </label>
                <input
                  type="password"
                  required
                  value={authPassword}
                  onChange={(e) => setAuthPassword(e.target.value)}
                  placeholder="••••••••"
                  style={{
                    width: "100%", padding: "var(--space-2) var(--space-4)", borderRadius: 8,
                    border: "1px solid var(--border-light)", background: "var(--bg-subtle)",
                    outline: "none", fontSize: "var(--text-sm)", color: "var(--text-primary)",
                    transition: "border-color 0.2s",
                  }}
                />
              </div>

              <button
                type="submit"
                disabled={authLoading}
                className="btn btn-primary"
                style={{ padding: "12px", width: "100%", justifyContent: "center", marginTop: 8 }}
              >
                {authLoading ? "Please wait..." : isSignUp ? "Create Account" : "Sign In"}
              </button>
            </form>

            <div style={{ textAlign: "center", marginTop: 28, fontSize: "0.82rem" }}>
              <span style={{ color: "var(--text-muted)" }}>
                {isSignUp ? "Already have an account?" : "New to the platform?"}
              </span>{" "}
              <button
                onClick={() => {
                  setIsSignUp(!isSignUp);
                  setAuthError("");
                }}
                style={{ background: "none", border: "none", color: "var(--brand-600)", fontSize: "var(--text-sm)", fontWeight: 600, cursor: "pointer", outline: "none" }}
              >
                {isSignUp ? "Sign In" : "Register Now"}
              </button>
            </div>
          </div>
        </div>

        <style>{`
          @media (max-width: 900px) {
            .auth-left-pane { display: none !important; }
            .auth-right-pane { width: 100% !important; height: 100vh !important; }
          }
        `}</style>
      </div>
    );
  }

  // ── Render Dashboard Screen if Authenticated ──────────────────────────
  return (
    <div style={{ display: "flex", minHeight: "100vh", background: "var(--bg-base)" }}>
      {/* Mobile overlay */}
      {sidebarOpen && (
        <div
          onClick={() => setSidebarOpen(false)}
          style={{
            position: "fixed", inset: 0,
            background: "rgba(15,23,42,0.4)",
            zIndex: 99,
            display: "none",
          }}
          id="mobile-overlay"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      {/* Sidebar */}
      <Sidebar
        activeTab={activeTab}
        onTabChange={(tab) => {
          setActiveTab(tab as Tab);
          setSidebarOpen(false);
        }}
      />

      {/* Main content */}
      <div className="dashboard-main">
        {/* Top header bar */}
        <header className="header-bar" style={{
          height: "var(--header-height)",
          background: "var(--bg-glass)",
          backdropFilter: "blur(12px)",
          WebkitBackdropFilter: "blur(12px)",
          borderBottom: "1px solid var(--border-light)",
          display: "flex",
          alignItems: "center",
          position: "sticky",
          top: 0,
          zIndex: 50,
          boxShadow: "var(--shadow-xs)",
        }}>
          {/* Mobile menu button */}
          <button
            onClick={() => setSidebarOpen(!sidebarOpen)}
            style={{
              display: "none",
              padding: 6, borderRadius: 8,
              border: "1px solid var(--border-light)",
              background: "none", cursor: "pointer",
              color: "var(--text-secondary)",
            }}
            id="mobile-menu-toggle"
            aria-label="Toggle sidebar"
          >
            <Menu size={18} />
          </button>

          {/* Breadcrumb / page title */}
          <div style={{ flex: 1 }}>
            <h1 style={{
              fontSize: "var(--text-base)", fontWeight: 700,
              color: "var(--text-primary)", lineHeight: 1.2,
            }}>
              {activeTab === "overview" && "Overview"}
              {activeTab === "upload" && "Upload Video"}
              {activeTab === "jobs" && "Transcoding Jobs"}
              {activeTab === "settings" && "Settings"}
            </h1>
            <p style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
              PixelPipe Dashboard
            </p>
          </div>

          {/* Search bar */}
          <div className="hide-on-mobile" style={{
            display: "flex",
            alignItems: "center",
            gap: 8,
            padding: "7px 14px",
            borderRadius: "var(--radius-full)",
            border: "1px solid var(--border-light)",
            background: "var(--bg-subtle)",
            cursor: "text",
          }}>
            <Search size={14} color="var(--text-muted)" />
            <input
              id="dashboard-search"
              type="text"
              placeholder="Search jobs…"
              style={{
                border: "none",
                background: "transparent",
                outline: "none",
                fontSize: "0.82rem",
                color: "var(--text-primary)",
                width: 160,
              }}
            />
          </div>

          {/* Notification bell */}
          <button
            id="notification-bell"
            aria-label="Notifications"
            style={{
              position: "relative",
              padding: 8, borderRadius: 10,
              border: "1px solid var(--border-light)",
              background: "var(--bg-surface)", cursor: "pointer",
              color: "var(--text-secondary)",
              display: "flex", alignItems: "center",
              transition: "all 0.15s",
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
            <Bell size={16} />
            <span style={{
              position: "absolute", top: 6, right: 6,
              width: 7, height: 7, borderRadius: "50%",
              background: "var(--brand-600)",
              border: "1.5px solid #fff",
            }} />
          </button>

          {/* Sign Out Button */}
          <button
            onClick={handleSignOut}
            aria-label="Sign Out"
            style={{
              padding: "var(--space-2)", borderRadius: 10,
              border: "1px solid var(--border-light)",
              background: "var(--bg-surface)", cursor: "pointer",
              color: "#ef4444",
              display: "flex", alignItems: "center", gap: 6,
              fontSize: "var(--text-xs)", fontWeight: 600,
              transition: "all 0.15s",
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = "rgba(239, 68, 68, 0.08)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = "var(--bg-surface)";
            }}
          >
            <LogOut size={14} />
            <span>Sign Out</span>
          </button>

          {/* User Avatar Initials */}
          <div
            title={userEmail}
            style={{
              width: 34, height: 34, borderRadius: "50%",
              background: "linear-gradient(135deg, #2563eb, #7c3aed)",
              display: "flex", alignItems: "center", justifyContent: "center",
              color: "#fff", fontWeight: 700, fontSize: "var(--text-sm)",
              cursor: "default",
              flexShrink: 0,
            }}
          >
            {userEmail ? userEmail.charAt(0).toUpperCase() : "U"}
          </div>
        </header>

        {/* Page content */}
        <main className="main-content" style={{ flex: 1, maxWidth: 1400, width: "100%" }}>
          {/* ── Overview ── */}
          {activeTab === "overview" && (
            <Overview
              stats={liveStats}
              recentJobs={jobs}
              onGoToJobs={() => setActiveTab("jobs")}
              onGoToUpload={() => setActiveTab("upload")}
            />
          )}

          {/* ── Upload ── */}
          {activeTab === "upload" && (
            <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
              <UploadPanel
                onUploadComplete={handleUploadComplete}
                onFilesChange={handleFilesChange}
              />
              <ChunkProgress files={uploadFiles} />
            </div>
          )}

          {/* ── Jobs ── */}
          {activeTab === "jobs" && (
            <JobsList
              jobs={jobs}
              onSelectJob={setSelectedJob}
              onDeleteJob={handleDeleteJob}
              onRetryJob={handleRetryJob}
            />
          )}

          {/* ── Settings ── */}
          {activeTab === "settings" && (
            <div className="card" style={{ padding: 32, textAlign: "center" }}>
              <div style={{ fontSize: "2rem", marginBottom: 12 }}>⚙️</div>
              <h2 style={{ fontWeight: 700, marginBottom: 8 }}>Settings</h2>
              <p style={{ color: "var(--text-muted)", fontSize: "0.875rem" }}>
                Active user profile: <strong>{userEmail}</strong>. R2 storage bucket configuration and custom API options will appear here.
              </p>
            </div>
          )}
        </main>
      </div>

      {/* Resolution side panel */}
      <ResolutionPanel job={selectedJob} onClose={() => setSelectedJob(null)} />

      <style>{`
        @media (max-width: 768px) {
          aside { transform: translateX(${sidebarOpen ? "0" : "-100%"}); transition: transform 0.25s ease; }
          #mobile-menu-toggle { display: flex !important; }
          #mobile-overlay { display: block !important; }
        }
      `}</style>
    </div>
  );
}
