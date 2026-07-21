import Link from "next/link";
import Image from "next/image";
import { ArrowRight, Play, Zap, Shield, Cpu } from "lucide-react";

export default function Hero() {
  return (
    <section
      className="landing-bg"
      style={{
        minHeight: "100vh",
        display: "flex",
        alignItems: "center",
        paddingTop: 80,
        position: "relative",
        overflow: "hidden",
      }}
    >
      {/* Decorative blobs */}
      <div aria-hidden style={{
        position: "absolute", top: "5%", right: "-15%",
        width: 700, height: 700,
        backgroundImage: "url('/text-mask.png')",
        backgroundSize: "cover",
        backgroundPosition: "center",
        opacity: 0.25,
        WebkitMaskImage: "radial-gradient(circle, black 0%, transparent 60%)",
        maskImage: "radial-gradient(circle, black 0%, transparent 60%)",
        pointerEvents: "none",
        mixBlendMode: "screen",
      }} />
      <div aria-hidden style={{
        position: "absolute", bottom: "-10%", left: "-15%",
        width: 600, height: 600,
        backgroundImage: "url('/text-mask.png')",
        backgroundSize: "cover",
        backgroundPosition: "center",
        opacity: 0.2,
        WebkitMaskImage: "radial-gradient(circle, black 0%, transparent 60%)",
        maskImage: "radial-gradient(circle, black 0%, transparent 60%)",
        pointerEvents: "none",
        mixBlendMode: "screen",
      }} />

      <div className="container" style={{ padding: "var(--space-16) var(--space-6)", width: "100%" }}>
        <div style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: "var(--space-16)",
          alignItems: "center",
        }}>
          {/* Left: text */}
          <div>
            {/* Eyebrow badge */}
            <div className="badge badge-blue animate-fade-in" style={{ marginBottom: 24, opacity: 0 }}>
              <Zap size={12} />
              Fast, Secure, Resumable Transcoding
            </div>

            <h1
              className="animate-fade-in-up"
              style={{
                fontSize: "clamp(2.4rem, 5vw, 3.6rem)",
                fontWeight: 900,
                letterSpacing: "-0.04em",
                lineHeight: 1.08,
                marginBottom: 24,
                opacity: 0,
                animationDelay: "100ms",
              }}
            >
              Video Transcoding at{" "}
              <span className="gradient-text">Extreme Speed</span>
              <br />
              with Instant Multi-Resolution Output
            </h1>

            <p
              className="animate-fade-in-up"
              style={{
                fontSize: "1.15rem",
                color: "var(--text-secondary)",
                maxWidth: 480,
                marginBottom: 36,
                opacity: 0,
                animationDelay: "200ms",
                lineHeight: 1.7,
              }}
            >
              Upload any video and get every resolution automatically — 4K, 1080p, 720p,
              480p, 360p. No complex setups, no lag. Just pure, blazing-fast
              video processing built for modern platforms.
            </p>

            <div
              className="animate-fade-in-up"
              style={{
                display: "flex",
                gap: 14,
                flexWrap: "wrap",
                opacity: 0,
                animationDelay: "300ms",
              }}
            >
              <Link
                href="/dashboard"
                className="btn btn-primary btn-lg"
                id="hero-cta-primary"
                style={{ display: "flex", alignItems: "center", gap: 8 }}
              >
                Start Transcoding Free
                <ArrowRight size={18} />
              </Link>
              <a
                href="#how-it-works"
                className="btn btn-ghost btn-lg"
                id="hero-cta-secondary"
                style={{ display: "flex", alignItems: "center", gap: 8 }}
              >
                <Play size={16} />
                See How It Works
              </a>
            </div>

            {/* Trust signals */}
            <div
              className="animate-fade-in"
              style={{
                display: "flex",
                gap: 24,
                marginTop: 40,
                opacity: 0,
                animationDelay: "500ms",
              }}
            >
              {[
                { icon: <Zap size={14} />, text: "10× Faster than standard servers" },
                { icon: <Shield size={14} />, text: "TLS + secure resumable uploads" },
                { icon: <Cpu size={14} />, text: "Instant parallel scaling" },
              ].map((item) => (
                <div
                  key={item.text}
                  style={{ display: "flex", alignItems: "center", gap: 6, color: "var(--text-secondary)", fontSize: "0.8rem", fontWeight: 500 }}
                >
                  <span style={{ color: "var(--brand-500)" }}>{item.icon}</span>
                  {item.text}
                </div>
              ))}
            </div>
          </div>

          {/* Right: pipeline visualisation or image */}
          <div
            className="animate-fade-in animate-float"
            style={{
              opacity: 0,
              animationDelay: "400ms",
              animationFillMode: "forwards",
              position: "relative",
              display: "flex",
              justifyContent: "center",
              alignItems: "center"
            }}
          >
            <div style={{ position: "relative", width: "100%", maxWidth: 540, aspectRatio: "4/3" }}>
              <Image 
                src="/hero_illustration.png"
                alt="Video Transcoding Speed"
                fill
                priority
                style={{
                  objectFit: "contain",
                  filter: "drop-shadow(0 20px 40px rgba(0,0,0,0.15))"
                }}
              />
            </div>
          </div>
        </div>
      </div>

      <style>{`
        @media (max-width: 900px) {
          .hero-grid { grid-template-columns: 1fr !important; }
          .hero-viz { display: none; }
        }
        .animate-fade-in-up { animation-fill-mode: forwards; }
        .animate-fade-in { animation-fill-mode: forwards; }
      `}</style>
    </section>
  );
}
