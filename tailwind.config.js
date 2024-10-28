/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			fontFamily: {
				sans: ['Nunito Sans', 'sans-serif'],
				mono: ['Roboto Mono', 'monospace']
			}
		}
	},
	safelist: [
		'bg-green-700',
		'enabled:hover:bg-green-600',
		'bg-red-700',
		'enabled:hover:bg-red-600',
		'enabled:hover:bg-gray-600',
		'ring-red-600',
		'font-semibold'
	],
	plugins: []
};
