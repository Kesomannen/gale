/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			fontFamily: {
				sans: ['Nunito Sans', 'sans-serif'],
				mono: ['Fira Code', 'monospace']
			},
			transitionDuration: {
				DEFAULT: '100ms'
			}
		}
	},
	safelist: [
		'max-w-[55%]',
		'max-w-[85%]'
	],
	plugins: []
};
