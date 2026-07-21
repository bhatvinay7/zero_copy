"use client";
import Link from "next/link";
import Image from "next/image";
import { Zap, Code2, AtSign, BookOpen } from "lucide-react";

const links = {
  Product: [
    { label: "Features", href: "#features" },
    { label: "How It Works", href: "#how-it-works" },
    { label: "Pricing", href: "#pricing" },
    { label: "Changelog", href: "#" },
  ],
  Developers: [
    { label: "Documentation", href: "#" },
    { label: "API Reference", href: "#" },
    { label: "SDK", href: "#" },
    { label: "Status", href: "#" },
  ],
  Company: [
    { label: "About", href: "#" },
    { label: "Blog", href: "#" },
    { label: "Privacy Policy", href: "#" },
    { label: "Terms of Service", href: "#" },
  ],
};

export default function Footer() {
  return (
    <footer style={{
      background: "#0f172a",
      color: "#fff",
      padding: "var(--space-20) 0 var(--space-10)",
    }}>
      <div className="container">
        {/* Top grid */}
        <div style={{
          display: "grid",
          gridTemplateColumns: "2fr 1fr 1fr 1fr",
          gap: 48,
          paddingBottom: 48,
          borderBottom: "1px solid rgba(255,255,255,0.08)",
        }}>
          {/* Brand */}
          <div>
            <Link href="/" style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 18, textDecoration: "none" }}>
              <span style={{ fontWeight: 800, fontSize: "1.05rem", letterSpacing: "-0.02em" }}>
                Pixel<span style={{ color: "#60a5fa" }}>Pipe</span>
              </span>
            </Link>
            <p style={{ fontSize: "0.875rem", color: "rgba(255,255,255,0.5)", lineHeight: 1.7, maxWidth: 280, marginBottom: 24 }}>
              High-performance video transcoding pipeline. Upload once and automatically receive optimized playback resolutions for any device.
            </p>

            {/* Tech stack badges */}
            <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
              {["Auto-Scaling", "CDN Delivery", "Resumable Uploads", "HLS Adaptive Streaming", "Parallel Engine"].map((t) => (
                <span
                  key={t}
                  style={{
                    fontSize: "0.7rem", fontWeight: 600,
                    padding: "3px 10px",
                    borderRadius: "var(--radius-full)",
                    border: "1px solid rgba(255,255,255,0.12)",
                    color: "rgba(255,255,255,0.5)",
                  }}
                >
                  {t}
                </span>
              ))}
            </div>
          </div>

          {/* Link columns */}
          {Object.entries(links).map(([category, items]) => (
            <div key={category}>
              <h4 style={{ fontSize: "0.78rem", fontWeight: 700, color: "rgba(255,255,255,0.35)", letterSpacing: "0.08em", textTransform: "uppercase", marginBottom: 16 }}>
                {category}
              </h4>
              <ul style={{ listStyle: "none", display: "flex", flexDirection: "column", gap: 10 }}>
                {items.map((item) => (
                  <li key={item.label}>
                    <a
                      href={item.href}
                      style={{
                        fontSize: "0.875rem",
                        color: "rgba(255,255,255,0.55)",
                        transition: "color 0.2s",
                        textDecoration: "none",
                      }}
                      onMouseEnter={(e) => (e.currentTarget.style.color = "#fff")}
                      onMouseLeave={(e) => (e.currentTarget.style.color = "rgba(255,255,255,0.55)")}
                    >
                      {item.label}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        {/* Bottom bar */}
        <div style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          paddingTop: 32,
          flexWrap: "wrap",
          gap: 16,
        }}>
          <p style={{ fontSize: "0.8rem", color: "rgba(255,255,255,0.3)" }}>
            © {new Date().getFullYear()} PixelPipe. Built for speed and reliability ❤️
          </p>

          <div style={{ display: "flex", gap: 16, alignItems: "center" }}>
            {[
              { icon: <Code2 size={18} />, label: "GitHub", href: "#" },
              { icon: <AtSign size={18} />, label: "Twitter", href: "#" },
              { icon: <BookOpen size={18} />, label: "Docs", href: "#" },
            ].map((item) => (
              <a
                key={item.label}
                href={item.href}
                aria-label={item.label}
                style={{
                  color: "rgba(255,255,255,0.35)",
                  transition: "color 0.2s",
                }}
                onMouseEnter={(e) => (e.currentTarget.style.color = "#fff")}
                onMouseLeave={(e) => (e.currentTarget.style.color = "rgba(255,255,255,0.35)")}
              >
                {item.icon}
              </a>
            ))}
          </div>
        </div>
      </div>
    </footer>
  );
}
