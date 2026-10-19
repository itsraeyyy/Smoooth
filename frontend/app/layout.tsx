import type { Metadata } from "next";
import "./globals.css";

const siteUrl = "https://www.deepseek.com/en/harness/";
const siteTitle = "DeepSeek Harness | Explore the limits of intelligence";
const siteDescription =
  "Use DeepSeek Harness to work with documents, analyze spreadsheets, write code, and schedule tasks. Extend its capabilities with composable plugins.";

export const metadata: Metadata = {
  metadataBase: new URL("https://www.deepseek.com"),
  title: siteTitle,
  description: siteDescription,
  applicationName: "DeepSeek Harness",
  keywords: [
    "DeepSeek",
    "AGI",
    "AI",
    "LLM",
    "DeepSeek V4",
    "DeepSeek R1",
    "DeepSeek API",
    "DeepSeek Chat",
    "open source",
    "artificial intelligence",
  ],
  alternates: {
    canonical: siteUrl,
    languages: {
      "zh-CN": "https://www.deepseek.com/harness/",
      "en-US": "https://www.deepseek.com/en/harness/",
      "x-default": "https://www.deepseek.com/harness/",
    },
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
    siteName: "DeepSeek Harness",
    locale: "en_US",
    type: "website",
    images: [
      {
        url: "/harness/images/deepseek-harness-social.jpg",
        width: 1200,
        height: 630,
        alt: "DeepSeek Harness",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: siteTitle,
    description: siteDescription,
    images: ["/harness/images/deepseek-harness-social.jpg"],
  },
  robots: {
    index: true,
    follow: true,
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en-US">
      <head>
        <link rel="icon" href="/harness/favicon.svg" type="image/svg+xml" />
        <link rel="icon" href="/favicon.ico" type="image/x-icon" />
      </head>
      <body className="antialiased min-h-screen bg-[#080C14] text-[#F9FAFB]">
        {children}
      </body>
    </html>
  );
}
