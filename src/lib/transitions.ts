import { quadIn, quadOut } from 'svelte/easing';
import { fade, fly } from 'svelte/transition';

export const dropTransition = dropTransitionTo({ y: -7 });

export function dropTransitionTo({ x, y }: { x?: number; y?: number }) {
	return {
		inTransition: fly,
		inTransitionConfig: { duration: 100, easing: quadOut, x, y },
		outTransition: fade,
		outTransitionConfig: { duration: 100 }
	};
}

export const popupTransition = {
	inTransition: fly,
	inTransitionConfig: { duration: 150, easing: quadOut, y: 5 },
	outTransition: fly,
	outTransitionConfig: { duration: 100, easing: quadIn, y: 5 }
};
