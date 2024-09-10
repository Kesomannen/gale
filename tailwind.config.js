/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			fontFamily: {
				sans: ['Nunito Sans', 'sans-serif'],
				mono: ['Fira Code', 'monospace']
			}
		}
	},
	safelist: [
		'bg-green-700',
		'hover:bg-green-600',
		'bg-red-700',
		'hover:bg-red-600',
		'bg-blue-700',
		'hover:bg-blue-600',
		'border-red-600',
		'font-semibold',
		'max-w-[55%]',
		'max-w-[85%]'
	],
	plugins: []
};
