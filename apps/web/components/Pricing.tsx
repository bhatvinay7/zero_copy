"use client";

import Link from "next/link";
import { Check, Zap } from "lucide-react";

const plans = [
  {
    id: "starter",
    name: "Starter",
    price: "Free",
    period: "",
    description: "Perfect for developers and small projects.",
    color: "#64748b",
    accentBg: "var(--bg-muted)",
    cta: "Get Started Free",
    ctaVariant: "outline" as const,
    features: [
      "5 GB R2 storage",
      "Up to 720p output",
      "10 uploads / month",
      "TUS resumable uploads",
      "720p, 480p, 360p",
      "Community support",
    ],
    badge: null,
  },
  {
    id: "pro",
    name: "Pro",
    price: "$29",
    period: "/month",
    description: "For content creators and growing teams.",
    color: "#2563eb",
    accentBg: "linear-gradient(135deg, #2563eb, #7c3aed)",
    cta: "Start 14-Day Trial",
    ctaVariant: "primary" as const,
    features: [
      "500 GB R2 storage",
      "4K UHD output",
      "Unlimited uploads",
      "Priority transcoding queue",
      "All 5 resolutions",
      "HLS manifest generation",
      "Webhook notifications",
      "Email support",
    ],
    badge: "Most Popular",
  },
  {
    id: "enterprise",
    name: "Enterprise",
    price: "Custom",
    period: "",
    description: "Dedicated infrastructure for high-volume workloads.",
    color: "#7c3aed",
    accentBg: "var(--bg-muted)",
    cta: "Contact Sales",
    ctaVariant: "outline" as const,
    features: [
      "Unlimited R2 storage",
      "4K UHD + 8K roadmap",
      "Dedicated high-performance workers",
      "Custom codec profiles",
      "SLA 99.9% uptime",
      "On-prem deployment option",
      "SSO / SAML auth",
      "24/7 dedicated support",
    ],
    badge: null,
  },
];

export default function Pricing() {
  return (
    <section id="pricing" className="section" style={{ background: "var(--bg-surface)" }}>
      <div className="container">
        {/* Header */}
        <div style={{ textAlign: "center", marginBottom: "var(--space-16)" }}>
          <span className="badge badge-blue" style={{ marginBottom: 16 }}>Pricing</span>
          <h2 style={{ fontSize: "clamp(1.9rem, 4vw, 2.8rem)", fontWeight: 800, letterSpacing: "-0.03em", marginBottom: 16 }}>
            Simple, transparent pricing
          </h2>
          <p style={{ fontSize: "1.05rem", color: "var(--text-secondary)", maxWidth: 480, margin: "0 auto" }}>
            No hidden fees. Pay for what you use. Cancel anytime.
          </p>
        </div>

        {/* Cards */}
        <div style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))",
          gap: 24,
          alignItems: "stretch",
        }}>
          {plans.map((plan) => {
            const isPro = plan.id === "pro";
            return (
              <div
                key={plan.id}
                className="card"
                style={{
                  padding: 0,
                  position: "relative",
                  border: isPro ? `2px solid ${plan.color}` : "1px solid var(--border-light)",
                  boxShadow: isPro ? `0 12px 48px ${plan.color}20` : "var(--shadow-sm)",
                  display: "flex",
                  flexDirection: "column",
                }}
              >
                {/* Popular badge */}
                {plan.badge && (
                  <div style={{
                    position: "absolute",
                    top: -14, left: "50%", transform: "translateX(-50%)",
                    background: "linear-gradient(135deg, #2563eb, #7c3aed)",
                    color: "#fff",
                    padding: "4px 18px",
                    borderRadius: "var(--radius-full)",
                    fontSize: "0.75rem",
                    fontWeight: 700,
                    whiteSpace: "nowrap",
                    boxShadow: "var(--shadow-brand)",
                    display: "flex",
                    alignItems: "center",
                    gap: 5,
                  }}>
                    <Zap size={11} fill="#fff" />
                    {plan.badge}
                  </div>
                )}

                {/* Header */}
                <div
                  style={{
                    padding: "28px 28px 24px",
                    borderRadius: "var(--radius-lg) var(--radius-lg) 0 0",
                    background: isPro ? "var(--bg-pro-gradient)" : "var(--bg-subtle)",
                    borderBottom: "1px solid var(--border-light)",
                  }}
                >
                  <h3 style={{ fontWeight: 700, fontSize: "1rem", marginBottom: 4, color: plan.color }}>{plan.name}</h3>
                  <p style={{ fontSize: "0.83rem", color: "var(--text-secondary)", marginBottom: 16 }}>{plan.description}</p>
                  <div style={{ display: "flex", alignItems: "baseline", gap: 4 }}>
                    <span style={{ fontSize: "2.4rem", fontWeight: 900, color: "var(--text-primary)", letterSpacing: "-0.04em" }}>
                      {plan.price}
                    </span>
                    {plan.period && (
                      <span style={{ fontSize: "0.9rem", color: "var(--text-muted)", fontWeight: 500 }}>{plan.period}</span>
                    )}
                  </div>
                </div>

                {/* Features */}
                <div style={{ padding: "24px 28px", flex: 1 }}>
                  <ul style={{ listStyle: "none", display: "flex", flexDirection: "column", gap: 10 }}>
                    {plan.features.map((f) => (
                      <li key={f} style={{ display: "flex", alignItems: "center", gap: 10, fontSize: "0.875rem", color: "var(--text-secondary)" }}>
                        <span style={{
                          width: 18, height: 18, borderRadius: "50%",
                          background: `${plan.color}15`,
                          display: "flex", alignItems: "center", justifyContent: "center",
                          flexShrink: 0,
                        }}>
                          <Check size={11} color={plan.color} strokeWidth={3} />
                        </span>
                        {f}
                      </li>
                    ))}
                  </ul>
                </div>

                {/* CTA */}
                <div style={{ padding: "0 28px 28px" }}>
                  <Link
                    href="/dashboard"
                    id={`pricing-cta-${plan.id}`}
                    className={`btn ${isPro ? "btn-primary" : "btn-outline"}`}
                    style={{ width: "100%", justifyContent: "center" }}
                  >
                    {plan.cta}
                  </Link>
                </div>
              </div>
            );
          })}
        </div>

        {/* Bottom note */}
        <p style={{ textAlign: "center", marginTop: 32, fontSize: "0.83rem", color: "var(--text-muted)" }}>
          All plans include resumable uploads, secure cloud storage, and ultra-fast parallel transcoding.
          <br />
          Storage overages billed at $0.015 / GB beyond plan limit.
        </p>
      </div>
    </section>
  );
}
