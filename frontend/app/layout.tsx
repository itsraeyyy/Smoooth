import type { Metadata } from "next";
import "./globals.css";

const siteUrl = "https://smoooth.dev";
const siteTitle = "Smoooth — Self-Hosted AI Agent VPS Infrastructure & MCP Runtime";
const siteDescription =
  "Deploy Docker containers, Postgres, Redis, and APIs directly from Claude Code, Cursor, and Windsurf onto your Linux VPS. Open-source, zero cloud lock-in.";

export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),
  title: {
    default: siteTitle,
    template: "%s | Smoooth",
  },
  description: siteDescription,
  applicationName: "Smoooth",
  authors: [{ name: "Smoooth Core Team", url: siteUrl }],
  creator: "Smoooth",
  publisher: "Smoooth",
  keywords: [
    "self-hosted AI agent infrastructure",
    "mcp server docker deploy",
    "deploy docker containers claude code",
    "cursor mcp server deployment",
    "autonomous coding agent vps runtime",
    "ai agent database adapter",
    "postgres pgvector docker deployment",
    "open source e2b alternative",
    "linux vps agent runtime",
    "model context protocol server",
  ],
  alternates: {
    canonical: siteUrl,
  },
  openGraph: {
    title: siteTitle,
    description: siteDescription,
    url: siteUrl,
    siteName: "Smoooth",
    locale: "en_US",
    type: "website",
    images: [
      {
        url: "/desktop.png",
        width: 1200,
        height: 630,
        alt: "Smoooth — Self-Hosted AI Agent VPS Infrastructure & MCP Runtime",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: siteTitle,
    description: siteDescription,
    images: ["/desktop.png"],
    creator: "@smoooth_dev",
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      "max-video-preview": -1,
      "max-image-preview": "large",
      "max-snippet": -1,
    },
  },
  category: "technology",
};

const jsonLd = {
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "SoftwareApplication",
      "@id": "https://smoooth.dev/#software",
      name: "Smoooth",
      applicationCategory: "DeveloperApplication",
      operatingSystem: "Linux (Ubuntu 22.04+, Debian 12+)",
      url: siteUrl,
      description: siteDescription,
      license: "https://opensource.org/licenses/Apache-2.0",
      offers: {
        "@type": "Offer",
        price: "0",
        priceCurrency: "USD",
        availability: "https://schema.org/InStock",
      },
      featureList: [
        "Model Context Protocol (MCP) Server for Docker",
        "Multi-Database Provisioning (PostgreSQL with pgvector, Redis, MongoDB, SQLite)",
        "Scoped 256-bit Bearer Token Security & RBAC",
        "Transactional 2-Phase DAG Container Deployments",
        "Direct pairing with Claude Code, Cursor, and Windsurf",
        "Local-First Architecture on developer-owned Linux VPS",
      ],
    },
    {
      "@type": "WebSite",
      "@id": "https://smoooth.dev/#website",
      url: siteUrl,
      name: "Smoooth",
      description: siteDescription,
      publisher: {
        "@type": "Organization",
        name: "Smoooth",
        url: siteUrl,
      },
    },
  ],
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
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
        />
      </head>
      <body className="antialiased min-h-screen bg-white text-[#111111]">
        {children}
      </body>
    </html>
  );
}
