import type { LayoutProps, Metadata } from "@farm.js/core";
import "./globals.css";

const siteUrl = "https://smoooth.dev";
const siteTitle = "Smoooth — Infrastructure Runtime for AI Coding Agents";
const siteDescription =
  "Deploy stateful apps on your own Linux VPS with Claude Code, Cursor, and Windsurf over MCP. Sandboxed Docker, managed databases, zero cloud markups.";

export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),
  title: siteTitle,
  description: siteDescription,
  authors: [{ name: "Smoooth Core Team", url: siteUrl }],
  creator: "Smoooth",
  publisher: "Smoooth",
  keywords: [
    "Smoooth",
    "infrastructure runtime",
    "AI coding agents",
    "Model Context Protocol",
    "MCP server",
    "Claude Code",
    "Cursor",
    "Windsurf",
    "Docker deployment",
    "PostgreSQL VPS",
    "Redis",
    "self-hosted PaaS",
    "Linux VPS",
    "autonomous DevOps",
    "open source"
  ],
  alternates: {
    canonical: siteUrl,
  },
  icons: {
    icon: [
      { url: "/harness/favicon.svg", type: "image/svg+xml" },
      { url: "/favicon.ico", type: "image/x-icon" },
    ],
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
        url: "/og-image.png",
        width: 1200,
        height: 630,
        alt: "Smoooth — Infrastructure Runtime for AI Coding Agents",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: siteTitle,
    description: siteDescription,
    creator: "@smoooth_dev",
    images: ["/og-image.png"],
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
};

export default function RootLayout({ children }: LayoutProps) {
  const jsonLd = {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "SoftwareApplication",
        "name": "Smoooth",
        "url": siteUrl,
        "operatingSystem": "Linux (Ubuntu 22.04/24.04, Debian 12), macOS, Windows",
        "applicationCategory": "DeveloperApplication",
        "applicationSubCategory": "Infrastructure & DevOps Runtime",
        "description": siteDescription,
        "license": "https://opensource.org/licenses/Apache-2.0",
        "offers": {
          "@type": "Offer",
          "price": "0",
          "priceCurrency": "USD"
        },
        "featureList": [
          "Model Context Protocol (MCP) Server Interface",
          "Docker Container Lifecycle Orchestration",
          "Stateful Database Adapters (PostgreSQL, Redis, MongoDB, SQLite)",
          "256-bit Scoped Cryptographic Bearer Tokens",
          "Transactional 2-Phase DAG Deployments with Rollbacks",
          "Zero Cloud Lock-In Self-Hosted Runtime",
          "Loopback Isolation (127.0.0.1:4040)"
        ]
      },
      {
        "@type": "Organization",
        "name": "Smoooth",
        "url": siteUrl,
        "logo": `${siteUrl}/harness/favicon.svg`,
        "sameAs": [
          "https://github.com/itsraeyyy/Smoooth"
        ]
      }
    ]
  };

  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <link rel="icon" href="/harness/favicon.svg" type="image/svg+xml" />
        <link rel="icon" href="/favicon.ico" type="image/x-icon" />
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
        />
      </head>
      <body className="antialiased min-h-screen bg-[#0a0a0a] text-[#F9FAFB]" suppressHydrationWarning>
        {children}
      </body>
    </html>
  );
}
