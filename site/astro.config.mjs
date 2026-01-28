// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import cloudflare from '@astrojs/cloudflare';

// https://astro.build/config
export default defineConfig({
	site: 'https://docs.knitli.com',
	base: '/ReCoco',
	adapter: cloudflare(),
	integrations: [
		starlight({
			title: 'ReCoco Docs',
			description: 'Incremental ETL and data processing framework in pure Rust. Feature-gated, modular architecture for sources, targets, and functions.',
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/knitli/recoco' }
			],
			customCss: [
				'./src/styles/custom.css',
			],
			head: [
				{
					tag: 'meta',
					attrs: {
						property: 'og:image',
						content: 'https://docs.knitli.com/ReCoco/og-image.png',
					},
				},
				{
					tag: 'meta',
					attrs: {
						property: 'twitter:card',
						content: 'summary_large_image',
					},
				},
			],
			sidebar: [
				{
					label: 'Guides',
					autogenerate: { directory: 'guides' },
				},
				{
					label: 'Examples',
					autogenerate: { directory: 'examples' },
				},
				{
					label: 'Reference',
					autogenerate: { directory: 'reference' },
				},
			],
		}),
	],
});