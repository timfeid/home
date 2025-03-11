import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import svg from '@poppanator/sveltekit-svg';

export default defineConfig({
	plugins: [
		sveltekit(),
		tailwindcss(),
		svg({
			includePaths: ['./src'],
			svgoOptions: {
				multipass: true,

				plugins: [
					{
						name: 'preset-default',
						params: { overrides: { removeViewBox: false } },
					},
					{
						name: 'reusePaths',
					},
					{ name: 'convertColors', params: { currentColor: true } },
				],
			},
		}),
	],
});
