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
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/knitli/recoco' }
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