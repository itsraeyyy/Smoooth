import type { MetadataRoute } from "@farm.js/core";

export default function robots(): MetadataRoute.Robots {
  const baseUrl = 'https://smoooth.dev';

  return {
    rules: [
      {
        userAgent: '*',
        allow: '/',
        disallow: ['/api/', '/private/'],
      },
    ],
    sitemap: `${baseUrl}/sitemap.xml`,
    host: baseUrl,
  };
}
