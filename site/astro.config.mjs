// @ts-check

import cloudflare from "@astrojs/cloudflare";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import favicons from "astro-favicons";
import { DocsAssets } from "@knitli/docs-components";

const { headlineLogoDark, headlinelogoLight, recocoLogoMed, knitliWordmark, Favicon } = DocsAssets;

// https://astro.build/config
export default defineConfig({
  site: "https://docs.knitli.com",
  base: "/recoco",
  adapter: cloudflare({
    imageService: "compile",
    environment: process.env.NODE_ENV === "development" ? "local" : undefined,
  }),
  // Image optimization
  image: {
    service: {
      entrypoint: "astro/assets/services/sharp",
    },
    responsiveStyles: true,
    layout: "constrained",
    domains: [
      "github.com",
      "raw.githubusercontent.com",
      "docs.knitli.com",
      "knitli.com",
    ],
  },

  // Build optimizations
  build: {
    inlineStylesheets: "auto",
    assets: "_astro",
  },
  markdown: {
    shikiConfig: { theme: "github-dark" },
  },
  // Vite configuration for better bundling
  vite: {
    assetsInclude: [
      "src/*.webp",
      "src/*.png",
      "src/*.jpg",
      "src/*.jpeg",
      "src/*.svg",
      "src/*.avif",
    ],
    build: {
      cssCodeSplit: true,
      cssMinify: "lightningcss",

      rolldownOptions: {
        output: {
          experimental: {
            nativeMagicString: true,
          },
        },
        treeshake: "smallest",
        optimization: {
          inlineConst: "smart",
        },
        ssr: false,
      },
    },
    css: {
      lightningcss: {},
    },
  },
  prefetch: {
    defaultStrategy: "viewport",
  },
  experimental: {
    chromeDevtoolsWorkspace: true,
    clientPrerender: true,
    contentIntellisense: true,
    svgo: {
      plugins: [
        {
          name: "preset-default",
          params: {
            overrides: {
              removeMetadata: false,
            },
          },
        },
      ],
    },
    headingIdCompat: true,
    preserveScriptOrder: true,
  },
  // Static site generation for Cloudflare
  output: "static",
  integrations: [
    starlight({
      title: "Recoco Docs",
      logo: {
        dark: headlineLogoDark,
        light: headlinelogoLight,
      },
      description:
        "Incremental ETL and data processing framework in pure Rust. Feature-gated, modular architecture for sources, targets, and functions.",
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/knitli/recoco",
        },
      ],
      components: {
        Footer: '@knitli/docs-components/Footer.astro',
        PageFrame: '@knitli/docs-components/PageFrame.astro',
      },
      customCss: [],
      head: [
        {
          tag: "meta",
          attrs: {
            property: "og:image",
            content: "https://docs.knitli.com/recoco/og-image.png",
          },
        },
        {
          tag: "meta",
          attrs: {
            property: "twitter:card",
            content: "summary_large_image",
          },
        },
      ],
      sidebar: [
        {
          label: "Guides",
          autogenerate: { directory: "guides" },
        },
        {
          label: "Examples",
          autogenerate: { directory: "examples" },
        },
        {
          label: "Reference",
          autogenerate: { directory: "reference" },
        },
      ],
    }),
    mdx(),
    favicons(),
    sitemap({
      filter: (page) => !/\^\/(?!cdn-cgi\/)/.test(page),
      changefreq: "weekly",
      priority: 0.7,
      lastmod: new Date(),
      namespaces: {
        image: false,
        video: false,
      },
    }),
  ],
});
