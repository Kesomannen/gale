/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		fontFamily: {
			sans: ['Nunito Sans', 'sans-serif'],
			mono: ['Roboto Mono', 'monospace']
		},
		extend: {
			colors: {
				accent: {
					50: 'var(--color-accent-50)',
					100: 'var(--color-accent-100)',
					200: 'var(--color-accent-200)',
					300: 'var(--color-accent-300)',
					400: 'var(--color-accent-400)',
					500: 'var(--color-accent-500)',
					600: 'var(--color-accent-600)',
					700: 'var(--color-accent-700)',
					800: 'var(--color-accent-800)',
					900: 'var(--color-accent-900)',
					950: 'var(--color-accent-950)'
				}
			}
		}
	},
	safelist: [
		'bg-accent-700',
		'enabled:hover:bg-accent-600',
		'bg-red-700',
		'enabled:hover:bg-red-600',
		'enabled:hover:bg-slate-600',
		'ring-red-600',
		'font-semibold'
	],
	plugins: []
};
