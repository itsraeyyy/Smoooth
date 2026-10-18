import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Smoooth — Infrastructure Freedom, Built for AI Agents",
  description: "Deploy and operate applications on infrastructure you control. Smoooth connects Claude Code, Cursor, and Windsurf directly to your Linux VPS over MCP with Docker and multi-database persistence.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link
          href="https://fonts.googleapis.com/css2?family=Geist:wght@400;500;600&family=Geist+Mono:wght@400;500;600&family=Lexend:wght@300;400;500;600;700;800;900&family=Silkscreen&display=swap"
          rel="stylesheet"
        />
      </head>
      <body className="antialiased min-h-screen bg-white text-[#111111]">
        {children}
      </body>
    </html>
  );
}
