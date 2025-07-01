import { quadOut } from 'svelte/easing';

export const dropIn = dropInTo({ y: -7 });
export const dropOut = dropOutFrom({ y: -7 });

export function dropInTo({ x, y }: { x?: number; y?: number }) {
	return { duration: 100, easing: quadOut, x, y };
};

export function dropOutFrom({}: { x?: number; y?: number }) {
	return { duration: 100 };
}
