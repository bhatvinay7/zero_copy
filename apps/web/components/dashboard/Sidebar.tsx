"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import Image from "next/image";
import { ThemeToggle } from "../ThemeToggle";
import {
  Zap,
  LayoutDashboard,
  Upload,
  Briefcase,
  Settings,
  LogOut,
  HardDrive,
  ChevronRight,
} from "lucide-react";

const NAV = [
  { href: "/dashboard", icon: <LayoutDashboard size={18} />, label: "Overview" },
  { href: "/dashboard?tab=upload", icon: <Upload size={18} />, label: "Upload" },
  { href: "/dashboard?tab=jobs", icon: <Briefcase size={18} />, label: "Jobs" },
  { href: "/dashboard?tab=settings", icon: <Settings size={18} />, label: "Settings" },
];

interface SidebarProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
}

export default function Sidebar({
  activeTab,
  onTabChange,
}: SidebarProps) {
  return (
    <aside
      style={{
        width: "var(--sidebar-width)",
        minHeight: "100vh",
        background: "var(--bg-glass)",
        backdropFilter: "blur(16px)",
        WebkitBackdropFilter: "blur(16px)",
        borderRight: "1px solid var(--border-light)",
        display: "flex",
        flexDirection: "column",
        position: "fixed",
        left: 0,
        top: 0,
        bottom: 0,
        zIndex: 100,
        overflowY: "auto",
      }}
    >
      {/* Logo */}
      <div style={{ padding: "var(--space-6) var(--space-6) var(--space-4)", borderBottom: "1px solid var(--border-light)" }}>
        <Link href="/" style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
          <div>
            <div style={{ fontWeight: 800, fontSize: "var(--text-base)", letterSpacing: "-0.02em", color: "var(--text-primary)", lineHeight: 1.2 }}>
              Pixel<span style={{ color: "var(--brand-600)" }}>Pipe</span>
            </div>
            <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", fontWeight: 500 }}>Automated · High-Speed</div>
          </div>
        </Link>
      </div>

      {/* Nav */}
      <nav style={{ padding: "var(--space-4) var(--space-4)", flex: 1 }}>
        <div style={{ fontSize: "var(--text-xs)", fontWeight: 700, color: "var(--text-muted)", letterSpacing: "0.09em", textTransform: "uppercase", padding: "var(--space-2) var(--space-4) var(--space-2)" }}>
          Menu
        </div>
        {NAV.map((item) => {
          const tab = item.href.includes("?tab=")
            ? item.href.split("?tab=")[1]!
            : "overview";
          const isActive = activeTab === tab;

          return (
            <button
              key={item.label}
              id={`sidebar-nav-${tab}`}
              onClick={() => onTabChange(tab)}
              style={{
                width: "100%",
                display: "flex",
                alignItems: "center",
                gap: "var(--space-2)",
                padding: "var(--space-2) var(--space-4)",
                borderRadius: "var(--radius-md)",
                marginBottom: 2,
                border: "none",
                cursor: "pointer",
                fontWeight: 600,
                fontSize: "var(--text-sm)",
                transition: "all 0.18s",
                background: isActive
                  ? "var(--sidebar-active-bg)"
                  : "transparent",
                color: isActive ? "var(--brand-600)" : "var(--text-secondary)",
                textAlign: "left",
              }}
              onMouseEnter={(e) => {
                if (!isActive) {
                  e.currentTarget.style.background = "var(--bg-muted)";
                  e.currentTarget.style.color = "var(--text-primary)";
                }
              }}
              onMouseLeave={(e) => {
                if (!isActive) {
                  e.currentTarget.style.background = "transparent";
                  e.currentTarget.style.color = "var(--text-secondary)";
                }
              }}
            >
              <span style={{ opacity: isActive ? 1 : 0.7 }}>{item.icon}</span>
              {item.label}
              {isActive && (
                <ChevronRight size={14} style={{ marginLeft: "auto", opacity: 0.5 }} />
              )}
            </button>
          );
        })}
      </nav>

      {/* User */}
      <div style={{
        padding: "var(--space-4) var(--space-6) var(--space-6)",
        borderTop: "1px solid var(--border-light)",
        display: "flex",
        alignItems: "center",
        gap: "var(--space-2)",
      }}>
        <div style={{
          width: 34, height: 34, borderRadius: "50%",
          background: "linear-gradient(135deg, #2563eb, #7c3aed)",
          display: "flex", alignItems: "center", justifyContent: "center",
          color: "#fff", fontWeight: 700, fontSize: "var(--text-sm)",
          flexShrink: 0,
        }}>
          U
        </div>
        <div style={{ flex: 1, overflow: "hidden" }}>
          <div style={{ fontWeight: 600, fontSize: "var(--text-sm)", color: "var(--text-primary)", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
            User Account
          </div>
          <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>Pro Plan</div>
        </div>
        <ThemeToggle />
        <button
          aria-label="Sign out"
          style={{
            color: "var(--text-muted)",
            padding: 4,
            borderRadius: 6,
            border: "none",
            background: "none",
            cursor: "pointer",
            transition: "color 0.15s",
          }}
          onMouseEnter={(e) => (e.currentTarget.style.color = "var(--rose-500)")}
          onMouseLeave={(e) => (e.currentTarget.style.color = "var(--text-muted)")}
        >
          <LogOut size={16} />
        </button>
      </div>
    </aside>
  );
}
