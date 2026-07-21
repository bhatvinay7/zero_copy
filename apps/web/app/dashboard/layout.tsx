import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Dashboard — PixelPipe",
  description: "Upload videos, monitor transcoding jobs and download multi-resolution outputs.",
};

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  return <>{children}</>;
}
