"use client";

import Navbar from "../components/Navbar";
import Hero from "../components/Hero";
import Features from "../components/Features";
import HowItWorks from "../components/HowItWorks";
import Pricing from "../components/Pricing";
import Footer from "../components/Footer";

// Stats strip between features and how-it-works
function StatsStrip() {
  const stats = [
    { value: "10×", label: "Faster than FFmpeg-only", color: "#2563eb" },
    { value: "0 MB", label: "Extra memory buffer", color: "#7c3aed" },
    { value: "5", label: "Resolutions per upload", color: "#059669" },
    { value: "< 50ms", label: "Pipeline startup latency", color: "#d97706" },
    { value: "99.9%", label: "Uptime SLA", color: "#0891b2" },
  ];

  return (
    <div style={{
      background: "linear-gradient(135deg, #0f172a 0%, #1e1b4b 100%)",
      padding: "var(--space-10) 0",
    }}>
      <div className="container">
        <div style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fit, minmax(140px, 1fr))",
          gap: 24,
          textAlign: "center",
        }}>
          {stats.map((s) => (
            <div key={s.label}>
              <div style={{
                fontSize: "2.2rem",
                fontWeight: 900,
                letterSpacing: "-0.04em",
                color: s.color,
                marginBottom: 6,
                lineHeight: 1,
              }}>
                {s.value}
              </div>
              <div style={{ fontSize: "0.78rem", color: "rgba(255,255,255,0.5)", fontWeight: 500 }}>
                {s.label}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

export default function HomePage() {
  return (
    <>
      <Navbar />
      <main>
        <Hero />
        <Features />
        <StatsStrip />
        <HowItWorks />
        <Pricing />
      </main>
      <Footer />
    </>
  );
}
