import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Vorcel AI — AI-Powered Coding Without the Complexity",
  description: "Accelerate development workflows with AI-powered code generation, debugging, documentation, and intelligent suggestions built directly into your coding environment.",
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
          href="https://fonts.googleapis.com/css2?family=Instrument+Serif:ital@0;1&family=JetBrains+Mono:wght@400;500;600&family=Lexend:wght@300;400;500;600;700;800;900&display=swap"
          rel="stylesheet"
        />
      </head>
      <body className="antialiased min-h-screen bg-white text-[#111111]">
        {children}
      </body>
    </html>
  );
}
