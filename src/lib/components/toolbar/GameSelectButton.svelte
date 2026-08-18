<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';

	import Icon from '@iconify/svelte';
	import GameSelect from './GameSelect.svelte';
	import games from '$lib/state/game.svelte';
	import { m } from '$lib/paraglide/messages';
	import { gameIconSrc, timeSince } from '$lib/util';

	let gamesOpen = $state(false);

	let timeSinceGamesUpdate = $derived.by(() => {
		gamesOpen; // refresh whenever the dialog is opened
		return timeSince(games.lastUpdated);
	});
</script>

<button
	onclick={() => (gamesOpen = !gamesOpen)}
	class="group text-primary-300 group-hover:text-primary-200 hover:bg-primary-800 flex shrink-0 items-center rounded-lg px-4 py-1 font-semibold"
>
	<img
		src={games.active ? gameIconSrc(games.active) : ''}
		class="mr-2 size-8 rounded"
		alt={games.active?.name}
	/>

	<div class="mr-4 hidden lg:block">{games.active?.name}</div>

	<Icon icon="mdi:menu" class="text-primary-300 group-hover:text-primary-200  shrink-0 text-lg" />
</button>

<Dialog title={m.toolBar_dialog_games_title()} bind:open={gamesOpen}>
	<GameSelect onselect={() => (gamesOpen = false)} />
	<div class="text-primary-400 my-1 text-center text-sm">
		{m.toolBar_dialog_games_lastUpdated({ time: timeSinceGamesUpdate })}
	</div>
</Dialog>
