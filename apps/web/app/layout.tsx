import type { Metadata } from "next";
import { ThemeProvider } from "../components/ThemeProvider";
import { ReactQueryProvider } from "../components/ReactQueryProvider";
import "./globals.css";

export const metadata: Metadata = {
  title: "PixelPipe — Ultra-Fast Automated Video Transcoding",
  description:
    "Ultra-fast automated video transcoding pipeline. Upload once, get every resolution automatically — 4K, 1080p, 720p, 480p, 360p.",
  keywords: ["video transcoding", "automated", "cloud transcoding", "pipeline", "HLS", "4K", "adaptive streaming"],
  openGraph: {
    images: ["/zero_copy.png"],
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link
          href="https://fonts.googleapis.com/css2?family=Nunito:wght@300;400;500;600;700;800;900&display=swap"
          rel="stylesheet"
        />
      </head>
      <body suppressHydrationWarning>
        <ReactQueryProvider>
          <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
            {children}
          </ThemeProvider>
        </ReactQueryProvider>
      </body>
    </html>
  );
}

