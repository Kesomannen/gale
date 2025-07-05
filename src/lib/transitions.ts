import { quadOut } from 'svelte/easing';

export const dropIn = dropInTo({ y: -5 });
export const dropOut = dropOutFrom({ y: -5 });

export function dropInTo({ x, y }: { x?: number; y?: number }) {
	return { duration: 75, easing: quadOut, x, y };
}

export function dropOutFrom({}: { x?: number; y?: number }) {
	return { duration: 50 };
}
