import { fetch } from '@tauri-apps/plugin-http';

const root = document.querySelector(':root') as HTMLElement;

export async function setColor(hex: string, category: string) {
	let res = await fetch('https://www.tints.dev/api/color/' + hex);
	let colors = await res.json();
	let shades = colors.color as { [shade: string]: string };

	for (const [shade, value] of Object.entries(shades)) {
		root.style.setProperty(`--color-${category}-${shade}`, value);
	}

	localStorage.setItem(category + 'Color', hex);
}

/*
export function getColor(category: ColorCategory, fallback: Color) {
	return (localStorage.getItem(category + 'Color') as Color | null) ?? fallback;
}

export function refreshColor(category: ColorCategory, fallback: Color) {
	setColor(category, getColor(category, fallback));
}
*/
