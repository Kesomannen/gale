import { platform } from '@tauri-apps/plugin-os';
import { PersistedState } from '$lib/state/persisted-state.svelte';
import getPalette from 'tailwindcss-palette-generator';
import * as api from '$lib/api';
import { rgbToHex } from '$lib/util';
import { DEFAULT_COLORS, type DefaultColor } from '$lib/default-colors';

/**
 * The value of a color setting. Can either be a system defined color given by the Rust API,
 * a default color from the `DEFAULT_COLORS` palette, or a custom color defined by a hex string.
 */
export type Color =
	| {
			type: 'system';
	  }
	| {
			type: 'default';
			name: DefaultColor;
	  }
	| {
			type: 'custom';
			hex: string;
	  };

const root = document.querySelector(':root') as HTMLElement;

export class ColorSetting {
	#name: string;
	#defaultColor: Color;
	#value: PersistedState<Color>;
	systemColorPromise: Promise<string | null> | null;

	constructor(
		name: string,
		defaultColor: Color,
		systemColorPromise: Promise<string | null> | null = null
	) {
		this.#name = name;
		this.#defaultColor = defaultColor;
		this.#value = new PersistedState<Color>(`${name}Color`, defaultColor);
		this.systemColorPromise = systemColorPromise;

		void this.applyShades(this.current);
	}

	get defaultColor(): Color {
		return this.#defaultColor;
	}

	get hasSystemColor() {
		return this.systemColorPromise !== null;
	}

	get current(): Color {
		return this.#value.current;
	}

	set current(color: Color) {
		this.#value.current = color;
		void this.applyShades(color);
	}

	async applyShades(color: Color) {
		const shades = await this.getShades(color);
		for (const [shade, value] of Object.entries(shades)) {
			root.style.setProperty(`--color-${this.#name}-${shade}`, value);
		}
	}

	async getShades(color: Color): Promise<{ [shade: string]: string }> {
		switch (color.type) {
			case 'custom':
				let palette = getPalette({
					color: color.hex,
					name: 'main',
					shade: 600
				});
				return palette['main'];

			case 'system':
				if (!this.systemColorPromise) {
					console.info('System accent color is not available, falling back to default');
					return await this.getShades(this.#defaultColor);
				}

				const systemColor = await this.systemColorPromise;
				if (systemColor) {
					let palette = getPalette({
						color: systemColor,
						name: 'main',
						shade: 600
					});
					return palette['main'];
				} else {
					console.info('System accent color is not available, falling back to default');
					return await this.getShades(this.#defaultColor);
				}

			case 'default':
				return DEFAULT_COLORS[color.name];
		}
	}
}

const systemAccentColorPromise = api.prefs.getSystemAccentColor().then((color) => {
	if (color) {
		return rgbToHex(color);
	}
	return null;
});

export const accentColorSetting = new ColorSetting(
	'accent',
	{ type: 'default', name: 'green' },
	systemAccentColorPromise
);
export const primaryColorSetting = new ColorSetting('primary', { type: 'default', name: 'slate' });

const defaultFont = 'Inter';

const font = new PersistedState<string>('font', defaultFont);

$effect.root(() => {
	$effect(() => {
		root.style.fontFamily = `'${font.current}', '${defaultFont}', sans-serif`;
	});
});

export function setFont(fontFamily: string) {
	font.current = fontFamily;
}

export function getFont() {
	return font.current;
}

export const useNativeMenu = new PersistedState(
	'useNativeMenu',
	platform() === 'windows' ? false : true
);
