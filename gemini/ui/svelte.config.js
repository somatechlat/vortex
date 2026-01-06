import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter({
			pages: 'build',
			assets: 'build',
			fallback: 'index.html',
			precompress: true,
			strict: true
		}),
		alias: {
			$lib: './src/lib',
			$stores: './src/lib/stores',
			$components: './src/lib/components'
		}
	},
	compilerOptions: {
		runes: true  // Enable Svelte 5 Runes
	}
};

export default config;
