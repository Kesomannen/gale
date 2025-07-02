<script lang="ts">
	import * as api from '$lib/api';
	import { activeGame, setActiveGame } from '$lib/stores.svelte';
	import type { Game } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { toHeaderCase } from 'js-convert-case';

	type Props = { game: Game; onselect?: () => void; onfavorite?: (favorite: boolean) => void };

	let { game, onselect, onfavorite }: Props = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	class={[
		$activeGame?.slug === game.slug
			? ' border-primary-500 bg-primary-700'
			: 'hover:bg-primary-700 border-transparent',
		'group hover:bg-primary-700 mr-2 flex cursor-pointer items-center rounded-lg border p-1.5 '
	]}
	onclick={() => {
		setActiveGame(game.slug);
		onselect?.();
	}}
	role="button"
	tabindex="0"
>
	<img src="games/{game.slug}.webp" alt={game.name} class="mr-2 size-12 rounded-sm" />

	<div class="grow pl-1 text-left">
		<div class="font-medium text-white">
			{game.name}
		</div>

		<div class="text-primary-400">
			<span>{game.modLoader} </span>

			{#if game.platforms.length > 0}
				<span class="text-primary-500 mx-1">|</span>

				<span class="mr-1">{game.platforms.map(toHeaderCase).join(', ')}</span>
			{/if}
		</div>
	</div>

	<button
		class={[
			game.favorite ? 'block' : 'hidden group-hover:block',
			'hover:bg-primary-600 mr-1 rounded p-1.5'
		]}
		onclick={(evt) => {
			evt.stopPropagation();
			onfavorite?.(!game.favorite);
			api.profile.favoriteGame(game.slug);
		}}
	>
		<Icon icon={game.favorite ? 'mdi:star' : 'mdi:star-outline'} class="text-accent-500 text-xl" />
	</button>
</div>
