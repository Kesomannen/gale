/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			fontFamily: {
				sans: ['Nunito', 'sans-serif']
			}
		}
	},
	safelist: [
		'bg-green-700',
		'hover:bg-green-600'
	],
	plugins: []
};
