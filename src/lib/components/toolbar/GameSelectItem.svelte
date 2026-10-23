<script lang="ts">
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';
	import games from '$lib/state/game.svelte';
	import { Backend, type Game } from '$lib/types';
	import { gameIconSrc } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { toHeaderCase } from 'js-convert-case';

	type Props = { game: Game; onselect?: () => void; onfavorite?: (favorite: boolean) => void };

	let { game, onselect, onfavorite }: Props = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	class={[
		games.active?.slug === game.slug
			? ' border-primary-300 dark:border-primary-500 dark:bg-primary-700 bg-primary-100'
			: 'dark:hover:bg-primary-700 hover:bg-primary-50 border-transparent',
		'group dark:hover:bg-primary-700 hover:bg-primary-200 mr-2 flex cursor-pointer items-center rounded-lg border p-1.5'
	]}
	onclick={() => {
		games.setActive(game.slug);
		onselect?.();
	}}
	role="button"
	tabindex="0"
>
	<img src={gameIconSrc(game)} alt={game.name} class="mr-2 size-12 rounded-sm" />

	<div class="grow pl-1 text-left">
		<div class="text-primary-900 font-medium dark:text-white">
			{game.name}

			{#if game.backends.length === 1 && game.backends[0] === Backend.Hexium}
				<img
					src="hexium.ico"
					alt="[Hexium]"
					title={m.gameSelect_hexium_icon()}
					class="inline h-4"
				/>
			{/if}
		</div>

		<div class="text-primary-500 dark:text-primary-400">
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
			'dark:hover:bg-primary-600 hover:bg-primary-300 mr-1 rounded p-1.5'
		]}
		onclick={(evt) => {
			evt.stopPropagation();
			onfavorite?.(!game.favorite);
			api.profile.favoriteGame(game.slug);
		}}
	>
		<Icon
			icon={game.favorite ? 'mdi:star' : 'mdi:star-outline'}
			class="text-accent-600 dark:text-accent-500 text-xl"
		/>
	</button>
</div>
