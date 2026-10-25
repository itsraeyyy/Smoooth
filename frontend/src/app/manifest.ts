import type { MetadataRoute } from "@farm.js/core";

export default function manifest(): MetadataRoute.Manifest {
  return {
    name: 'Smoooth — Self-Hosted AI Agent VPS Infrastructure',
    short_name: 'Smoooth',
    description: 'Deploy Docker containers, Postgres, Redis, and APIs directly from Claude Code, Cursor, and Windsurf onto your Linux VPS over MCP.',
    start_url: '/',
    display: 'standalone',
    background_color: '#FFFFFF',
    theme_color: '#111111',
    icons: [
      {
        src: '/favicon.ico',
        sizes: 'any',
        type: 'image/x-icon',
      },
    ],
  };
}
