// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: Apache-2.0

// @ts-check

import cloudflare from "@astrojs/cloudflare";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import favicons from "astro-favicons";
import { DocsAssets } from "@knitli/docs-components";
import { searchForWorkspaceRoot } from "vite";
import starlightHeadingBadges from "starlight-heading-badges";
import starlightContextualMenu from "starlight-contextual-menu";
import starlightTags from "starlight-tags";
import starlightLlmsText from "starlight-llms-txt";

const { headlineLogoDark, headlineLogoLight, variables, docsStyle, faviconIco, faviconSvg } = DocsAssets;

// https://astro.build/config
export default defineConfig({
  site: "https://docs.knitli.com",
  base: "/recoco",
  adapter: cloudflare({
    experimental: {
      headersAndRedirectsDevModeSupport: true,
    },
    configPath: "./wrangler.jsonc",
    imageService: "compile",
  }),
  favicon: faviconIco,
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
      "avatars.githubusercontent.com",
      "ui-avatars.com",
      "recoco.knitli.com",
    ],
  },
  headingIdCompat: true,
  preserveScriptOrder: true,
  // Build optimizations
  build: {
    inlineStylesheets: "auto",
    assets: "_astro",
  },
  markdown: {
    shikiConfig: {
      themes: {
        light: "catppuccin-latte",
        dark: "github-dark",
      }
    },
  },
  trailingSlash: 'always',
  // Vite configuration for better bundling
  vite: {
    server: {
      fs: {
        allow: [
          searchForWorkspaceRoot(process.cwd())
        ],
      },
    },
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
    rustCompiler: true,
  },
  // Static site generation for Cloudflare
  output: "static",
  integrations: [
    starlight({
      title: "Recoco Docs",
      logo: {
        dark: headlineLogoDark,
        light: headlineLogoLight,
        replacesTitle: true
      },
      plugins: [starlightHeadingBadges(), starlightContextualMenu({
        actions: ["copy", "view", "claude", "chatgpt"]
      }),
      // We need to configure starlight-tags with a tags.yml.
      //starlightTags(), 
      starlightLlmsText({
        projectName: "Recoco",
        description: `Recoco is a pure-Rust fork of the popular data processing and ETL framework, [CocoIndex](https://cocoindex.io). Recoco features a highly modular architecture, with all sources, targets, and functions (i.e. transforms) implemented as feature-gated plugins. This allows users to easily customize and extend the framework to fit their specific data processing needs, while keeping the core lightweight and efficient.
        Like CocoIndex, Recoco is fast, efficient, and works on data incrementally. Both libraries are built using a *dataflow architecture*, allowing you to easily define complex data processing pipelines with multiple sources, targets, and transforms in only a few lines of code. CocoIndex is built in Rust, but its entire API is only in Python, and does not allow for choosing integrations. Recoco exposes a robust Rust API that allows you to choose exactly which sources, targets, and transforms you want to use.
        `,
        promote: ["getting-started*", "architecture*", "file-processing*", "transient-flow*"],
        demote: ["contributing*", "changelog*"],
        minify: {
          whitespace: true,
          note: true,
          details: true,
        }
      })],
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
      customCss: [variables, docsStyle, "./src/styles/recoco.css"],
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
    favicons({
      name: "Recoco Docs by Knitli",
      short_name: "Recoco Docs",
      input: {
        favicons: [faviconSvg],
      },
    }),
    sitemap({
      filter: (page) => !/\^\/(?!cdn-cgi\/)/.test(page),
      changefreq: "weekly",
      priority: 0.4,
      lastmod: new Date(),
      namespaces: {
        image: false,
        video: false,
      },
    }),
  ],
});
